param(
  [Parameter(Mandatory = $true)]
  [string]$Initiation,
  [string]$CandidateDir = "",
  [string]$DeviceReport = "",
  [string]$Approval = "",
  [string]$DownloadDir = "",
  [string]$Output = ""
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

function Get-LocalHeadSha {
  $output = & git rev-parse HEAD
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine local Git HEAD."
  }
  return ($output | Out-String).Trim()
}

function Get-LocalOrigin {
  $output = & git remote get-url origin
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine local origin remote."
  }
  return ($output | Out-String).Trim()
}

function Assert-CleanWorkingTree {
  $output = & git status --porcelain
  if ($LASTEXITCODE -ne 0) {
    throw "Could not inspect local Git working tree."
  }
  if (-not [string]::IsNullOrWhiteSpace(($output | Out-String).Trim())) {
    throw "The working tree is not clean. P7 release completion requires an unchanged release-source checkout."
  }
}

function DocumentsBytePath([string]$Name) {
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path (Join-Path $documents "Byte") $Name
}

function Resolve-DefaultCandidate([string]$Commit) {
  return DocumentsBytePath ("ReleaseApproval\candidate-signed-" + $Commit)
}

function Resolve-DefaultCompletionDownload([string]$Version) {
  return DocumentsBytePath ("ReleaseCompletion\public-v" + $Version)
}

Require-Command "gh"
Require-Command "git"

& gh auth status
if ($LASTEXITCODE -ne 0) {
  throw "GitHub CLI authentication is required for P7 release completion."
}

$initiationPath = (Resolve-Path -LiteralPath $Initiation).Path
$initiationData = Get-Content -LiteralPath $initiationPath -Raw | ConvertFrom-Json

if ($initiationData.schema_version -ne 1 -or
    $initiationData.product -ne "Byte" -or
    $initiationData.initiation_scope -ne "controlled-tag-and-release-workflow-start") {
  throw "P7 requires a valid P6 release-initiation receipt."
}
if ($initiationData.release_workflow.detected -ne $true) {
  throw "P7 requires a P6 initiation receipt with a detected Byte Release workflow."
}

$sourceCommit = [string]$initiationData.source.commit
$tag = [string]$initiationData.tag.name
$version = [string]$initiationData.version

if ($sourceCommit -notmatch '^[0-9a-fA-F]{40}$') {
  throw "P6 initiation source commit is invalid."
}
if ($tag -ne ("v" + $version)) {
  throw "P6 initiation tag/version identity is inconsistent."
}

$localHead = Get-LocalHeadSha
$localOrigin = Get-LocalOrigin
Assert-CleanWorkingTree

if ($localOrigin -notmatch '(^|[:/])thiepn/byte(?:\.git)?$') {
  throw "P7 must run from the thiepn/byte repository. origin is '$localOrigin'."
}
if ($localHead.ToLowerInvariant() -ne $sourceCommit.ToLowerInvariant()) {
  throw "P7 must run from the exact released source commit $sourceCommit. Current HEAD is $localHead."
}

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
if ([string]$config.version -ne $version) {
  throw "Local release-source version does not match the P6 initiation receipt."
}

if ([string]::IsNullOrWhiteSpace($CandidateDir)) {
  $CandidateDir = Resolve-DefaultCandidate $sourceCommit
}
$candidatePath = (Resolve-Path -LiteralPath $CandidateDir).Path

if ([string]::IsNullOrWhiteSpace($DeviceReport)) {
  $DeviceReport = DocumentsBytePath ([string]$initiationData.physical_device_report.file)
}
$deviceReportPath = (Resolve-Path -LiteralPath $DeviceReport).Path

if ([string]::IsNullOrWhiteSpace($Approval)) {
  $Approval = DocumentsBytePath ([string]$initiationData.approval.file)
}
$approvalPath = (Resolve-Path -LiteralPath $Approval).Path

$initiationVerifyArgs = @{
  CandidateDir = $candidatePath
  DeviceReport = $deviceReportPath
  Approval = $approvalPath
  Initiation = $initiationPath
  RequireRemoteState = $true
  RequireWorkflowSuccess = $true
}
& (Join-Path $PSScriptRoot "verify-release-initiation.ps1") @initiationVerifyArgs

$workflowRunId = [long]$initiationData.release_workflow.run_id
$workflow = Invoke-GhJson @(
  "api",
  "repos/$repository/actions/runs/$workflowRunId"
)

if ($workflow.status -ne "completed" -or $workflow.conclusion -ne "success") {
  throw "Byte Release workflow has not completed successfully."
}

if ([string]::IsNullOrWhiteSpace($DownloadDir)) {
  $DownloadDir = Resolve-DefaultCompletionDownload $version
}
$publishedDir = [IO.Path]::GetFullPath($DownloadDir)
if (Test-Path -LiteralPath $publishedDir) {
  Remove-Item -LiteralPath $publishedDir -Recurse -Force
}
New-Item -ItemType Directory -Path $publishedDir -Force | Out-Null

Write-Host "Downloading final public release $tag..." -ForegroundColor Cyan
& gh release download $tag --repo $repository --dir $publishedDir
if ($LASTEXITCODE -ne 0) {
  throw "Could not download the final public release $tag."
}

$publicCertPath = Join-Path $publishedDir "public-release-certification.json"
if (!(Test-Path -LiteralPath $publicCertPath)) {
  throw "Completed release is missing public-release-certification.json."
}

$publicVerifyArgs = @{
  PublishedDir = $publishedDir
  Certification = $publicCertPath
  RequireRemoteState = $true
  RequireWorkflowSuccess = $true
  RecheckRuntime = $true
}
& (Join-Path $PSScriptRoot "verify-public-release-certification.ps1") @publicVerifyArgs

$publicCert = Get-Content -LiteralPath $publicCertPath -Raw | ConvertFrom-Json

if ([long]$publicCert.workflow.run_id -ne $workflowRunId) {
  throw "Public release certification is bound to a different Byte Release workflow run."
}
if ([string]$publicCert.source_commit -ne $sourceCommit) {
  throw "Public release certification source commit does not match P6 initiation."
}
if ([string]$publicCert.tag -ne $tag) {
  throw "Public release certification tag does not match P6 initiation."
}

$release = Invoke-GhJson @(
  "api",
  "repos/$repository/releases/tags/$tag"
)

$outputPath = if ([string]::IsNullOrWhiteSpace($Output)) {
  DocumentsBytePath ("release-completion-v" + $version + ".json")
} else {
  [IO.Path]::GetFullPath($Output)
}
$outputParent = Split-Path $outputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outputParent)) {
  New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
}

$assetEvidence = @()
foreach ($file in @(Get-ChildItem -LiteralPath $publishedDir -File | Sort-Object Name)) {
  $assetEvidence += [ordered]@{
    name = $file.Name
    size_bytes = [long]$file.Length
    sha256 = Digest $file.FullName
  }
}

$completion = [ordered]@{
  schema_version = 1
  product = "Byte"
  version = $version
  tag = $tag
  completed_at_utc = [DateTime]::UtcNow.ToString("o")
  completion_scope = "post-publish-release-closure"
  source = [ordered]@{
    repository = $repository
    commit = $sourceCommit
    local_head = $localHead
  }
  initiation = [ordered]@{
    file = [IO.Path]::GetFileName($initiationPath)
    sha256 = Digest $initiationPath
    workflow_run_id = $workflowRunId
  }
  approval = [ordered]@{
    file = [IO.Path]::GetFileName($approvalPath)
    sha256 = Digest $approvalPath
  }
  physical_device_report = [ordered]@{
    file = [IO.Path]::GetFileName($deviceReportPath)
    sha256 = Digest $deviceReportPath
  }
  public_release_certification = [ordered]@{
    file = [IO.Path]::GetFileName($publicCertPath)
    sha256 = Digest $publicCertPath
    schema_version = [int]$publicCert.schema_version
  }
  release = [ordered]@{
    database_id = [long]$release.id
    tag_name = [string]$release.tag_name
    name = [string]$release.name
    url = [string]$release.html_url
    draft = [bool]$release.draft
    prerelease = [bool]$release.prerelease
    latest = [bool]$publicCert.release.latest
    published_at = [string]$release.published_at
  }
  workflow = [ordered]@{
    run_id = [long]$workflow.id
    url = [string]$workflow.html_url
    name = [string]$workflow.name
    event = [string]$workflow.event
    head_sha = [string]$workflow.head_sha
    head_branch = [string]$workflow.head_branch
    status = [string]$workflow.status
    conclusion = [string]$workflow.conclusion
  }
  assets = $assetEvidence
  checks = [ordered]@{
    p6_initiation_verified = $true
    release_workflow_success = $true
    final_public_download_complete = $true
    public_release_certification_verified = $true
    public_asset_hashes_verified = $true
    authenticode_verified = $true
    github_provenance_verified = $true
    public_portable_launch_rechecked = $true
    public_installer_lifecycle_rechecked = $true
    release_metadata_verified = $true
    stable_beta_metadata_verified = $true
    exact_tag_target_verified = $true
  }
  tag_mutation_performed = $false
  release_mutation_performed = $false
  asset_upload_performed = $false
  complete = $true
  receipt_self_hashed = $false
}

$completion | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $outputPath -Encoding utf8

$completionVerifyArgs = @{
  PublishedDir = $publishedDir
  PublicCertification = $publicCertPath
  Initiation = $initiationPath
  CandidateDir = $candidatePath
  DeviceReport = $deviceReportPath
  Approval = $approvalPath
  Completion = $outputPath
  RequireRemoteState = $true
}
& (Join-Path $PSScriptRoot "verify-release-completion.ps1") @completionVerifyArgs

Write-Host ""
Write-Host "P7 release completion certified." -ForegroundColor Green
Write-Host "  Tag: $tag"
Write-Host "  Commit: $sourceCommit"
Write-Host "  Byte Release run: $workflowRunId"
Write-Host "  Public release: $($release.html_url)"
Write-Host "  Completion receipt: $outputPath"
Write-Host ""
Write-Host "P7 performed no tag, release, or asset mutation."
