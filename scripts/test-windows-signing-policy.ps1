$ErrorActionPreference = "Stop"

$scriptFiles = @(
  "scripts/lib/windows-signing.ps1",
  "scripts/prepare-windows-signing.ps1",
  "scripts/cleanup-windows-signing.ps1",
  "scripts/build-windows-release.ps1",
  "scripts/stage-release.ps1",
  "scripts/verify-release-artifacts.ps1",
  "scripts/finalize-release-certification.ps1",
  "scripts/test-windows-installer.ps1"
)

foreach ($file in $scriptFiles) {
  $tokens = $null
  $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path $file).Path,
    [ref]$tokens,
    [ref]$errors
  )
  if ($errors.Count -gt 0) {
    $messages = $errors | ForEach-Object { $_.Message }
    throw ("PowerShell parse errors in " + $file + ": " + ($messages -join "; "))
  }
}

. (Join-Path $PSScriptRoot "lib\windows-signing.ps1")
$signTool = Get-ByteSignToolPath
if (!(Test-Path $signTool)) {
  throw "signtool.exe discovery returned a missing path."
}
Write-Host "signtool.exe available at $signTool"

$names = @(
  "WINDOWS_CERTIFICATE",
  "WINDOWS_CERTIFICATE_PASSWORD",
  "WINDOWS_TIMESTAMP_URL"
)
$backup = @{}
foreach ($name in $names) {
  $backup[$name] = [Environment]::GetEnvironmentVariable($name)
}

function Clear-TestSigningEnvironment {
  foreach ($name in $names) {
    [Environment]::SetEnvironmentVariable($name, $null)
  }
}

try {
  Clear-TestSigningEnvironment
  $failedClosed = $false
  try {
    & (Join-Path $PSScriptRoot "prepare-windows-signing.ps1") -RequireSigning
  } catch {
    if ($_.Exception.Message -notmatch "require Windows Authenticode signing") {
      throw
    }
    $failedClosed = $true
  }
  if (-not $failedClosed) {
    throw "Required signing unexpectedly succeeded without signing secrets."
  }

  [Environment]::SetEnvironmentVariable("WINDOWS_CERTIFICATE", "dGVzdA==")
  $partialFailed = $false
  try {
    & (Join-Path $PSScriptRoot "prepare-windows-signing.ps1")
  } catch {
    if ($_.Exception.Message -notmatch "partially configured") {
      throw
    }
    $partialFailed = $true
  }
  if (-not $partialFailed) {
    throw "Partial signing configuration unexpectedly succeeded."
  }

  Clear-TestSigningEnvironment
  $harnessPasswordText = "Byte-CI-" + [guid]::NewGuid().ToString("N")
  $harnessPassword = ConvertTo-SecureString -String $harnessPasswordText -AsPlainText -Force
  $harnessPfx = Join-Path $env:RUNNER_TEMP ("byte-ci-signing-" + [guid]::NewGuid().ToString("N") + ".pfx")
  $certArgs = @{
    Subject = "CN=Byte CI Trust Harness"
    Type = "CodeSigningCert"
    CertStoreLocation = "Cert:\CurrentUser\My"
    KeyExportPolicy = "Exportable"
    NotAfter = (Get-Date).AddDays(2)
  }
  $harnessCertificate = New-SelfSignedCertificate @certArgs

  try {
    $exportArgs = @{
      Cert = $harnessCertificate
      FilePath = $harnessPfx
      Password = $harnessPassword
    }
    Export-PfxCertificate @exportArgs | Out-Null

    $base64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes($harnessPfx))
    [Environment]::SetEnvironmentVariable("WINDOWS_CERTIFICATE", $base64)
    [Environment]::SetEnvironmentVariable("WINDOWS_CERTIFICATE_PASSWORD", $harnessPasswordText)
    [Environment]::SetEnvironmentVariable("WINDOWS_TIMESTAMP_URL", "https://timestamp.invalid.example/")

    & (Join-Path $PSScriptRoot "prepare-windows-signing.ps1") -RequireSigning

    $configPath = "src-tauri/tauri.signing.conf.json"
    if (!(Test-Path $configPath)) {
      throw "Signing preparation did not create the temporary Tauri signing configuration."
    }

    $signingConfig = Get-Content $configPath -Raw | ConvertFrom-Json
    $preparedThumbprint = ([string]$signingConfig.bundle.windows.certificateThumbprint).ToUpperInvariant()
    if ($preparedThumbprint -ne $harnessCertificate.Thumbprint.ToUpperInvariant()) {
      throw "Prepared Tauri signing thumbprint does not match the harness certificate."
    }
    if ([string]$signingConfig.bundle.windows.digestAlgorithm -ne "sha256") {
      throw "Prepared Tauri signing digest must remain sha256."
    }
    if ([string]$signingConfig.bundle.windows.timestampUrl -ne "https://timestamp.invalid.example/") {
      throw "Prepared Tauri timestamp URL does not match the configured value."
    }

    $metadataPath = Get-ByteSigningMetadataPath
    if (!(Test-Path $metadataPath)) {
      throw "Signing preparation did not create trust metadata."
    }
    $metadata = Get-Content $metadataPath -Raw | ConvertFrom-Json
    if ([string]$metadata.subject -notmatch "Byte CI Trust Harness") {
      throw "Signing metadata did not preserve the certificate subject."
    }
    if ([string]$metadata.thumbprint -ne $preparedThumbprint) {
      throw "Signing metadata thumbprint does not match the Tauri signing configuration."
    }

    & (Join-Path $PSScriptRoot "cleanup-windows-signing.ps1")

    if (Test-Path $configPath) {
      throw "Signing cleanup left the temporary Tauri signing configuration behind."
    }
    if (Test-Path $metadataPath) {
      throw "Signing cleanup left certificate metadata behind."
    }
  } finally {
    Remove-Item $harnessPfx -Force -ErrorAction SilentlyContinue
    Remove-Item ("Cert:\CurrentUser\My\" + $harnessCertificate.Thumbprint) -Force -ErrorAction SilentlyContinue
    & (Join-Path $PSScriptRoot "cleanup-windows-signing.ps1")
  }
} finally {
  foreach ($name in $names) {
    [Environment]::SetEnvironmentVariable($name, $backup[$name])
  }
}

Write-Host "Windows signing policy smoke test passed."
