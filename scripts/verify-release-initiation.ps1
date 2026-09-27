param(
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [Parameter(Mandatory = $true)]
  [string]$DeviceReport,
  [Parameter(Mandatory = $true)]
  [string]$Approval,
  [Parameter(Mandatory = $true)]
  [string]$Initiation,
  [switch]$RequireRemoteState,
  [switch]$RequireWorkflowSuccess
)

$ErrorActionPreference = "Stop"

$repository = "thiepn/byte"

function Digest([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-Equal([object]$Actual, [object]$Expected, [string]$Message) {
  if ($Actual -ne $Expected) {
    throw "$Message Expected '$Expected', got '$Actual'."
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

$root = (Resolve-Path -LiteralPath $CandidateDir).Path
$deviceReportPath = (Resolve-Path -LiteralPath $DeviceReport).Path
$approvalPath = (Resolve-Path -LiteralPath $Approval).Path
$initiationPath = (Resolve-Path -LiteralPath $Initiation).Path

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$expectedTag = "v$version"

$manifestPath = Join-Path $root "release-manifest.json"
$releaseCertificationPath = Join-Path $root "release-certification.json"

foreach ($required in @($manifestPath, $releaseCertificationPath)) {
  if (!(Test-Path $required)) {
    throw "P6 verification is missing candidate evidence: $required"
  }
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$approvalData = Get-Content -LiteralPath $approvalPath -Raw | ConvertFrom-Json
$receipt = Get-Content -LiteralPath $initiationPath -Raw | ConvertFrom-Json

$approvalVerifyArgs = @{
  CandidateDir = $root
  DeviceReport = $deviceReportPath
  Approval = $approvalPath
  RequireTaggingApproval = $true
}
& (Join-Path $PSScriptRoot "verify-release-approval.ps1") @approvalVerifyArgs

Assert-Equal $receipt.schema_version 1 "Release initiation schema mismatch."
Assert-Equal $receipt.product "Byte" "Release initiation product mismatch."
Assert-Equal ([string]$receipt.version) $version "Release initiation version mismatch."
Assert-Equal ([string]$receipt.initiation_scope) "controlled-tag-and-release-workflow-start" "Release initiation scope mismatch."

Assert-Equal ([string]$receipt.source.repository) $repository "Release initiation repository mismatch."
Assert-Equal ([string]$receipt.source.branch) "main" "Release initiation source branch mismatch."
Assert-Equal ([string]$receipt.source.commit) ([string]$manifest.commit) "Release initiation source commit mismatch."
Assert-Equal ([string]$receipt.source.local_head) ([string]$manifest.commit) "Release initiation local-head mismatch."

Assert-Equal ([string]$receipt.approval.file) ([IO.Path]::GetFileName($approvalPath)) "Release initiation approval filename mismatch."
Assert-Equal ([string]$receipt.approval.sha256) (Digest $approvalPath) "Release initiation approval hash mismatch."
Assert-Equal ([bool]$receipt.approval.approved_for_tagging) $true "Release initiation must preserve approved_for_tagging."
Assert-Equal ([bool]$receipt.approval.github_attestation_verified) $true "Release initiation must preserve candidate provenance verification."

Assert-Equal ([string]$receipt.physical_device_report.file) ([IO.Path]::GetFileName($deviceReportPath)) "Release initiation device-report filename mismatch."
Assert-Equal ([string]$receipt.physical_device_report.sha256) (Digest $deviceReportPath) "Release initiation device-report hash mismatch."
Assert-Equal ([string]$receipt.candidate.release_manifest_sha256) (Digest $manifestPath) "Release initiation release-manifest hash mismatch."
Assert-Equal ([string]$receipt.candidate.release_certification_sha256) (Digest $releaseCertificationPath) "Release initiation release-certification hash mismatch."
Assert-Equal ([bool]$receipt.candidate.github_attestation_reverified) $true "Release initiation must record fresh GitHub provenance verification."

if ($receipt.preflight.exact_remote_main -ne $true) {
  throw "Release initiation receipt does not record exact-main preflight success."
}
if ($receipt.preflight.clean_working_tree -ne $true) {
  throw "Release initiation receipt does not record a clean working tree."
}
if ($receipt.preflight.pre_tag_main_reverified -ne $true) {
  throw "Release initiation receipt does not record immediate pre-tag exact-main revalidation."
}
if ($receipt.preflight.release_order_verified -ne $true) {
  throw "Release initiation receipt does not record release-order verification."
}
if ($receipt.preflight.ci.conclusion -ne "success" -or
    $receipt.preflight.packaging.conclusion -ne "success") {
  throw "Release initiation receipt does not contain successful exact-main CI and Packaging evidence."
}

Assert-Equal ([string]$receipt.tag.name) $expectedTag "Release initiation tag name mismatch."
Assert-Equal ([string]$receipt.tag.ref) ("refs/tags/" + $expectedTag) "Release initiation tag ref mismatch."
Assert-Equal ([string]$receipt.tag.type) "lightweight" "P6 tags must be lightweight exact-commit tags."
Assert-Equal ([string]$receipt.tag.target_commit) ([string]$manifest.commit).ToLowerInvariant() "Release initiation tag target mismatch."
if ($receipt.tag.forced_update -ne $false) {
  throw "P6 must never force-update a tag."
}

$created = [bool]$receipt.tag.created_by_this_run
$resumed = [bool]$receipt.tag.resumed_existing_tag
if ($created -eq $resumed) {
  throw "Release initiation receipt must record exactly one of created_by_this_run or resumed_existing_tag."
}
if ($created -and $receipt.preflight.tag_and_release_name_available_before_creation -ne $true) {
  throw "A newly created P6 tag must record pre-creation name availability."
}
if ($resumed -and $receipt.preflight.tag_and_release_name_available_before_creation -ne $false) {
  throw "A resumed P6 tag must not claim pre-creation name availability."
}

if ($receipt.direct_release_publish_performed -ne $false) {
  throw "P6 must not directly publish a GitHub Release."
}
if ($receipt.direct_asset_upload_performed -ne $false) {
  throw "P6 must not directly upload release assets."
}

if ($receipt.release_workflow.detected -ne $true) {
  throw "Release initiation receipt does not contain a detected Byte Release workflow."
}
Assert-Equal ([string]$receipt.release_workflow.workflow) "Byte Release" "Release initiation workflow name mismatch."
Assert-Equal ([string]$receipt.release_workflow.workflow_file) "release.yml" "Release initiation workflow file mismatch."
Assert-Equal ([string]$receipt.release_workflow.event) "push" "Release initiation workflow event mismatch."
Assert-Equal ([string]$receipt.release_workflow.head_sha) ([string]$manifest.commit) "Release initiation workflow head SHA mismatch."
if ($null -eq $receipt.release_workflow.run_id) {
  throw "Release initiation receipt is missing the Byte Release workflow run ID."
}

if ($RequireRemoteState -or $RequireWorkflowSuccess) {
  if ($null -eq (Get-Command "gh" -ErrorAction SilentlyContinue)) {
    throw "GitHub CLI is required for remote P6 verification."
  }

  & gh auth status
  if ($LASTEXITCODE -ne 0) {
    throw "GitHub CLI authentication is required for remote P6 verification."
  }

  $tagRef = Invoke-GhJson @(
    "api",
    "repos/$repository/git/ref/tags/$expectedTag"
  )

  Assert-Equal ([string]$tagRef.ref) ("refs/tags/" + $expectedTag) "Remote tag ref mismatch."
  Assert-Equal ([string]$tagRef.object.type) "commit" "P6 remote tag must be lightweight."
  Assert-Equal ([string]$tagRef.object.sha) ([string]$manifest.commit) "Remote tag target mismatch."

  foreach ($name in @([string]$manifest.installer, [string]$manifest.portable)) {
    $artifact = Join-Path $root $name
    & gh attestation verify $artifact --repo $repository
    if ($LASTEXITCODE -ne 0) {
      throw "Remote P6 verification could not validate GitHub provenance for $artifact."
    }
  }

  $runId = [long]$receipt.release_workflow.run_id
  $run = Invoke-GhJson @(
    "run", "view", ([string]$runId),
    "--repo", $repository,
    "--json", "databaseId,workflowName,event,headSha,headBranch,status,conclusion,url"
  )

  Assert-Equal ([long]$run.databaseId) $runId "Remote Byte Release run ID mismatch."
  Assert-Equal ([string]$run.workflowName) "Byte Release" "Remote workflow name mismatch."
  Assert-Equal ([string]$run.event) "push" "Remote workflow event mismatch."
  Assert-Equal ([string]$run.headSha) ([string]$manifest.commit) "Remote workflow head SHA mismatch."
  if (-not [string]::IsNullOrWhiteSpace([string]$run.headBranch) -and
      [string]$run.headBranch -ne $expectedTag) {
    throw "Remote Byte Release run belongs to '$($run.headBranch)', not '$expectedTag'."
  }

  Assert-Equal ([string]$receipt.release_workflow.url) ([string]$run.url) "Release initiation workflow URL mismatch."

  if ($RequireWorkflowSuccess) {
    if ($run.status -ne "completed" -or $run.conclusion -ne "success") {
      throw "Byte Release workflow has not completed successfully."
    }
  }
}

Write-Host "P6 release initiation receipt verified for Byte $version."
Write-Host "Tag: $expectedTag -> $($manifest.commit)"
Write-Host "Byte Release run: $($receipt.release_workflow.run_id)"
Write-Host "Recorded workflow state: $($receipt.release_workflow.status) $($receipt.release_workflow.conclusion)"
