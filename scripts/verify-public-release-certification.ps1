param(
  [Parameter(Mandatory = $true)]
  [string]$PublishedDir,
  [string]$Certification = "",
  [switch]$RequireRemoteState,
  [switch]$RequireWorkflowSuccess,
  [switch]$RecheckRuntime
)

$ErrorActionPreference = "Stop"

$repository = "thiepn/byte"

function Require-Command([string]$Name) {
  if ($null -eq (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Required command '$Name' is not available on PATH."
  }
}

function Invoke-GhJson([string[]]$Arguments) {
  $output = & gh @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "GitHub CLI command failed: gh $($Arguments -join ' ')"
  }
  $raw = ($output | Out-String).Trim()
  if ([string]::IsNullOrWhiteSpace($raw)) {
    return $null
  }
  return $raw | ConvertFrom-Json
}

function Digest([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function TextDigest([string]$Text) {
  $normalized = ($Text -replace "\r\n", "\n").TrimEnd()
  $bytes = [Text.Encoding]::UTF8.GetBytes($normalized)
  $sha = [Security.Cryptography.SHA256]::Create()
  try {
    return ([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace("-", "").ToLowerInvariant()
  } finally {
    $sha.Dispose()
  }
}

function Assert-Equal([object]$Actual, [object]$Expected, [string]$Message) {
  if ($Actual -ne $Expected) {
    throw "$Message Expected '$Expected', got '$Actual'."
  }
}

$root = (Resolve-Path -LiteralPath $PublishedDir).Path
if ([string]::IsNullOrWhiteSpace($Certification)) {
  $Certification = Join-Path $root "public-release-certification.json"
}
$certPath = (Resolve-Path -LiteralPath $Certification).Path

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$expectedTag = "v$version"

$manifestPath = Join-Path $root "release-manifest.json"
$productCertificationPath = Join-Path $root "product-certification.json"
$releaseCertificationPath = Join-Path $root "release-certification.json"
$checksumPath = Join-Path $root "SHA256SUMS.txt"

$artifactVerifyArgs = @{
  OutputDir = $root
  RequireCertification = $true
  RequireSigning = $true
}
& (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") @artifactVerifyArgs

$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$releaseCertification = Get-Content -LiteralPath $releaseCertificationPath -Raw | ConvertFrom-Json
$certData = Get-Content -LiteralPath $certPath -Raw | ConvertFrom-Json

Assert-Equal $certData.schema_version 1 "Public release certification schema mismatch."
Assert-Equal $certData.product "Byte" "Public release certification product mismatch."
Assert-Equal ([string]$certData.version) $version "Public release certification version mismatch."
Assert-Equal ([string]$certData.tag) $expectedTag "Public release certification tag mismatch."
Assert-Equal ([string]$certData.source_commit) ([string]$manifest.commit) "Public release certification source commit mismatch."
Assert-Equal ([string]$certData.certification_scope) "fresh-public-download-post-publish" "Public release certification scope mismatch."

$isBeta = $version -match '-beta\.'
$expectedChannel = if ($isBeta) { "beta" } else { "stable" }
Assert-Equal ([string]$certData.channel) $expectedChannel "Public release certification channel mismatch."

Assert-Equal ([string]$certData.release.tag_name) $expectedTag "Public release certification release tag mismatch."
Assert-Equal ([string]$certData.release.name) ("Byte " + $version) "Public release certification release title mismatch."
if ($certData.release.draft -ne $false) {
  throw "Public release certification must record a non-draft release."
}
Assert-Equal ([bool]$certData.release.prerelease) ([bool]$isBeta) "Public release certification prerelease state mismatch."
$expectedLatest = -not $isBeta
Assert-Equal ([bool]$certData.release.latest) ([bool]$expectedLatest) "Public release certification latest-release state mismatch."
if ([long]$certData.release.database_id -le 0) {
  throw "Public release certification release database ID is invalid."
}
if ([string]::IsNullOrWhiteSpace([string]$certData.release.url) -or
    [string]$certData.release.url -notmatch '^https://github\.com/thiepn/byte/releases/tag/') {
  throw "Public release certification release URL is invalid."
}
if ([string]::IsNullOrWhiteSpace([string]$certData.release.published_at)) {
  throw "Public release certification publication timestamp is missing."
}
if ([string]$certData.release.body_sha256 -notmatch '^[0-9a-f]{64}if ($certData.public_distribution_ready -ne $true) {
  throw "Public release certification does not mark public_distribution_ready."
}
if ($certData.receipt_self_hashed -ne $false) {
  throw "Public release certification must not claim to hash itself."
}
if ($releaseCertification.public_distribution_ready -ne $true) {
  throw "Downloaded release certification does not mark public_distribution_ready."
}

$requiredChecks = @(
  "exact_asset_set",
  "sha256_and_manifest",
  "authenticode_and_timestamp",
  "same_signer",
  "release_certification_public_distribution_ready",
  "github_provenance_installer",
  "github_provenance_portable",
  "public_portable_launch",
  "public_installer_lifecycle",
  "release_title_and_tag",
  "release_notes_match",
  "channel_metadata",
  "exact_lightweight_tag_target"
)
foreach ($check in $requiredChecks) {
  if ($certData.checks.$check -ne $true) {
    throw "Public release certification check '$check' is not green."
  }
}

if ($certData.signing.signed -ne $true -or
    $certData.signing.timestamped -ne $true -or
    $certData.signing.same_signer -ne $true) {
  throw "Public release certification signing state is incomplete."
}
Assert-Equal ([string]$certData.signing.application_signer_thumbprint) ([string]$manifest.signing.application.signer_thumbprint) "Public release application signer mismatch."
Assert-Equal ([string]$certData.signing.installer_signer_thumbprint) ([string]$manifest.signing.installer.signer_thumbprint) "Public release installer signer mismatch."

Assert-Equal ([string]$certData.release_manifest_sha256) (Digest $manifestPath) "Public release manifest hash mismatch."
Assert-Equal ([string]$certData.product_certification_sha256) (Digest $productCertificationPath) "Public product-certification hash mismatch."
Assert-Equal ([string]$certData.release_certification_sha256) (Digest $releaseCertificationPath) "Public release-certification hash mismatch."
Assert-Equal ([string]$certData.checksum_file_sha256) (Digest $checksumPath) "Public checksum-file hash mismatch."

$baseAssets = @(
  [string]$manifest.installer,
  [string]$manifest.portable,
  "release-manifest.json",
  "product-certification.json",
  "release-certification.json",
  "SHA256SUMS.txt"
)

$certAssetNames = @($certData.assets | ForEach-Object { [string]$_.name } | Sort-Object)
$expectedBaseList = ($baseAssets | Sort-Object) -join [Environment]::NewLine
if (($certAssetNames -join [Environment]::NewLine) -ne $expectedBaseList) {
  throw "Public release certification asset inventory mismatch."
}

foreach ($entry in @($certData.assets)) {
  $name = [string]$entry.name
  $path = Join-Path $root $name
  if (!(Test-Path -LiteralPath $path)) {
    throw "Public release certification references a missing asset: $name"
  }
  Assert-Equal ([long]$entry.size_bytes) ([long](Get-Item -LiteralPath $path).Length) "Public asset size mismatch for $name."
  Assert-Equal ([string]$entry.sha256) (Digest $path) "Public asset hash mismatch for $name."
}

$finalAssets = @($baseAssets + "public-release-certification.json") | Sort-Object
$localNames = @(Get-ChildItem -LiteralPath $root -File | Select-Object -ExpandProperty Name | Sort-Object)
if (($localNames -join [Environment]::NewLine) -ne ($finalAssets -join [Environment]::NewLine)) {
  throw "Completed public release download does not contain exactly the seven expected P7 assets."
}

if ($RecheckRuntime) {
  $portablePath = Join-Path $root ([string]$manifest.portable)
  $installerPath = Join-Path $root ([string]$manifest.installer)

  & (Join-Path $PSScriptRoot "test-portable-launch.ps1") -Portable $portablePath

  $installerArgs = @{
    Installer = $installerPath
    RequireSigning = $true
  }
  & (Join-Path $PSScriptRoot "test-windows-installer.ps1") @installerArgs
}

if ($RequireRemoteState -or $RequireWorkflowSuccess) {
  Require-Command "gh"
  & gh auth status
  if ($LASTEXITCODE -ne 0) {
    throw "GitHub CLI authentication is required for remote P7 verification."
  }

  $release = Invoke-GhJson @(
    "api",
    "repos/$repository/releases/tags/$expectedTag"
  )

  Assert-Equal ([long]$release.id) ([long]$certData.release.database_id) "Remote release database ID mismatch."
  Assert-Equal ([string]$release.tag_name) $expectedTag "Remote release tag mismatch."
  Assert-Equal ([string]$release.name) ("Byte " + $version) "Remote release title mismatch."
  Assert-Equal ([string]$release.html_url) ([string]$certData.release.url) "Remote release URL mismatch."
  if ($release.draft -ne $false) {
    throw "Remote release unexpectedly became a draft."
  }
  Assert-Equal ([bool]$release.prerelease) ([bool]$isBeta) "Remote release prerelease/channel mismatch."
  if (-not $isBeta) {
    $latestRelease = Invoke-GhJson @(
      "api",
      "repos/$repository/releases/latest"
    )
    if ([string]$latestRelease.tag_name -ne $expectedTag) {
      throw "Stable completed release is not the repository's latest release."
    }
  }
  Assert-Equal ([string]$release.published_at) ([string]$certData.release.published_at) "Remote release publication timestamp mismatch."
  Assert-Equal (TextDigest ([string]$release.body)) ([string]$certData.release.body_sha256) "Remote release notes changed after certification."

  $remoteNames = @($release.assets | ForEach-Object { [string]$_.name } | Sort-Object)
  if (($remoteNames -join [Environment]::NewLine) -ne ($finalAssets -join [Environment]::NewLine)) {
    throw "Remote completed release asset set is not exactly the seven expected P7 assets."
  }

  foreach ($name in $finalAssets) {
    $remoteAsset = @($release.assets | Where-Object { [string]$_.name -eq $name })
    if ($remoteAsset.Count -ne 1) {
      throw "Remote completed release must contain exactly one asset named '$name'."
    }
    $localPath = Join-Path $root $name
    if ($null -ne $remoteAsset[0].size -and [long]$remoteAsset[0].size -ne [long](Get-Item -LiteralPath $localPath).Length) {
      throw "Remote completed release size mismatch for '$name'."
    }
    if (-not [string]::IsNullOrWhiteSpace([string]$remoteAsset[0].digest)) {
      Assert-Equal ([string]$remoteAsset[0].digest) ("sha256:" + (Digest $localPath)) "Remote completed release digest mismatch for $name."
    }
  }

  $tagRef = Invoke-GhJson @(
    "api",
    "repos/$repository/git/ref/tags/$expectedTag"
  )
  Assert-Equal ([string]$tagRef.ref) ("refs/tags/" + $expectedTag) "Remote completed release tag ref mismatch."
  Assert-Equal ([string]$tagRef.object.type) "commit" "Completed release tag must remain lightweight."
  Assert-Equal ([string]$tagRef.object.sha) ([string]$manifest.commit) "Completed release tag target mismatch."

  $installerPath = Join-Path $root ([string]$manifest.installer)
  $portablePath = Join-Path $root ([string]$manifest.portable)
  foreach ($artifact in @($installerPath, $portablePath)) {
    & gh attestation verify $artifact --repo $repository
    if ($LASTEXITCODE -ne 0) {
      throw "Remote completed release provenance verification failed for $artifact."
    }
  }

  $runId = [long]$certData.workflow.run_id
  $workflow = Invoke-GhJson @(
    "api",
    "repos/$repository/actions/runs/$runId"
  )

  Assert-Equal ([long]$workflow.id) $runId "Remote release workflow run ID mismatch."
  Assert-Equal ([string]$workflow.name) "Byte Release" "Remote release workflow name mismatch."
  Assert-Equal ([string]$workflow.event) "push" "Remote release workflow event mismatch."
  Assert-Equal ([string]$workflow.head_sha) ([string]$manifest.commit) "Remote release workflow source SHA mismatch."
  if (-not [string]::IsNullOrWhiteSpace([string]$workflow.head_branch) -and
      [string]$workflow.head_branch -ne $expectedTag) {
    throw "Remote release workflow belongs to '$($workflow.head_branch)', not '$expectedTag'."
  }
  Assert-Equal ([string]$workflow.html_url) ([string]$certData.workflow.url) "Remote release workflow URL mismatch."

  if ($RequireWorkflowSuccess) {
    if ($workflow.status -ne "completed" -or $workflow.conclusion -ne "success") {
      throw "Byte Release workflow has not completed successfully."
    }
  }
}

Write-Host "P7 public release certification verified for Byte $version."
Write-Host "Tag: $expectedTag"
Write-Host "Public distribution ready: True"
) {
  throw "Public release certification release-notes hash is invalid."
}

Assert-Equal ([string]$certData.tag_identity.type) "lightweight" "Public release certification tag type mismatch."
Assert-Equal ([string]$certData.tag_identity.ref) ("refs/tags/" + $expectedTag) "Public release certification tag ref mismatch."
Assert-Equal ([string]$certData.tag_identity.target_commit) ([string]$manifest.commit) "Public release certification tag target mismatch."

Assert-Equal ([string]$certData.workflow.name) "Byte Release" "Public release certification workflow name mismatch."
Assert-Equal ([string]$certData.workflow.event) "push" "Public release certification workflow event mismatch."
Assert-Equal ([string]$certData.workflow.head_sha) ([string]$manifest.commit) "Public release certification workflow source mismatch."
if (-not [string]::IsNullOrWhiteSpace([string]$certData.workflow.head_branch) -and
    [string]$certData.workflow.head_branch -ne $expectedTag) {
  throw "Public release certification workflow tag mismatch."
}
if ([long]$certData.workflow.run_id -le 0) {
  throw "Public release certification workflow run ID is invalid."
}

if ($certData.public_distribution_ready -ne $true) {
  throw "Public release certification does not mark public_distribution_ready."
}
if ($certData.receipt_self_hashed -ne $false) {
  throw "Public release certification must not claim to hash itself."
}
if ($releaseCertification.public_distribution_ready -ne $true) {
  throw "Downloaded release certification does not mark public_distribution_ready."
}

$requiredChecks = @(
  "exact_asset_set",
  "sha256_and_manifest",
  "authenticode_and_timestamp",
  "same_signer",
  "release_certification_public_distribution_ready",
  "github_provenance_installer",
  "github_provenance_portable",
  "public_portable_launch",
  "public_installer_lifecycle",
  "release_title_and_tag",
  "release_notes_match",
  "channel_metadata",
  "exact_lightweight_tag_target"
)
foreach ($check in $requiredChecks) {
  if ($certData.checks.$check -ne $true) {
    throw "Public release certification check '$check' is not green."
  }
}

if ($certData.signing.signed -ne $true -or
    $certData.signing.timestamped -ne $true -or
    $certData.signing.same_signer -ne $true) {
  throw "Public release certification signing state is incomplete."
}
Assert-Equal ([string]$certData.signing.application_signer_thumbprint) ([string]$manifest.signing.application.signer_thumbprint) "Public release application signer mismatch."
Assert-Equal ([string]$certData.signing.installer_signer_thumbprint) ([string]$manifest.signing.installer.signer_thumbprint) "Public release installer signer mismatch."

Assert-Equal ([string]$certData.release_manifest_sha256) (Digest $manifestPath) "Public release manifest hash mismatch."
Assert-Equal ([string]$certData.product_certification_sha256) (Digest $productCertificationPath) "Public product-certification hash mismatch."
Assert-Equal ([string]$certData.release_certification_sha256) (Digest $releaseCertificationPath) "Public release-certification hash mismatch."
Assert-Equal ([string]$certData.checksum_file_sha256) (Digest $checksumPath) "Public checksum-file hash mismatch."

$baseAssets = @(
  [string]$manifest.installer,
  [string]$manifest.portable,
  "release-manifest.json",
  "product-certification.json",
  "release-certification.json",
  "SHA256SUMS.txt"
)

$certAssetNames = @($certData.assets | ForEach-Object { [string]$_.name } | Sort-Object)
$expectedBaseList = ($baseAssets | Sort-Object) -join [Environment]::NewLine
if (($certAssetNames -join [Environment]::NewLine) -ne $expectedBaseList) {
  throw "Public release certification asset inventory mismatch."
}

foreach ($entry in @($certData.assets)) {
  $name = [string]$entry.name
  $path = Join-Path $root $name
  if (!(Test-Path -LiteralPath $path)) {
    throw "Public release certification references a missing asset: $name"
  }
  Assert-Equal ([long]$entry.size_bytes) ([long](Get-Item -LiteralPath $path).Length) "Public asset size mismatch for $name."
  Assert-Equal ([string]$entry.sha256) (Digest $path) "Public asset hash mismatch for $name."
}

$finalAssets = @($baseAssets + "public-release-certification.json" | Sort-Object)
$localNames = @(Get-ChildItem -LiteralPath $root -File | Select-Object -ExpandProperty Name | Sort-Object)
if (($localNames -join [Environment]::NewLine) -ne ($finalAssets -join [Environment]::NewLine)) {
  throw "Completed public release download does not contain exactly the seven expected P7 assets."
}

if ($RecheckRuntime) {
  $portablePath = Join-Path $root ([string]$manifest.portable)
  $installerPath = Join-Path $root ([string]$manifest.installer)

  & (Join-Path $PSScriptRoot "test-portable-launch.ps1") -Portable $portablePath

  $installerArgs = @{
    Installer = $installerPath
    RequireSigning = $true
  }
  & (Join-Path $PSScriptRoot "test-windows-installer.ps1") @installerArgs
}

if ($RequireRemoteState -or $RequireWorkflowSuccess) {
  Require-Command "gh"
  & gh auth status
  if ($LASTEXITCODE -ne 0) {
    throw "GitHub CLI authentication is required for remote P7 verification."
  }

  $release = Invoke-GhJson @(
    "api",
    "repos/$repository/releases/tags/$expectedTag"
  )

  Assert-Equal ([long]$release.id) ([long]$certData.release.database_id) "Remote release database ID mismatch."
  Assert-Equal ([string]$release.tag_name) $expectedTag "Remote release tag mismatch."
  Assert-Equal ([string]$release.name) ("Byte " + $version) "Remote release title mismatch."
  Assert-Equal ([string]$release.html_url) ([string]$certData.release.url) "Remote release URL mismatch."
  if ($release.draft -ne $false) {
    throw "Remote release unexpectedly became a draft."
  }
  Assert-Equal ([bool]$release.prerelease) ([bool]$isBeta) "Remote release prerelease/channel mismatch."
  if (-not $isBeta) {
    $latestRelease = Invoke-GhJson @(
      "api",
      "repos/$repository/releases/latest"
    )
    if ([string]$latestRelease.tag_name -ne $expectedTag) {
      throw "Stable completed release is not the repository's latest release."
    }
  }
  Assert-Equal ([string]$release.published_at) ([string]$certData.release.published_at) "Remote release publication timestamp mismatch."
  Assert-Equal (TextDigest ([string]$release.body)) ([string]$certData.release.body_sha256) "Remote release notes changed after certification."

  $remoteNames = @($release.assets | ForEach-Object { [string]$_.name } | Sort-Object)
  if (($remoteNames -join [Environment]::NewLine) -ne ($finalAssets -join [Environment]::NewLine)) {
    throw "Remote completed release asset set is not exactly the seven expected P7 assets."
  }

  foreach ($name in $finalAssets) {
    $remoteAsset = @($release.assets | Where-Object { [string]$_.name -eq $name })
    if ($remoteAsset.Count -ne 1) {
      throw "Remote completed release must contain exactly one asset named '$name'."
    }
    $localPath = Join-Path $root $name
    if ($null -ne $remoteAsset[0].size -and [long]$remoteAsset[0].size -ne [long](Get-Item -LiteralPath $localPath).Length) {
      throw "Remote completed release size mismatch for '$name'."
    }
    if (-not [string]::IsNullOrWhiteSpace([string]$remoteAsset[0].digest)) {
      Assert-Equal ([string]$remoteAsset[0].digest) ("sha256:" + (Digest $localPath)) "Remote completed release digest mismatch for $name."
    }
  }

  $tagRef = Invoke-GhJson @(
    "api",
    "repos/$repository/git/ref/tags/$expectedTag"
  )
  Assert-Equal ([string]$tagRef.ref) ("refs/tags/" + $expectedTag) "Remote completed release tag ref mismatch."
  Assert-Equal ([string]$tagRef.object.type) "commit" "Completed release tag must remain lightweight."
  Assert-Equal ([string]$tagRef.object.sha) ([string]$manifest.commit) "Completed release tag target mismatch."

  $installerPath = Join-Path $root ([string]$manifest.installer)
  $portablePath = Join-Path $root ([string]$manifest.portable)
  foreach ($artifact in @($installerPath, $portablePath)) {
    & gh attestation verify $artifact --repo $repository
    if ($LASTEXITCODE -ne 0) {
      throw "Remote completed release provenance verification failed for $artifact."
    }
  }

  $runId = [long]$certData.workflow.run_id
  $workflow = Invoke-GhJson @(
    "api",
    "repos/$repository/actions/runs/$runId"
  )

  Assert-Equal ([long]$workflow.id) $runId "Remote release workflow run ID mismatch."
  Assert-Equal ([string]$workflow.name) "Byte Release" "Remote release workflow name mismatch."
  Assert-Equal ([string]$workflow.event) "push" "Remote release workflow event mismatch."
  Assert-Equal ([string]$workflow.head_sha) ([string]$manifest.commit) "Remote release workflow source SHA mismatch."
  if (-not [string]::IsNullOrWhiteSpace([string]$workflow.head_branch) -and
      [string]$workflow.head_branch -ne $expectedTag) {
    throw "Remote release workflow belongs to '$($workflow.head_branch)', not '$expectedTag'."
  }
  Assert-Equal ([string]$workflow.html_url) ([string]$certData.workflow.url) "Remote release workflow URL mismatch."

  if ($RequireWorkflowSuccess) {
    if ($workflow.status -ne "completed" -or $workflow.conclusion -ne "success") {
      throw "Byte Release workflow has not completed successfully."
    }
  }
}

Write-Host "P7 public release certification verified for Byte $version."
Write-Host "Tag: $expectedTag"
Write-Host "Public distribution ready: True"
