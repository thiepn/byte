param(
  [string]$OutputDir = "release-artifacts",
  [switch]$RequireCertification,
  [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.IO.Compression.FileSystem
. (Join-Path $PSScriptRoot "lib\windows-signing.ps1")

$root = (Resolve-Path $OutputDir).Path
$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version

$installerName = "Byte-v$version-windows-x64-setup.exe"
$portableName = "Byte-v$version-windows-x64-portable.zip"
$manifestName = "release-manifest.json"
$certificationName = "release-certification.json"
$productCertificationName = "product-certification.json"
$checksumName = "SHA256SUMS.txt"

$installer = Join-Path $root $installerName
$portable = Join-Path $root $portableName
$manifestPath = Join-Path $root $manifestName
$certificationPath = Join-Path $root $certificationName
$productCertificationPath = Join-Path $root $productCertificationName
$checksumPath = Join-Path $root $checksumName

foreach ($required in @($installer, $portable, $manifestPath, $checksumPath)) {
  if (!(Test-Path $required)) {
    throw "Required release artifact is missing: $required"
  }
}

if ((Get-Item $installer).Length -lt 100KB) {
  throw "Installer artifact is unexpectedly small."
}
if ((Get-Item $portable).Length -lt 100KB) {
  throw "Portable artifact is unexpectedly small."
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
if ($manifest.schema_version -ne 2) { throw "Release manifest schema mismatch." }
if ($manifest.product -ne "Byte") { throw "Release manifest product mismatch." }
if ([string]$manifest.version -ne $version) { throw "Release manifest version mismatch." }
if ($manifest.identifier -ne "io.github.thiepn.byte") { throw "Release manifest identifier mismatch." }
if ($manifest.publisher -ne [string]$config.bundle.publisher) { throw "Release manifest publisher mismatch." }
if ($manifest.homepage -ne [string]$config.bundle.homepage) { throw "Release manifest homepage mismatch." }
if ($manifest.target -ne "x86_64-pc-windows-msvc") { throw "Release manifest target mismatch." }
if ($manifest.installer -ne $installerName) { throw "Release manifest installer filename mismatch." }
if ($manifest.portable -ne $portableName) { throw "Release manifest portable filename mismatch." }
if ($manifest.installer_scope -ne "currentUser") { throw "Release manifest install scope mismatch." }
if ($manifest.downgrades_allowed -ne $false) { throw "Release manifest downgrade policy mismatch." }

$checksumMap = @{}
foreach ($line in Get-Content $checksumPath) {
  if ([string]::IsNullOrWhiteSpace($line)) { continue }
  if ($line -notmatch '^([0-9a-fA-F]{64})\s{2}(.+)$') {
    throw "Malformed checksum line: $line"
  }
  $checksumMap[$matches[2]] = $matches[1].ToLowerInvariant()
}

$hashTargets = @($installerName, $portableName, $manifestName)
if (Test-Path $productCertificationPath) {
  $hashTargets += $productCertificationName
} elseif ($RequireCertification) {
  throw "Certified release artifacts require $productCertificationName."
}

if (Test-Path $certificationPath) {
  $hashTargets += $certificationName
} elseif ($RequireCertification) {
  throw "Certified release artifacts require $certificationName."
}

foreach ($name in $hashTargets) {
  if (!$checksumMap.ContainsKey($name)) {
    throw "SHA256SUMS.txt does not contain $name."
  }
  $actual = (Get-FileHash (Join-Path $root $name) -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $checksumMap[$name]) {
    throw "SHA-256 mismatch for $name."
  }
}

$temp = Join-Path $env:RUNNER_TEMP ("byte-artifact-verify-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temp | Out-Null

try {
  [System.IO.Compression.ZipFile]::ExtractToDirectory($portable, $temp)
  $entries = @(Get-ChildItem $temp -File -Recurse)
  if ($entries.Count -ne 1 -or $entries[0].Name -ne "Byte.exe") {
    throw "Portable ZIP must contain exactly one Byte.exe."
  }

  $binary = $entries[0].FullName
  $versionInfo = (Get-Item $binary).VersionInfo
  $reportedVersion = [string]$versionInfo.ProductVersion
  if ([string]::IsNullOrWhiteSpace($reportedVersion)) {
    $reportedVersion = [string]$versionInfo.FileVersion
  }
  if ([string]::IsNullOrWhiteSpace($reportedVersion) -or -not $reportedVersion.StartsWith($version)) {
    throw "Portable Byte.exe version '$reportedVersion' does not match '$version'."
  }

  $stream = [IO.File]::OpenRead($binary)
  try {
    $reader = [IO.BinaryReader]::new($stream)
    $stream.Position = 0x3c
    $peOffset = $reader.ReadInt32()
    $stream.Position = $peOffset
    $signature = $reader.ReadUInt32()
    if ($signature -ne 0x00004550) { throw "Byte.exe does not contain a valid PE signature." }
    $machine = $reader.ReadUInt16()
    if ($machine -ne 0x8664) {
      throw ("Portable Byte.exe is not x64 (PE machine 0x{0:x4})." -f $machine)
    }
  } finally {
    if ($null -ne $reader) { $reader.Dispose() }
    $stream.Dispose()
  }

  $binarySignature = Get-ByteSignatureDetails -Path $binary
  $installerSignature = Get-ByteSignatureDetails -Path $installer
  $actuallySigned = [bool]$binarySignature.valid -and [bool]$installerSignature.valid
  $sameSigner = $actuallySigned -and ($binarySignature.signer_thumbprint -eq $installerSignature.signer_thumbprint)
  $timestamped = $actuallySigned -and [bool]$binarySignature.timestamped -and [bool]$installerSignature.timestamped

  if ([bool]$manifest.signed -ne $actuallySigned) {
    throw "Release manifest signing state does not match the staged binaries."
  }
  if ([bool]$manifest.signing.same_signer -ne [bool]$sameSigner) {
    throw "Release manifest same-signer state does not match the staged binaries."
  }
  if ([bool]$manifest.signing.timestamped -ne [bool]$timestamped) {
    throw "Release manifest timestamp state does not match the staged binaries."
  }

  if ($actuallySigned) {
    if (-not $sameSigner) { throw "Portable Byte.exe and installer have different Authenticode signers." }
    if ([string]$manifest.signing.application.signer_thumbprint -ne $binarySignature.signer_thumbprint) {
      throw "Release manifest application signer thumbprint mismatch."
    }
    if ([string]$manifest.signing.installer.signer_thumbprint -ne $installerSignature.signer_thumbprint) {
      throw "Release manifest installer signer thumbprint mismatch."
    }
  }

  if ($RequireSigning) {
    if (-not [bool]$manifest.signing.required_for_this_build) {
      throw "This verification requires signing, but the release manifest was not staged in required-signing mode."
    }
    if (-not $actuallySigned) {
      throw "This public release requires valid Authenticode signatures."
    }
    if (-not $timestamped) {
      throw "This public release requires timestamped Authenticode signatures."
    }
  }

  if ($RequireCertification) {
    $productCertification = Get-Content $productCertificationPath -Raw | ConvertFrom-Json
    if ($productCertification.schema_version -ne 1) {
      throw "Product certification schema mismatch."
    }
    if ($productCertification.product -ne "Byte" -or [string]$productCertification.version -ne $version) {
      throw "Product certification identity mismatch."
    }
    if ($productCertification.target -ne "x86_64-pc-windows-msvc") {
      throw "Product certification target mismatch."
    }
    if ($productCertification.automated_ready -ne $true) {
      throw "Product certification does not mark the automated runtime gate ready."
    }
    if ($productCertification.manual_device_signoff_required -ne $true) {
      throw "Product certification must preserve the manual device signoff requirement."
    }
    $requiredProductChecks = @(
      "fresh_first_run",
      "onboarding_boundary",
      "all_display_modes",
      "interface_scales_100_through_200",
      "high_contrast_profile",
      "reduced_motion_profile",
      "monitoring_disabled_profile",
      "runtime_schema_migration",
      "corrupt_config_quarantine_and_recovery",
      "sustained_runtime",
      "runaway_resource_guardrails"
    )
    foreach ($check in $requiredProductChecks) {
      if ($productCertification.automated_checks.$check -ne $true) {
        throw "Product certification check '$check' is not green."
      }
    }
    if (($productCertification.supported_interface_scales -join ",") -ne "100,110,125,150,175,200") {
      throw "Product certification interface-scale coverage mismatch."
    }
    if (($productCertification.exercised_display_modes -join ",") -ne "HABITAT,PERCH,MINI,EDGE,TRAY") {
      throw "Product certification display-mode coverage mismatch."
    }
    if ([int]$productCertification.sustained_runtime_seconds -lt 30) {
      throw "Product certification sustained runtime is below policy."
    }

    $certification = Get-Content $certificationPath -Raw | ConvertFrom-Json
    if ($certification.schema_version -ne 3) {
      throw "Release certification schema mismatch."
    }
    if ($certification.product -ne "Byte" -or [string]$certification.version -ne $version) {
      throw "Release certification identity mismatch."
    }
    if ($certification.target -ne "x86_64-pc-windows-msvc") {
      throw "Release certification target mismatch."
    }
    if ($certification.distribution_ready -ne $true) {
      throw "Release certification does not mark the build distribution-ready."
    }
    if ($certification.checks.real_world_product_runtime -ne $true -or
        $certification.checks.fresh_first_run -ne $true -or
        $certification.checks.onboarding_boundary -ne $true -or
        $certification.checks.display_mode_matrix -ne $true -or
        $certification.checks.interface_scale_matrix -ne $true -or
        $certification.checks.runtime_schema_migration -ne $true -or
        $certification.checks.corrupt_config_recovery -ne $true -or
        $certification.checks.sustained_runtime -ne $true -or
        $certification.checks.runaway_resource_guardrails -ne $true -or
        $certification.checks.manual_device_signoff_gate_declared -ne $true) {
      throw "Release certification is missing P4 product-quality gates."
    }
    $productHash = (Get-FileHash $productCertificationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([string]$certification.product_certification.sha256 -ne $productHash) {
      throw "Release certification product-certification hash mismatch."
    }
    if ($RequireSigning -and $certification.public_distribution_ready -ne $true) {
      throw "Release certification does not mark the signed build public-distribution-ready."
    }
  }

  Write-Host "Release artifact verification passed for Byte v$version."
  Write-Host "Authenticode: signed=$actuallySigned timestamped=$timestamped same_signer=$sameSigner"
} finally {
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
