param(
  [ValidateSet("signed", "unsigned")]
  [string]$CandidateType = "signed",
  [string]$CandidateDir = "",
  [string]$Report = "",
  [string]$ApprovalOutput = "",
  [string]$DownloadRoot = "",
  [int]$PerformanceMinutes = 10,
  [switch]$ReuseExistingCandidate,
  [switch]$VerifyOnly
)

$ErrorActionPreference = "Stop"

$repository = "thiepn/byte"
$deviceWorkflow = "device-signoff-candidate.yml"
$packageWorkflow = "package.yml"

function Require-Command([string]$Name) {
  if ($null -eq (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Required command '$Name' is not available on PATH."
  }
}

function Invoke-Gh([string[]]$Arguments) {
  & gh @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "GitHub CLI command failed: gh $($Arguments -join ' ')"
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

function Get-RemoteMainSha {
  $result = Invoke-GhJson @("api", "repos/$repository/commits/main")
  return [string]$result.sha
}

function Get-LocalHeadSha {
  $output = & git rev-parse HEAD
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine the local Git HEAD."
  }
  $sha = ($output | Out-String).Trim()
  if ([string]::IsNullOrWhiteSpace($sha)) {
    throw "Local Git HEAD is empty."
  }
  return $sha
}

function Get-LocalOrigin {
  $output = & git remote get-url origin
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine the local origin remote."
  }
  return ($output | Out-String).Trim()
}

function Get-LocalBranch {
  $output = & git branch --show-current
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine the local Git branch."
  }
  return ($output | Out-String).Trim()
}

function Assert-CleanWorkingTree {
  $output = & git status --porcelain
  if ($LASTEXITCODE -ne 0) {
    throw "Could not inspect the local Git working tree."
  }
  $status = ($output | Out-String).Trim()
  if (-not [string]::IsNullOrWhiteSpace($status)) {
    throw "The working tree is not clean. Commit or stash local changes before release approval."
  }
}

function Get-WorkflowFile([string]$Type) {
  if ($Type -eq "signed") {
    return $deviceWorkflow
  }
  return $packageWorkflow
}

function Get-CandidateArtifactName([string]$Type, [string]$Sha) {
  if ($Type -eq "signed") {
    return "byte-device-signoff-candidate-$Sha"
  }
  return "byte-release-candidate-$Sha"
}

function Get-WorkflowRuns([string]$Workflow, [string]$Sha) {
  $arguments = @(
    "run", "list",
    "--repo", $repository,
    "--workflow", $Workflow,
    "--branch", "main",
    "--limit", "30",
    "--json", "databaseId,headSha,status,conclusion,event,createdAt,url"
  )
  $runs = Invoke-GhJson $arguments
  if ($null -eq $runs) {
    return @()
  }
  return @($runs | Where-Object { $_.headSha -eq $Sha })
}

function Find-SuccessfulRun([string]$Workflow, [string]$Sha) {
  $runs = @(Get-WorkflowRuns $Workflow $Sha | Where-Object {
    $_.status -eq "completed" -and $_.conclusion -eq "success"
  } | Sort-Object createdAt -Descending)

  if ($runs.Count -eq 0) {
    return $null
  }
  return $runs[0]
}

function Wait-ForDispatchedRun(
  [string]$Workflow,
  [string]$Sha,
  [long[]]$KnownRunIds
) {
  $deadline = [DateTime]::UtcNow.AddMinutes(3)

  while ([DateTime]::UtcNow -lt $deadline) {
    $runs = @(Get-WorkflowRuns $Workflow $Sha | Where-Object {
      $_.event -eq "workflow_dispatch" -and
      $KnownRunIds -notcontains [long]$_.databaseId
    } | Sort-Object createdAt -Descending)

    if ($runs.Count -gt 1) {
      throw "More than one new workflow_dispatch run appeared for exact main; refusing to guess which signed candidate belongs to this approval session."
    }
    if ($runs.Count -eq 1) {
      return $runs[0]
    }

    Start-Sleep -Seconds 3
  }

  throw "Timed out waiting for the manually dispatched $Workflow run to appear."
}

function Resolve-OrCreateCandidateRun([string]$Type, [string]$Sha) {
  $workflow = Get-WorkflowFile $Type

  if ($ReuseExistingCandidate) {
    $existing = Find-SuccessfulRun $workflow $Sha
    if ($null -eq $existing) {
      throw "No successful existing $Type candidate workflow run was found for exact main commit $Sha."
    }
    Write-Host "Reusing successful workflow run $($existing.databaseId)." -ForegroundColor Cyan
    return $existing
  }

  if ($Type -eq "signed") {
    $known = @(Get-WorkflowRuns $workflow $Sha | ForEach-Object { [long]$_.databaseId })

    Write-Host "Dispatching signed Byte Device Signoff Candidate for $Sha..." -ForegroundColor Cyan
    Invoke-Gh @(
      "workflow", "run", $workflow,
      "--repo", $repository,
      "--ref", "main"
    )

    $run = Wait-ForDispatchedRun $workflow $Sha $known
    Write-Host "Watching signed candidate workflow run $($run.databaseId)..." -ForegroundColor Cyan

    & gh run watch $run.databaseId --repo $repository --exit-status
    if ($LASTEXITCODE -ne 0) {
      Write-Host ""
      Write-Host "Signed device candidate workflow failed." -ForegroundColor Red
      Write-Host "Confirm WINDOWS_CERTIFICATE, WINDOWS_CERTIFICATE_PASSWORD, and WINDOWS_TIMESTAMP_URL are configured and valid."
      throw "Signed device candidate workflow $($run.databaseId) did not complete successfully."
    }

    $runs = @(Get-WorkflowRuns $workflow $Sha | Where-Object {
      [long]$_.databaseId -eq [long]$run.databaseId -and
      $_.status -eq "completed" -and
      $_.conclusion -eq "success"
    })
    if ($runs.Count -ne 1) {
      throw "The dispatched signed candidate run did not resolve to one successful exact-main run."
    }
    return $runs[0]
  }

  $existing = Find-SuccessfulRun $workflow $Sha
  if ($null -eq $existing) {
    throw "No successful Packaging Certification run exists for exact main commit $Sha. Let Byte Packaging Certification finish first."
  }
  return $existing
}

function Download-Candidate(
  [object]$Run,
  [string]$Type,
  [string]$Sha,
  [string]$Destination
) {
  $artifactName = Get-CandidateArtifactName $Type $Sha

  if (Test-Path $Destination) {
    Remove-Item $Destination -Recurse -Force
  }
  New-Item -ItemType Directory -Path $Destination -Force | Out-Null

  Write-Host "Downloading $artifactName from workflow run $($Run.databaseId)..." -ForegroundColor Cyan
  & gh run download $Run.databaseId --repo $repository --name $artifactName --dir $Destination
  if ($LASTEXITCODE -ne 0) {
    throw "Could not download workflow artifact '$artifactName'."
  }

  if (!(Test-Path (Join-Path $Destination "release-manifest.json"))) {
    throw "Downloaded artifact does not contain release-manifest.json at its root."
  }
}

function Get-DefaultDownloadRoot {
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path $documents "Byte\ReleaseApproval"
}

function Resolve-ReportPath([string]$Version) {
  if (-not [string]::IsNullOrWhiteSpace($Report)) {
    return [IO.Path]::GetFullPath($Report)
  }
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path (Join-Path $documents "Byte") ("device-certification-v" + $Version + ".json")
}

function Resolve-ApprovalPath([string]$Version) {
  if (-not [string]::IsNullOrWhiteSpace($ApprovalOutput)) {
    return [IO.Path]::GetFullPath($ApprovalOutput)
  }
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path (Join-Path $documents "Byte") ("release-approval-v" + $Version + ".json")
}

function FileHash([string]$Path) {
  return (Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Verify-GitHubProvenance([string]$CandidatePath, [object]$Manifest) {
  $installer = Join-Path $CandidatePath ([string]$Manifest.installer)
  $portable = Join-Path $CandidatePath ([string]$Manifest.portable)

  foreach ($artifact in @($installer, $portable)) {
    & gh attestation verify $artifact --repo $repository
    if ($LASTEXITCODE -ne 0) {
      throw "GitHub provenance verification failed for $artifact."
    }
  }

  Write-Host "GitHub provenance verified for installer and portable candidate artifacts." -ForegroundColor Cyan
}

if (!(Test-Path "src-tauri/tauri.conf.json") -or !(Test-Path "package.json")) {
  throw "Run release approval from the Byte repository root."
}

Require-Command "gh"
Require-Command "git"
Invoke-Gh @("auth", "status")

$remoteMain = Get-RemoteMainSha
$localHead = Get-LocalHeadSha
$localBranch = Get-LocalBranch
$localOrigin = Get-LocalOrigin
Assert-CleanWorkingTree

if ($localOrigin -notmatch '(^|[:/])thiepn/byte(?:\.git)?$') {
  throw "Release approval must run from the thiepn/byte repository. origin is '$localOrigin'."
}
if ($localBranch -ne "main") {
  throw "Release approval must run from the local main branch. Current branch: '$localBranch'."
}
if ($localHead -ne $remoteMain) {
  throw "Local main ($localHead) does not match remote main ($remoteMain). Pull the exact current main commit first."
}

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version

if ([string]::IsNullOrWhiteSpace($DownloadRoot)) {
  $DownloadRoot = Get-DefaultDownloadRoot
}
$DownloadRoot = [IO.Path]::GetFullPath($DownloadRoot)

if ($VerifyOnly -and [string]::IsNullOrWhiteSpace($CandidateDir) -and -not $ReuseExistingCandidate) {
  throw "VerifyOnly is side-effect-free: provide -CandidateDir or add -ReuseExistingCandidate."
}

$run = $null
if ([string]::IsNullOrWhiteSpace($CandidateDir)) {
  $candidatePath = Join-Path $DownloadRoot ("candidate-" + $CandidateType + "-" + $remoteMain)
  $run = Resolve-OrCreateCandidateRun $CandidateType $remoteMain
  Download-Candidate $run $CandidateType $remoteMain $candidatePath
} else {
  $candidatePath = (Resolve-Path $CandidateDir).Path
}

$artifactVerifyArgs = @{
  OutputDir = $candidatePath
  RequireCertification = $true
}
if ($CandidateType -eq "signed") {
  $artifactVerifyArgs.RequireSigning = $true
}
& (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") @artifactVerifyArgs

$manifestPath = Join-Path $candidatePath "release-manifest.json"
$productCertificationPath = Join-Path $candidatePath "product-certification.json"
$releaseCertificationPath = Join-Path $candidatePath "release-certification.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json

if ([string]$manifest.commit -ne $remoteMain) {
  throw "Candidate source commit '$($manifest.commit)' does not match exact current main '$remoteMain'."
}
if ($CandidateType -eq "signed" -and -not [bool]$manifest.signed) {
  throw "Signed release approval requires a signed candidate."
}

$githubAttestationVerified = $false
if ($CandidateType -eq "signed") {
  Verify-GitHubProvenance $candidatePath $manifest
  $githubAttestationVerified = $true
}

$reportPath = Resolve-ReportPath $version

if (-not $VerifyOnly) {
  Write-Host ""
  Write-Host "Starting P4-M physical Windows signoff." -ForegroundColor Green

  $signoffArgs = @{
    CandidateDir = $candidatePath
    PerformanceMinutes = $PerformanceMinutes
    Output = $reportPath
  }
  if ($CandidateType -eq "signed") {
    $signoffArgs.RequireSigning = $true
  }

  & (Join-Path $PSScriptRoot "run-physical-device-signoff.ps1") @signoffArgs
}

if (!(Test-Path $reportPath)) {
  throw "Physical device report is missing: $reportPath"
}

$deviceVerifyArgs = @{
  CandidateDir = $candidatePath
  Report = $reportPath
}
if ($CandidateType -eq "signed") {
  $deviceVerifyArgs.RequireSigning = $true
  $deviceVerifyArgs.RequirePublicReleaseReady = $true
}
& (Join-Path $PSScriptRoot "verify-physical-device-signoff.ps1") @deviceVerifyArgs

$deviceReport = Get-Content $reportPath -Raw | ConvertFrom-Json
if ($deviceReport.device_ready -ne $true) {
  throw "Release approval requires a device-ready physical signoff report."
}
if ($CandidateType -eq "signed" -and $deviceReport.public_release_ready -ne $true) {
  throw "Signed release approval requires a public-release-ready physical device report."
}

$finalRemoteMain = Get-RemoteMainSha
$finalLocalHead = Get-LocalHeadSha
$finalLocalBranch = Get-LocalBranch
Assert-CleanWorkingTree

if ($finalRemoteMain -ne $remoteMain) {
  throw "Remote main moved during physical signoff ($remoteMain -> $finalRemoteMain). This evidence remains candidate-bound, but a fresh approval run is required before tagging."
}
if ($finalLocalHead -ne $localHead -or $finalLocalHead -ne $remoteMain) {
  throw "Local HEAD changed during physical signoff. A fresh exact-main approval run is required."
}
if ($finalLocalBranch -ne "main") {
  throw "Local branch changed during physical signoff. A fresh exact-main approval run is required."
}

$approvalPath = Resolve-ApprovalPath $version
$approvalParent = Split-Path $approvalPath -Parent
if (-not [string]::IsNullOrWhiteSpace($approvalParent)) {
  New-Item -ItemType Directory -Path $approvalParent -Force | Out-Null
}

$approval = [ordered]@{
  schema_version = 1
  product = "Byte"
  version = $version
  generated_at_utc = [DateTime]::UtcNow.ToString("o")
  approval_scope = if ($CandidateType -eq "signed") { "public-release-pretag" } else { "device-qa" }
  source = [ordered]@{
    repository = $repository
    branch = "main"
    commit = $remoteMain
    local_head = $localHead
  }
  candidate = [ordered]@{
    type = $CandidateType
    workflow_run_id = if ($null -eq $run) { $null } else { [long]$run.databaseId }
    workflow_url = if ($null -eq $run) { $null } else { [string]$run.url }
    signed = [bool]$manifest.signed
    timestamped = [bool]$manifest.signing.timestamped
    same_signer = [bool]$manifest.signing.same_signer
    github_attestation_verified = [bool]$githubAttestationVerified
    installer = [ordered]@{
      file = [string]$manifest.installer
      sha256 = FileHash (Join-Path $candidatePath ([string]$manifest.installer))
    }
    portable = [ordered]@{
      file = [string]$manifest.portable
      sha256 = FileHash (Join-Path $candidatePath ([string]$manifest.portable))
    }
    release_manifest_sha256 = FileHash $manifestPath
    product_certification_sha256 = FileHash $productCertificationPath
    release_certification_sha256 = FileHash $releaseCertificationPath
  }
  physical_device_report = [ordered]@{
    file = [IO.Path]::GetFileName($reportPath)
    sha256 = FileHash $reportPath
    device_ready = [bool]$deviceReport.device_ready
    public_release_ready = [bool]$deviceReport.public_release_ready
    generated_at_utc = [string]$deviceReport.generated_at_utc
  }
  approved_for_tagging = (
    $CandidateType -eq "signed" -and
    [bool]$manifest.signed -and
    [bool]$manifest.signing.timestamped -and
    [bool]$manifest.signing.same_signer -and
    [bool]$deviceReport.public_release_ready
  )
  automatic_tag_created = $false
  automatic_release_published = $false
}

$approval | ConvertTo-Json -Depth 12 | Set-Content $approvalPath -Encoding utf8

$approvalVerifyArgs = @{
  CandidateDir = $candidatePath
  DeviceReport = $reportPath
  Approval = $approvalPath
}
if ($CandidateType -eq "signed") {
  $approvalVerifyArgs.RequireTaggingApproval = $true
}
& (Join-Path $PSScriptRoot "verify-release-approval.ps1") @approvalVerifyArgs

Write-Host ""
Write-Host "Release approval receipt written to:" -ForegroundColor Green
Write-Host "  $approvalPath"
Write-Host "Candidate directory:"
Write-Host "  $candidatePath"
Write-Host "Device report:"
Write-Host "  $reportPath"
Write-Host ""

if ($approval.approved_for_tagging) {
  Write-Host "APPROVED FOR TAGGING." -ForegroundColor Green
  Write-Host "No tag or GitHub Release was created automatically."
} else {
  Write-Host "Device QA evidence verified, but this receipt is not a public tagging approval." -ForegroundColor Yellow
}
