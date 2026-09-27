param(
  [Parameter(Mandatory = $true)]
  [string]$PublishedDir,
  [Parameter(Mandatory = $true)]
  [string]$PublicCertification,
  [Parameter(Mandatory = $true)]
  [string]$Initiation,
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [Parameter(Mandatory = $true)]
  [string]$DeviceReport,
  [Parameter(Mandatory = $true)]
  [string]$Approval,
  [Parameter(Mandatory = $true)]
  [string]$Completion,
  [switch]$RequireRemoteState
)

$ErrorActionPreference = "Stop"

function Digest([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-Equal([object]$Actual, [object]$Expected, [string]$Message) {
  if ($Actual -ne $Expected) {
    throw "$Message Expected '$Expected', got '$Actual'."
  }
}

$publishedRoot = (Resolve-Path -LiteralPath $PublishedDir).Path
$publicCertPath = (Resolve-Path -LiteralPath $PublicCertification).Path
$initiationPath = (Resolve-Path -LiteralPath $Initiation).Path
$candidatePath = (Resolve-Path -LiteralPath $CandidateDir).Path
$deviceReportPath = (Resolve-Path -LiteralPath $DeviceReport).Path
$approvalPath = (Resolve-Path -LiteralPath $Approval).Path
$completionPath = (Resolve-Path -LiteralPath $Completion).Path

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$expectedTag = "v$version"

$completionData = Get-Content -LiteralPath $completionPath -Raw | ConvertFrom-Json
$publicCert = Get-Content -LiteralPath $publicCertPath -Raw | ConvertFrom-Json
$initiationData = Get-Content -LiteralPath $initiationPath -Raw | ConvertFrom-Json

$initiationVerifyArgs = @{
  CandidateDir = $candidatePath
  DeviceReport = $deviceReportPath
  Approval = $approvalPath
  Initiation = $initiationPath
}
if ($RequireRemoteState) {
  $initiationVerifyArgs.RequireRemoteState = $true
  $initiationVerifyArgs.RequireWorkflowSuccess = $true
}
& (Join-Path $PSScriptRoot "verify-release-initiation.ps1") @initiationVerifyArgs

$publicVerifyArgs = @{
  PublishedDir = $publishedRoot
  Certification = $publicCertPath
}
if ($RequireRemoteState) {
  $publicVerifyArgs.RequireRemoteState = $true
  $publicVerifyArgs.RequireWorkflowSuccess = $true
}
& (Join-Path $PSScriptRoot "verify-public-release-certification.ps1") @publicVerifyArgs

Assert-Equal $completionData.schema_version 1 "Release completion schema mismatch."
Assert-Equal $completionData.product "Byte" "Release completion product mismatch."
Assert-Equal ([string]$completionData.version) $version "Release completion version mismatch."
Assert-Equal ([string]$completionData.tag) $expectedTag "Release completion tag mismatch."
Assert-Equal ([string]$completionData.completion_scope) "post-publish-release-closure" "Release completion scope mismatch."

Assert-Equal ([string]$completionData.source.repository) "thiepn/byte" "Release completion repository mismatch."
Assert-Equal ([string]$completionData.source.commit) ([string]$initiationData.source.commit) "Release completion source commit mismatch."
Assert-Equal ([string]$completionData.source.local_head) ([string]$initiationData.source.commit) "Release completion local-head mismatch."

Assert-Equal ([string]$completionData.initiation.file) ([IO.Path]::GetFileName($initiationPath)) "Release completion initiation filename mismatch."
Assert-Equal ([string]$completionData.initiation.sha256) (Digest $initiationPath) "Release completion initiation hash mismatch."
Assert-Equal ([long]$completionData.initiation.workflow_run_id) ([long]$initiationData.release_workflow.run_id) "Release completion workflow-run binding mismatch."

Assert-Equal ([string]$completionData.approval.file) ([IO.Path]::GetFileName($approvalPath)) "Release completion approval filename mismatch."
Assert-Equal ([string]$completionData.approval.sha256) (Digest $approvalPath) "Release completion approval hash mismatch."

Assert-Equal ([string]$completionData.physical_device_report.file) ([IO.Path]::GetFileName($deviceReportPath)) "Release completion device-report filename mismatch."
Assert-Equal ([string]$completionData.physical_device_report.sha256) (Digest $deviceReportPath) "Release completion device-report hash mismatch."

Assert-Equal ([string]$completionData.public_release_certification.file) ([IO.Path]::GetFileName($publicCertPath)) "Release completion public-certification filename mismatch."
Assert-Equal ([string]$completionData.public_release_certification.sha256) (Digest $publicCertPath) "Release completion public-certification hash mismatch."
Assert-Equal ([int]$completionData.public_release_certification.schema_version) ([int]$publicCert.schema_version) "Release completion public-certification schema mismatch."

Assert-Equal ([long]$completionData.release.database_id) ([long]$publicCert.release.database_id) "Release completion release database ID mismatch."
Assert-Equal ([string]$completionData.release.tag_name) ([string]$publicCert.release.tag_name) "Release completion release tag mismatch."
Assert-Equal ([string]$completionData.release.name) ([string]$publicCert.release.name) "Release completion release name mismatch."
Assert-Equal ([string]$completionData.release.url) ([string]$publicCert.release.url) "Release completion release URL mismatch."
Assert-Equal ([bool]$completionData.release.draft) ([bool]$publicCert.release.draft) "Release completion draft state mismatch."
Assert-Equal ([bool]$completionData.release.prerelease) ([bool]$publicCert.release.prerelease) "Release completion prerelease state mismatch."
Assert-Equal ([bool]$completionData.release.latest) ([bool]$publicCert.release.latest) "Release completion latest-release state mismatch."
Assert-Equal ([string]$completionData.release.published_at) ([string]$publicCert.release.published_at) "Release completion publication timestamp mismatch."

Assert-Equal ([long]$completionData.workflow.run_id) ([long]$publicCert.workflow.run_id) "Release completion workflow run ID mismatch."
Assert-Equal ([string]$completionData.workflow.url) ([string]$publicCert.workflow.url) "Release completion workflow URL mismatch."
Assert-Equal ([string]$completionData.workflow.name) "Byte Release" "Release completion workflow name mismatch."
Assert-Equal ([string]$completionData.workflow.event) "push" "Release completion workflow event mismatch."
Assert-Equal ([string]$completionData.workflow.head_sha) ([string]$publicCert.source_commit) "Release completion workflow source SHA mismatch."
if (-not [string]::IsNullOrWhiteSpace([string]$completionData.workflow.head_branch) -and
    [string]$completionData.workflow.head_branch -ne $expectedTag) {
  throw "Release completion workflow head branch/tag mismatch."
}
if ($completionData.workflow.status -ne "completed" -or
    $completionData.workflow.conclusion -ne "success") {
  throw "Release completion receipt does not record a successful completed Byte Release workflow."
}

$requiredChecks = @(
  "p6_initiation_verified",
  "release_workflow_success",
  "final_public_download_complete",
  "public_release_certification_verified",
  "public_asset_hashes_verified",
  "authenticode_verified",
  "github_provenance_verified",
  "public_portable_launch_rechecked",
  "public_installer_lifecycle_rechecked",
  "release_metadata_verified",
  "stable_beta_metadata_verified",
  "exact_tag_target_verified"
)
foreach ($check in $requiredChecks) {
  if ($completionData.checks.$check -ne $true) {
    throw "Release completion check '$check' is not green."
  }
}

if ($completionData.tag_mutation_performed -ne $false) {
  throw "P7 completion must not mutate the release tag."
}
if ($completionData.release_mutation_performed -ne $false) {
  throw "P7 completion must not mutate the GitHub Release."
}
if ($completionData.asset_upload_performed -ne $false) {
  throw "P7 completion must not upload release assets."
}
if ($completionData.complete -ne $true) {
  throw "Release completion receipt does not mark the release complete."
}
if ($completionData.receipt_self_hashed -ne $false) {
  throw "Release completion receipt must not claim to hash itself."
}

$localFiles = @(Get-ChildItem -LiteralPath $publishedRoot -File | Sort-Object Name)
$completionAssets = @($completionData.assets | Sort-Object name)
if ($completionAssets.Count -ne $localFiles.Count) {
  throw "Release completion asset inventory count mismatch."
}

for ($i = 0; $i -lt $localFiles.Count; $i++) {
  $file = $localFiles[$i]
  $entry = $completionAssets[$i]
  Assert-Equal ([string]$entry.name) $file.Name "Release completion asset name mismatch."
  Assert-Equal ([long]$entry.size_bytes) ([long]$file.Length) "Release completion asset size mismatch for $($file.Name)."
  Assert-Equal ([string]$entry.sha256) (Digest $file.FullName) "Release completion asset hash mismatch for $($file.Name)."
}

if ($RequireRemoteState) {
  $publicVerifyRemote = @{
    PublishedDir = $publishedRoot
    Certification = $publicCertPath
    RequireRemoteState = $true
    RequireWorkflowSuccess = $true
  }
  & (Join-Path $PSScriptRoot "verify-public-release-certification.ps1") @publicVerifyRemote

  $initiationVerifyRemote = @{
    CandidateDir = $candidatePath
    DeviceReport = $deviceReportPath
    Approval = $approvalPath
    Initiation = $initiationPath
    RequireRemoteState = $true
    RequireWorkflowSuccess = $true
  }
  & (Join-Path $PSScriptRoot "verify-release-initiation.ps1") @initiationVerifyRemote
}

Write-Host "P7 release completion receipt verified for Byte $version."
Write-Host "Tag: $expectedTag"
Write-Host "Release closure complete: True"
