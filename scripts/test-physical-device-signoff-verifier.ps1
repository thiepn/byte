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
$temp = Join-Path $env:RUNNER_TEMP ("byte-device-verifier-test-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temp -Force | Out-Null

function Hash([string]$Path) {
  return (Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Expect-Rejected([string]$Path, [string]$Reason) {
  $rejected = $false
  try {
    & (Join-Path $PSScriptRoot "verify-physical-device-signoff.ps1") -CandidateDir $root -Report $Path
  } catch {
    $rejected = $true
    Write-Host "Expected verifier rejection ($Reason): $($_.Exception.Message)"
  }
  if (-not $rejected) {
    throw "Physical device verifier accepted invalid synthetic evidence: $Reason"
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
    throw "Synthetic verifier harness could not find portable Byte.exe."
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
    certification_scope = "physical-windows-device-signoff"
    development_dry_run = $false
    candidate = [ordered]@{
      source_commit = [string]$manifest.commit
      signed = $signed
      timestamped = [bool]$manifest.signing.timestamped
      same_signer = [bool]$manifest.signing.same_signer
      installer = [ordered]@{
        file = [string]$manifest.installer
        sha256 = (Hash $installerPath)
      }
      portable = [ordered]@{
        file = [string]$manifest.portable
        sha256 = (Hash $portablePath)
        executable_sha256 = (Hash $exePath)
      }
      release_manifest_sha256 = (Hash $manifestPath)
      product_certification_sha256 = (Hash $productPath)
      release_certification_sha256 = (Hash $releasePath)
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

  $valid = Join-Path $temp "valid-device-report.json"
  $report | ConvertTo-Json -Depth 20 | Set-Content $valid -Encoding utf8
  $serialized = Get-Content -LiteralPath $valid -Raw | ConvertFrom-Json
  if ($serialized.schema_version -ne 1 -or $serialized.product -ne "Byte") {
    throw "Synthetic physical-device report serialization lost required top-level identity fields."
  }

  $verifyArgs = @{
    CandidateDir = $root
    Report = $valid
  }
  if ($RequireSigning) {
    $verifyArgs.RequireSigning = $true
    $verifyArgs.RequirePublicReleaseReady = $true
  }
  & (Join-Path $PSScriptRoot "verify-physical-device-signoff.ps1") @verifyArgs

  $short = Get-Content $valid -Raw | ConvertFrom-Json
  $short.performance[0].duration_seconds = 599
  $shortPath = Join-Path $temp "short-device-report.json"
  $short | ConvertTo-Json -Depth 20 | Set-Content $shortPath -Encoding utf8
  Expect-Rejected $shortPath "performance duration below 10 minutes"

  $tampered = Get-Content $valid -Raw | ConvertFrom-Json
  $tampered.candidate.portable.sha256 = ("0" * 64)
  $tamperedPath = Join-Path $temp "tampered-device-report.json"
  $tampered | ConvertTo-Json -Depth 20 | Set-Content $tamperedPath -Encoding utf8
  Expect-Rejected $tamperedPath "candidate hash tampering"

  Write-Host "Physical device signoff verifier candidate-bound harness passed."
} finally {
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
