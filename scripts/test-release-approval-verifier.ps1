param(
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path $CandidateDir).Path
$manifestPath = Join-Path $root "release-manifest.json"
$productPath = Join-Path $root "product-certification.json"
$releasePath = Join-Path $root "release-certification.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$version = [string]$manifest.version
$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$temp = Join-Path $tempRoot ("byte-release-approval-test-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temp -Force | Out-Null

function Hash([string]$Path) {
  return (Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Expect-Rejected([string]$ApprovalPath, [string]$ReportPath, [string]$Reason) {
  $rejected = $false
  try {
    $args = @{
      CandidateDir = $root
      DeviceReport = $ReportPath
      Approval = $ApprovalPath
    }
    & (Join-Path $PSScriptRoot "verify-release-approval.ps1") @args
  } catch {
    $rejected = $true
    Write-Host "Expected release-approval rejection ($Reason): $($_.Exception.Message)"
  }
  if (-not $rejected) {
    throw "Release approval verifier accepted invalid synthetic evidence: $Reason"
  }
}

try {
  $portablePath = Join-Path $root ([string]$manifest.portable)
  $installerPath = Join-Path $root ([string]$manifest.installer)
  $extract = Join-Path $temp "portable"
  New-Item -ItemType Directory -Path $extract -Force | Out-Null
  Expand-Archive -Path $portablePath -DestinationPath $extract -Force
  $exePath = Join-Path $extract "Byte.exe"
  if (!(Test-Path $exePath)) {
    throw "Release approval harness could not find portable Byte.exe."
  }

  $signed = [bool]$manifest.signed
  $trustStatus = if ($signed) { "PASS" } else { "NA" }
  $gaps = if ($signed) { @() } else { @("signed-installer-trust: synthetic unsigned candidate") }

  $manual = [ordered]@{
    first_run_everyday = @{ status = "PASS" }
    companion = @{ status = "PASS" }
    lifecycle = @{ status = "PASS" }
    multimonitor_dpi = @{ status = "PASS" }
    accessibility_visual = @{ status = "PASS" }
    notifications = @{ status = "PASS" }
    installer_trust = @{ status = $trustStatus }
    performance_observation = @{ status = "PASS" }
  }

  $performance = @()
  foreach ($state in @("ACTIVE_CALM", "FULLSCREEN_REDUCED", "LOCKED_DISPLAY_OFF")) {
    $performance += [ordered]@{
      state = $state
      duration_seconds = 600
      automatic_pass = $true
      metrics = [ordered]@{
        cpu_average_percent = 0.05
        working_set_average_mb = 40
        gpu_average_percent = 0.1
        disk_write_average_bytes_per_sec = 0
        max_non_loopback_established_tcp_connections = 0
      }
    }
  }

  $publicReady =
    $signed -and
    [bool]$manifest.signing.timestamped -and
    [bool]$manifest.signing.same_signer

  $report = [ordered]@{
    schema_version = 1
    product = "Byte"
    version = $version
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
    certification_scope = "physical-windows-device-signoff"
    development_dry_run = $false
    candidate = [ordered]@{
      source_commit = [string]$manifest.commit
      signed = $signed
      timestamped = [bool]$manifest.signing.timestamped
      same_signer = [bool]$manifest.signing.same_signer
      github_attestation_verified = $signed
      installer = [ordered]@{
        file = [string]$manifest.installer
        sha256 = Hash $installerPath
      }
      portable = [ordered]@{
        file = [string]$manifest.portable
        sha256 = Hash $portablePath
        executable_sha256 = Hash $exePath
      }
      release_manifest_sha256 = Hash $manifestPath
      product_certification_sha256 = Hash $productPath
      release_certification_sha256 = Hash $releasePath
    }
    device = [ordered]@{
      os = [ordered]@{
        build_number = "26100"
        architecture = "64-bit"
      }
      display_count = 2
    }
    manual_checks = $manual
    performance = $performance
    capability_gaps = $gaps
    device_ready = $true
    public_release_ready = $publicReady
  }

  $reportPath = Join-Path $temp "device-report.json"
  $report | ConvertTo-Json -Depth 20 | Set-Content $reportPath -Encoding utf8

  $deviceVerifyArgs = @{
    CandidateDir = $root
    Report = $reportPath
  }
  if ($RequireSigning) {
    $deviceVerifyArgs.RequireSigning = $true
    $deviceVerifyArgs.RequirePublicReleaseReady = $true
  }
  & (Join-Path $PSScriptRoot "verify-physical-device-signoff.ps1") @deviceVerifyArgs

  $approval = [ordered]@{
    schema_version = 1
    product = "Byte"
    version = $version
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
    approval_scope = if ($signed) { "public-release-pretag" } else { "device-qa" }
    source = [ordered]@{
      repository = "thiepn/byte"
      branch = "main"
      commit = [string]$manifest.commit
      local_head = [string]$manifest.commit
    }
    candidate = [ordered]@{
      type = if ($signed) { "signed" } else { "unsigned" }
      workflow_run_id = $null
      workflow_url = $null
      signed = $signed
      timestamped = [bool]$manifest.signing.timestamped
      same_signer = [bool]$manifest.signing.same_signer
      installer = [ordered]@{
        file = [string]$manifest.installer
        sha256 = Hash $installerPath
      }
      portable = [ordered]@{
        file = [string]$manifest.portable
        sha256 = Hash $portablePath
      }
      release_manifest_sha256 = Hash $manifestPath
      product_certification_sha256 = Hash $productPath
      release_certification_sha256 = Hash $releasePath
    }
    physical_device_report = [ordered]@{
      file = [IO.Path]::GetFileName($reportPath)
      sha256 = Hash $reportPath
      device_ready = $true
      public_release_ready = $publicReady
      generated_at_utc = [string]$report.generated_at_utc
    }
    approved_for_tagging = $publicReady
    automatic_tag_created = $false
    automatic_release_published = $false
  }

  $approvalPath = Join-Path $temp "release-approval.json"
  $approval | ConvertTo-Json -Depth 20 | Set-Content $approvalPath -Encoding utf8

  $approvalVerifyArgs = @{
    CandidateDir = $root
    DeviceReport = $reportPath
    Approval = $approvalPath
  }
  if ($RequireSigning) {
    $approvalVerifyArgs.RequireTaggingApproval = $true
  }
  & (Join-Path $PSScriptRoot "verify-release-approval.ps1") @approvalVerifyArgs

  $tampered = Get-Content $approvalPath -Raw | ConvertFrom-Json
  $tampered.physical_device_report.sha256 = ("0" * 64)
  $tamperedPath = Join-Path $temp "tampered-report-hash-approval.json"
  $tampered | ConvertTo-Json -Depth 20 | Set-Content $tamperedPath -Encoding utf8
  Expect-Rejected $tamperedPath $reportPath "device-report hash tampering"

  $autoTag = Get-Content $approvalPath -Raw | ConvertFrom-Json
  $autoTag.automatic_tag_created = $true
  $autoTagPath = Join-Path $temp "automatic-tag-approval.json"
  $autoTag | ConvertTo-Json -Depth 20 | Set-Content $autoTagPath -Encoding utf8
  Expect-Rejected $autoTagPath $reportPath "automatic tag claim"

  $wrongCommit = Get-Content $approvalPath -Raw | ConvertFrom-Json
  $wrongCommit.source.commit = ("0" * 40)
  $wrongCommitPath = Join-Path $temp "wrong-commit-approval.json"
  $wrongCommit | ConvertTo-Json -Depth 20 | Set-Content $wrongCommitPath -Encoding utf8
  Expect-Rejected $wrongCommitPath $reportPath "source commit tampering"

  Write-Host "Release approval verifier candidate-bound harness passed."
} finally {
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
