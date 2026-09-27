param(
  [Parameter(Mandatory = $true)]
  [string]$Approval,
  [string]$CandidateDir = "",
  [string]$DeviceReport = "",
  [string]$Output = "",
  [string]$ConfirmTag = "",
  [switch]$DryRun,
  [switch]$ResumeExistingTag
)

$ErrorActionPreference = "Stop"

$repository = "thiepn/byte"
$releaseWorkflow = "release.yml"

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

function Get-RemoteMainSha {
  $result = Invoke-GhJson @("api", "repos/$repository/commits/main")
  return [string]$result.sha
}

function Get-LocalHeadSha {
  $output = & git rev-parse HEAD
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine local Git HEAD."
  }
  return ($output | Out-String).Trim()
}

function Get-LocalBranch {
  $output = & git branch --show-current
  if ($LASTEXITCODE -ne 0) {
    throw "Could not determine local Git branch."
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
    throw "The working tree is not clean. Commit or stash local changes before controlled tagging."
  }
}

function Get-RemoteTagSha([string]$Tag) {
  $output = & git ls-remote --exit-code --tags origin "refs/tags/$Tag" 2>$null
  $code = $LASTEXITCODE

  if ($code -eq 2) {
    return $null
  }
  if ($code -ne 0) {
    throw "Could not query remote tag refs for $Tag."
  }

  $line = (($output | Out-String).Trim() -split "\r?\n" | Select-Object -First 1)
  if ($line -notmatch '^([0-9a-fA-F]{40})\s+refs/tags/') {
    throw "Remote tag lookup returned an unexpected value for $Tag."
  }
  return $matches[1].ToLowerInvariant()
}

function Get-LocalTagCommit([string]$Tag) {
  $output = & git rev-parse --verify --quiet ("refs/tags/" + $Tag + "^{commit}") 2>$null
  if ($LASTEXITCODE -ne 0) {
    return $null
  }

  $sha = ($output | Out-String).Trim()
  if ($sha -notmatch '^[0-9a-fA-F]{40}$') {
    throw "Local tag '$Tag' resolved to an unexpected value."
  }
  return $sha.ToLowerInvariant()
}

function Assert-LocalTagState([string]$Tag, [string]$ApprovedSha, [bool]$AllowExisting) {
  $localTag = Get-LocalTagCommit $Tag
  if ($null -eq $localTag) {
    return
  }

  if (-not $AllowExisting) {
    throw "Local tag '$Tag' already exists. Fresh P6 initiation requires an unused local tag name."
  }
  if ($localTag -ne $ApprovedSha.ToLowerInvariant()) {
    throw "Local tag '$Tag' points to $localTag, not approved commit $ApprovedSha."
  }
}

function Get-Releases {
  $result = Invoke-GhJson @(
    "release", "list",
    "--repo", $repository,
    "--limit", "100",
    "--json", "tagName,isDraft,isPrerelease,publishedAt"
  )
  if ($null -eq $result) {
    return @()
  }
  return @($result)
}

function Assert-TagAndReleaseAbsent([string]$Tag) {
  $remoteTag = Get-RemoteTagSha $Tag
  if ($null -ne $remoteTag) {
    return $remoteTag
  }

  $sameRelease = @(Get-Releases | Where-Object { $_.tagName -eq $Tag })
  if ($sameRelease.Count -gt 0) {
    throw "GitHub already contains a release or draft for '$Tag'."
  }

  return $null
}

function Assert-ReleaseOrder([string]$Tag, [string]$Version) {
  $isBeta = $Version -match '-beta\.'
  $releases = @(Get-Releases | Where-Object {
    -not $_.isDraft -and
    $_.tagName -ne $Tag -and
    ($isBeta -or -not $_.isPrerelease)
  })

  foreach ($release in $releases) {
    if ([string]$release.tagName -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(?:-beta\.[0-9]+)?$') {
      continue
    }

    & node scripts/check-release-order.mjs --current $Tag --previous ([string]$release.tagName)
    if ($LASTEXITCODE -ne 0) {
      throw "Release ordering rejected $Tag relative to $($release.tagName)."
    }
  }
}

function Get-SuccessfulWorkflowRun([string]$Workflow, [string]$Sha) {
  $runs = Invoke-GhJson @(
    "run", "list",
    "--repo", $repository,
    "--workflow", $Workflow,
    "--branch", "main",
    "--commit", $Sha,
    "--limit", "20",
    "--json", "databaseId,headSha,status,conclusion,event,url,createdAt"
  )

  $matches = @($runs | Where-Object {
    $_.headSha -eq $Sha -and
    $_.event -eq "push" -and
    $_.status -eq "completed" -and
    $_.conclusion -eq "success"
  } | Sort-Object createdAt -Descending)

  if ($matches.Count -eq 0) {
    throw "No successful exact-main $Workflow push run exists for commit $Sha."
  }

  return $matches[0]
}

function Get-ReleaseRuns([string]$Sha) {
  $runs = Invoke-GhJson @(
    "run", "list",
    "--repo", $repository,
    "--workflow", $releaseWorkflow,
    "--limit", "50",
    "--json", "databaseId,headSha,headBranch,status,conclusion,event,url,createdAt,displayTitle"
  )
  if ($null -eq $runs) {
    return @()
  }
  return @($runs | Where-Object {
    $_.headSha -eq $Sha -and $_.event -eq "push"
  })
}

function Wait-ForReleaseRun(
  [string]$Tag,
  [string]$Sha,
  [DateTime]$NotBeforeUtc,
  [switch]$AllowExisting
) {
  $deadline = [DateTime]::UtcNow.AddMinutes(3)

  while ([DateTime]::UtcNow -lt $deadline) {
    $runs = @(Get-ReleaseRuns $Sha)

    if (-not $AllowExisting) {
      $runs = @($runs | Where-Object {
        [DateTime]::Parse([string]$_.createdAt).ToUniversalTime() -ge $NotBeforeUtc.AddMinutes(-1)
      })
    }

    $tagMatches = @($runs | Where-Object {
      [string]$_.headBranch -eq $Tag -or [string]$_.displayTitle -eq $Tag
    })

    if ($tagMatches.Count -eq 1) {
      return $tagMatches[0]
    }
    if ($tagMatches.Count -gt 1) {
      throw "Multiple Byte Release runs matched tag '$Tag'; refusing to guess."
    }

    if ($runs.Count -eq 1) {
      return $runs[0]
    }
    if ($runs.Count -gt 1) {
      throw "Multiple Byte Release runs matched commit $Sha without a unique tag identity."
    }

    Start-Sleep -Seconds 3
  }

  return $null
}

function FileHash([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Resolve-DefaultCandidate([string]$Commit) {
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path $documents ("Byte\ReleaseApproval\candidate-signed-" + $Commit)
}

function Resolve-DefaultDeviceReport([string]$FileName) {
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path (Join-Path $documents "Byte") ([IO.Path]::GetFileName($FileName))
}

function Resolve-DefaultOutput([string]$Version) {
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  return Join-Path (Join-Path $documents "Byte") ("release-initiation-v" + $Version + ".json")
}

if (!(Test-Path "package.json") -or !(Test-Path "src-tauri/tauri.conf.json")) {
  throw "Run P6 from the Byte repository root."
}

Require-Command "gh"
Require-Command "git"
Require-Command "node"

& gh auth status
if ($LASTEXITCODE -ne 0) {
  throw "GitHub CLI authentication is required."
}

$approvalPath = (Resolve-Path -LiteralPath $Approval).Path
$approvalData = Get-Content -LiteralPath $approvalPath -Raw | ConvertFrom-Json

if ($approvalData.schema_version -ne 1 -or $approvalData.product -ne "Byte") {
  throw "P6 requires a valid P5 Byte approval receipt."
}
if ($approvalData.approved_for_tagging -ne $true) {
  throw "P6 requires approved_for_tagging: true."
}
if ($approvalData.candidate.github_attestation_verified -ne $true) {
  throw "P6 requires a P5 receipt with verified GitHub provenance."
}

$approvedCommit = [string]$approvalData.source.commit
if ($approvedCommit -notmatch '^[0-9a-fA-F]{40}$') {
  throw "Approval receipt source commit is invalid."
}

if ([string]::IsNullOrWhiteSpace($CandidateDir)) {
  $CandidateDir = Resolve-DefaultCandidate $approvedCommit
}
$candidatePath = (Resolve-Path -LiteralPath $CandidateDir).Path

if ([string]::IsNullOrWhiteSpace($DeviceReport)) {
  $DeviceReport = Resolve-DefaultDeviceReport ([string]$approvalData.physical_device_report.file)
}
$deviceReportPath = (Resolve-Path -LiteralPath $DeviceReport).Path

$remoteMain = Get-RemoteMainSha
$localHead = Get-LocalHeadSha
$localBranch = Get-LocalBranch
$localOrigin = Get-LocalOrigin
Assert-CleanWorkingTree

if ($localOrigin -notmatch '(^|[:/])thiepn/byte(?:\.git)?$') {
  throw "P6 must run from the thiepn/byte repository. origin is '$localOrigin'."
}
if ($localBranch -ne "main") {
  throw "P6 must run from local main. Current branch: '$localBranch'."
}
if ($localHead -ne $remoteMain) {
  throw "Local main ($localHead) does not match remote main ($remoteMain)."
}
if ($approvedCommit.ToLowerInvariant() -ne $remoteMain.ToLowerInvariant()) {
  throw "P5 approval commit $approvedCommit is stale; current main is $remoteMain."
}

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$tag = "v$version"

if ([string]$approvalData.version -ne $version) {
  throw "Approval receipt version '$($approvalData.version)' does not match current Byte version '$version'."
}

& node scripts/verify-release-version.mjs --tag $tag
if ($LASTEXITCODE -ne 0) {
  throw "Release version/tag metadata verification failed."
}

& npm run maintenance:verify
if ($LASTEXITCODE -ne 0) {
  throw "Maintenance/release policy verification failed."
}

$tempNotes = Join-Path $env:TEMP ("byte-p6-notes-" + [guid]::NewGuid().ToString("N") + ".md")
try {
  & node scripts/extract-release-notes.mjs --version $version --output $tempNotes
  if ($LASTEXITCODE -ne 0 -or !(Test-Path $tempNotes) -or (Get-Item $tempNotes).Length -eq 0) {
    throw "Concrete release notes for Byte $version are missing."
  }
} finally {
  Remove-Item $tempNotes -Force -ErrorAction SilentlyContinue
}

$approvalVerifyArgs = @{
  CandidateDir = $candidatePath
  DeviceReport = $deviceReportPath
  Approval = $approvalPath
  RequireTaggingApproval = $true
}
& (Join-Path $PSScriptRoot "verify-release-approval.ps1") @approvalVerifyArgs

$ciRun = Get-SuccessfulWorkflowRun "ci.yml" $remoteMain
$packageRun = Get-SuccessfulWorkflowRun "package.yml" $remoteMain

Assert-ReleaseOrder $tag $version

$existingRemoteTag = Assert-TagAndReleaseAbsent $tag
Assert-LocalTagState $tag $remoteMain ([bool]$ResumeExistingTag)

if ($ResumeExistingTag) {
  if ($null -eq $existingRemoteTag) {
    throw "ResumeExistingTag was requested, but remote tag '$tag' does not exist."
  }
  if ($existingRemoteTag -ne $remoteMain.ToLowerInvariant()) {
    throw "Existing remote tag '$tag' points to $existingRemoteTag, not approved commit $remoteMain."
  }
} elseif ($null -ne $existingRemoteTag) {
  throw "Remote tag '$tag' already exists. Use -ResumeExistingTag only if it already points to the exact approved commit."
}

Write-Host ""
Write-Host "P6 controlled release preflight passed." -ForegroundColor Green
Write-Host "  Version: $version"
Write-Host "  Tag: $tag"
Write-Host "  Commit: $remoteMain"
Write-Host "  Approval SHA-256: $(FileHash $approvalPath)"
Write-Host "  Byte CI run: $($ciRun.databaseId)"
Write-Host "  Packaging run: $($packageRun.databaseId)"
Write-Host "  Resume existing tag: $([bool]$ResumeExistingTag)"
Write-Host ""

if ($DryRun) {
  if ($ResumeExistingTag) {
    Write-Host "Dry run passed. Existing exact-commit tag would be resumed; no Git operation was performed." -ForegroundColor Yellow
  } else {
    Write-Host "Dry run passed. No tag was created and no release was initiated." -ForegroundColor Yellow
  }
  exit 0
}

$createdByThisRun = $false
$initiationStartedUtc = [DateTime]::UtcNow

if (-not $ResumeExistingTag) {
  $confirmation = $ConfirmTag
  if ([string]::IsNullOrWhiteSpace($confirmation)) {
    $confirmation = Read-Host "Type '$tag' exactly to create the release tag and start Byte Release"
  }
  if ($confirmation -cne $tag) {
    throw "Tag confirmation did not exactly match '$tag'. Nothing was tagged."
  }

  Write-Host "Creating lightweight exact-commit tag $tag -> $remoteMain..." -ForegroundColor Cyan
  & git push origin ($remoteMain + ":refs/tags/" + $tag)
  if ($LASTEXITCODE -ne 0) {
    throw "Git tag push failed. P6 does not retry with force."
  }
  $createdByThisRun = $true
}

$verifiedRemoteTag = Get-RemoteTagSha $tag
if ($null -eq $verifiedRemoteTag -or $verifiedRemoteTag -ne $remoteMain.ToLowerInvariant()) {
  throw "Remote tag verification failed: '$tag' does not point exactly to approved commit $remoteMain."
}

$releaseRun = Wait-ForReleaseRun $tag $remoteMain $initiationStartedUtc -AllowExisting:$ResumeExistingTag

$outputPath = if ([string]::IsNullOrWhiteSpace($Output)) {
  Resolve-DefaultOutput $version
} else {
  [IO.Path]::GetFullPath($Output)
}
$outputParent = Split-Path $outputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outputParent)) {
  New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
}

$receipt = [ordered]@{
  schema_version = 1
  product = "Byte"
  version = $version
  generated_at_utc = [DateTime]::UtcNow.ToString("o")
  initiation_scope = "controlled-tag-and-release-workflow-start"
  source = [ordered]@{
    repository = $repository
    branch = "main"
    commit = $remoteMain
    local_head = $localHead
  }
  approval = [ordered]@{
    file = [IO.Path]::GetFileName($approvalPath)
    sha256 = FileHash $approvalPath
    approved_for_tagging = $true
    github_attestation_verified = $true
  }
  physical_device_report = [ordered]@{
    file = [IO.Path]::GetFileName($deviceReportPath)
    sha256 = FileHash $deviceReportPath
  }
  candidate = [ordered]@{
    release_manifest_sha256 = FileHash (Join-Path $candidatePath "release-manifest.json")
    release_certification_sha256 = FileHash (Join-Path $candidatePath "release-certification.json")
  }
  preflight = [ordered]@{
    exact_remote_main = $true
    clean_working_tree = $true
    release_order_verified = $true
    tag_and_release_name_available_before_creation = (-not $ResumeExistingTag)
    ci = [ordered]@{
      run_id = [long]$ciRun.databaseId
      url = [string]$ciRun.url
      conclusion = [string]$ciRun.conclusion
    }
    packaging = [ordered]@{
      run_id = [long]$packageRun.databaseId
      url = [string]$packageRun.url
      conclusion = [string]$packageRun.conclusion
    }
  }
  tag = [ordered]@{
    name = $tag
    ref = "refs/tags/$tag"
    type = "lightweight"
    target_commit = $verifiedRemoteTag
    created_by_this_run = $createdByThisRun
    resumed_existing_tag = [bool]$ResumeExistingTag
    forced_update = $false
  }
  release_workflow = [ordered]@{
    detected = ($null -ne $releaseRun)
    workflow = "Byte Release"
    workflow_file = $releaseWorkflow
    run_id = if ($null -eq $releaseRun) { $null } else { [long]$releaseRun.databaseId }
    url = if ($null -eq $releaseRun) { $null } else { [string]$releaseRun.url }
    status = if ($null -eq $releaseRun) { $null } else { [string]$releaseRun.status }
    conclusion = if ($null -eq $releaseRun) { $null } else { [string]$releaseRun.conclusion }
    event = if ($null -eq $releaseRun) { $null } else { [string]$releaseRun.event }
    head_sha = if ($null -eq $releaseRun) { $null } else { [string]$releaseRun.headSha }
  }
  direct_release_publish_performed = $false
  direct_asset_upload_performed = $false
}

$receipt | ConvertTo-Json -Depth 12 | Set-Content $outputPath -Encoding utf8

if ($null -eq $releaseRun) {
  Write-Host "Tag exists and is verified, but Byte Release was not detected within the P6 observation window." -ForegroundColor Red
  Write-Host "Initiation receipt written to $outputPath"
  throw "Release workflow detection failed. Do not recreate or force-move the tag; investigate the existing tag/workflow state."
}

$verifyArgs = @{
  CandidateDir = $candidatePath
  DeviceReport = $deviceReportPath
  Approval = $approvalPath
  Initiation = $outputPath
  RequireRemoteState = $true
}
& (Join-Path $PSScriptRoot "verify-release-initiation.ps1") @verifyArgs

Write-Host ""
Write-Host "P6 release initiation verified." -ForegroundColor Green
Write-Host "  Tag: $tag -> $remoteMain"
Write-Host "  Byte Release run: $($releaseRun.databaseId)"
Write-Host "  Workflow state: $($releaseRun.status) $($releaseRun.conclusion)"
Write-Host "  Receipt: $outputPath"
Write-Host ""
Write-Host "P6 did not publish assets directly. The existing Byte Release workflow owns signing, certification, publication, and public-download verification."
