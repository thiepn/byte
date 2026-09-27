param(
  [Parameter(Mandatory = $true)]
  [string]$PublishedDir,
  [Parameter(Mandatory = $true)]
  [string]$Tag,
  [Parameter(Mandatory = $true)]
  [string]$ReleaseNotesPath,
  [string]$Output = "public-release-certification.json"
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

Require-Command "gh"

$root = (Resolve-Path -LiteralPath $PublishedDir).Path
$notesPath = (Resolve-Path -LiteralPath $ReleaseNotesPath).Path
$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$expectedTag = "v$version"

if ($Tag -ne $expectedTag) {
  throw "Public release certification tag mismatch: expected '$expectedTag', got '$Tag'."
}
if (-not $env:GITHUB_SHA -or $env:GITHUB_SHA -notmatch '^[0-9a-fA-F]{40}$') {
  throw "Public release certification requires GITHUB_SHA from the tagged release workflow."
}
if (-not $env:GITHUB_RUN_ID) {
  throw "Public release certification requires GITHUB_RUN_ID."
}
if ($env:GITHUB_EVENT_NAME -ne "push") {
  throw "Public release certification must run from the tag push workflow."
}

$artifactVerifyArgs = @{
  OutputDir = $root
  RequireCertification = $true
  RequireSigning = $true
}
& (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") @artifactVerifyArgs

$manifestPath = Join-Path $root "release-manifest.json"
$productCertificationPath = Join-Path $root "product-certification.json"
$releaseCertificationPath = Join-Path $root "release-certification.json"
$checksumPath = Join-Path $root "SHA256SUMS.txt"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$releaseCertification = Get-Content $releaseCertificationPath -Raw | ConvertFrom-Json

if ([string]$manifest.commit -ne [string]$env:GITHUB_SHA) {
  throw "Fresh public release manifest commit does not match the tagged workflow source."
}
if ($releaseCertification.public_distribution_ready -ne $true) {
  throw "Fresh public release certification does not mark public_distribution_ready."
}

$installerPath = Join-Path $root ([string]$manifest.installer)
$portablePath = Join-Path $root ([string]$manifest.portable)

& (Join-Path $PSScriptRoot "test-portable-launch.ps1") -Portable $portablePath

$installerArgs = @{
  Installer = $installerPath
  RequireSigning = $true
}
& (Join-Path $PSScriptRoot "test-windows-installer.ps1") @installerArgs

foreach ($artifact in @($installerPath, $portablePath)) {
  & gh attestation verify $artifact --repo $repository
  if ($LASTEXITCODE -ne 0) {
    throw "GitHub provenance verification failed for freshly downloaded public artifact: $artifact"
  }
}

$release = Invoke-GhJson @(
  "release", "view", $Tag,
  "--repo", $repository,
  "--json", "databaseId,tagName,name,isDraft,isPrerelease,isLatest,publishedAt,url,targetCommitish,body,assets"
)

if ($null -eq $release) {
  throw "Could not load GitHub Release metadata for $Tag."
}
if ([string]$release.tagName -ne $Tag) {
  throw "Published release tag metadata mismatch."
}
if ([string]$release.name -ne ("Byte " + $version)) {
  throw "Published release title must be exactly 'Byte $version'."
}
if ($release.isDraft -ne $false) {
  throw "Published release must not be a draft."
}
if ([string]::IsNullOrWhiteSpace([string]$release.publishedAt)) {
  throw "Published release is missing publishedAt."
}

$isBeta = $version -match '-beta\.'
if ([bool]$release.isPrerelease -ne [bool]$isBeta) {
  throw "Published release prerelease state does not match the Byte version channel."
}
if (-not $isBeta -and $release.isLatest -ne $true) {
  throw "Stable Byte release must be marked as the latest release."
}

$tagRef = Invoke-GhJson @(
  "api",
  "repos/$repository/git/ref/tags/$Tag"
)
if ([string]$tagRef.ref -ne ("refs/tags/" + $Tag)) {
  throw "Remote release tag ref mismatch."
}
if ([string]$tagRef.object.type -ne "commit") {
  throw "P7 requires the P6 release tag to remain a lightweight direct commit ref."
}
if ([string]$tagRef.object.sha -ne [string]$env:GITHUB_SHA) {
  throw "Remote release tag does not point to the tagged workflow source commit."
}

$expectedAssets = @(
  [string]$manifest.installer,
  [string]$manifest.portable,
  "release-manifest.json",
  "product-certification.json",
  "release-certification.json",
  "SHA256SUMS.txt"
)

$localFiles = @(Get-ChildItem -LiteralPath $root -File | Select-Object -ExpandProperty Name | Sort-Object)
$localFileList = $localFiles -join [Environment]::NewLine
$expectedAssetList = ($expectedAssets | Sort-Object) -join [Environment]::NewLine
if ($localFileList -ne $expectedAssetList) {
  throw "Fresh public release download does not contain exactly the expected pre-P7 asset set."
}

$remoteAssetNames = @($release.assets | ForEach-Object { [string]$_.name } | Sort-Object)
$remoteAssetList = $remoteAssetNames -join [Environment]::NewLine
if ($remoteAssetList -ne $expectedAssetList) {
  throw "GitHub Release asset set does not exactly match the expected pre-P7 assets."
}

$assetEvidence = @()
foreach ($name in ($expectedAssets | Sort-Object)) {
  $path = Join-Path $root $name
  $remoteAsset = @($release.assets | Where-Object { [string]$_.name -eq $name })
  if ($remoteAsset.Count -ne 1) {
    throw "GitHub Release metadata does not contain exactly one asset named '$name'."
  }

  $sha256 = Digest $path
  $size = (Get-Item -LiteralPath $path).Length

  if ($null -ne $remoteAsset[0].size -and [long]$remoteAsset[0].size -ne [long]$size) {
    throw "Remote release asset size mismatch for '$name'."
  }
  if (-not [string]::IsNullOrWhiteSpace([string]$remoteAsset[0].digest)) {
    $expectedDigest = "sha256:" + $sha256
    if ([string]$remoteAsset[0].digest -ne $expectedDigest) {
      throw "Remote release asset digest mismatch for '$name'."
    }
  }

  $assetEvidence += [ordered]@{
    name = $name
    size_bytes = [long]$size
    sha256 = $sha256
    remote_digest = [string]$remoteAsset[0].digest
  }
}

$expectedNotes = Get-Content -LiteralPath $notesPath -Raw
$expectedNotesHash = TextDigest $expectedNotes
$publishedNotesHash = TextDigest ([string]$release.body)
if ($expectedNotesHash -ne $publishedNotesHash) {
  throw "Published release notes do not match the curated changelog-derived release notes."
}

$workflow = Invoke-GhJson @(
  "run", "view", ([string]$env:GITHUB_RUN_ID),
  "--repo", $repository,
  "--json", "databaseId,workflowName,event,headSha,headBranch,status,conclusion,url"
)

if ([string]$workflow.workflowName -ne "Byte Release") {
  throw "P7 must run inside the Byte Release workflow."
}
if ([string]$workflow.event -ne "push") {
  throw "Byte Release workflow event must be push."
}
if ([string]$workflow.headSha -ne [string]$env:GITHUB_SHA) {
  throw "Byte Release workflow source SHA mismatch."
}
if (-not [string]::IsNullOrWhiteSpace([string]$workflow.headBranch) -and
    [string]$workflow.headBranch -ne $Tag) {
  throw "Byte Release workflow head branch/tag mismatch."
}

$outputPath = [IO.Path]::GetFullPath($Output)
$outputParent = Split-Path $outputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outputParent)) {
  New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
}

$publicCertification = [ordered]@{
  schema_version = 1
  product = "Byte"
  version = $version
  tag = $Tag
  source_commit = [string]$env:GITHUB_SHA
  certified_at_utc = [DateTime]::UtcNow.ToString("o")
  certification_scope = "fresh-public-download-post-publish"
  channel = if ($isBeta) { "beta" } else { "stable" }
  release = [ordered]@{
    database_id = [long]$release.databaseId
    tag_name = [string]$release.tagName
    name = [string]$release.name
    url = [string]$release.url
    draft = [bool]$release.isDraft
    prerelease = [bool]$release.isPrerelease
    latest = [bool]$release.isLatest
    published_at = [string]$release.publishedAt
    target_commitish = [string]$release.targetCommitish
    body_sha256 = $publishedNotesHash
  }
  workflow = [ordered]@{
    run_id = [long]$workflow.databaseId
    run_number = if ($env:GITHUB_RUN_NUMBER) { [long]$env:GITHUB_RUN_NUMBER } else { $null }
    run_attempt = if ($env:GITHUB_RUN_ATTEMPT) { [long]$env:GITHUB_RUN_ATTEMPT } else { $null }
    name = [string]$workflow.workflowName
    event = [string]$workflow.event
    head_sha = [string]$workflow.headSha
    head_branch = [string]$workflow.headBranch
    url = [string]$workflow.url
    observed_status = [string]$workflow.status
    observed_conclusion = [string]$workflow.conclusion
  }
  tag_identity = [ordered]@{
    type = "lightweight"
    ref = [string]$tagRef.ref
    target_commit = [string]$tagRef.object.sha
  }
  checks = [ordered]@{
    exact_asset_set = $true
    sha256_and_manifest = $true
    authenticode_and_timestamp = $true
    same_signer = $true
    release_certification_public_distribution_ready = $true
    github_provenance_installer = $true
    github_provenance_portable = $true
    public_portable_launch = $true
    public_installer_lifecycle = $true
    release_title_and_tag = $true
    release_notes_match = $true
    channel_metadata = $true
    exact_lightweight_tag_target = $true
  }
  assets = $assetEvidence
  release_manifest_sha256 = Digest $manifestPath
  product_certification_sha256 = Digest $productCertificationPath
  release_certification_sha256 = Digest $releaseCertificationPath
  checksum_file_sha256 = Digest $checksumPath
  signing = [ordered]@{
    signed = [bool]$manifest.signed
    timestamped = [bool]$manifest.signing.timestamped
    same_signer = [bool]$manifest.signing.same_signer
    application_signer_thumbprint = [string]$manifest.signing.application.signer_thumbprint
    installer_signer_thumbprint = [string]$manifest.signing.installer.signer_thumbprint
  }
  public_distribution_ready = $true
  receipt_self_hashed = $false
}

$publicCertification | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $outputPath -Encoding utf8

Write-Host "P7 public release certification written for Byte $version."
Write-Host "Tag: $Tag"
Write-Host "Release: $($release.url)"
Write-Host "Fresh public installer/portable launch, signing, checksum, provenance, and metadata checks passed."
