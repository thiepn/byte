param(
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [Parameter(Mandatory = $true)]
  [string]$Report,
  [switch]$RequireSigning,
  [switch]$RequirePublicReleaseReady
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path $CandidateDir).Path
$reportPath = (Resolve-Path $Report).Path
$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version

$verifyArgs = @{
  OutputDir = $root
  RequireCertification = $true
}
if ($RequireSigning -or $RequirePublicReleaseReady) {
  $verifyArgs.RequireSigning = $true
}
& (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") @verifyArgs

$manifestPath = Join-Path $root "release-manifest.json"
$productCertificationPath = Join-Path $root "product-certification.json"
$releaseCertificationPath = Join-Path $root "release-certification.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$report = Get-Content $reportPath -Raw | ConvertFrom-Json

function Assert-Equal([object]$Actual, [object]$Expected, [string]$Message) {
  if ($Actual -ne $Expected) {
    throw "$Message Expected '$Expected', got '$Actual'."
  }
}

function Digest([string]$Path) {
  return (Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

Assert-Equal $report.schema_version 1 "Physical device certification schema mismatch."
Assert-Equal $report.product "Byte" "Physical device certification product mismatch."
Assert-Equal ([string]$report.version) $version "Physical device certification version mismatch."
Assert-Equal $report.certification_scope "physical-windows-device-signoff" "Physical device certification scope mismatch."

if ($report.development_dry_run -eq $true) {
  throw "Development dry-run reports cannot certify a release."
}

Assert-Equal ([string]$report.candidate.source_commit) ([string]$manifest.commit) "Device report source commit mismatch."
Assert-Equal ([bool]$report.candidate.signed) ([bool]$manifest.signed) "Device report signing state mismatch."
Assert-Equal ([bool]$report.candidate.timestamped) ([bool]$manifest.signing.timestamped) "Device report timestamp state mismatch."
Assert-Equal ([bool]$report.candidate.same_signer) ([bool]$manifest.signing.same_signer) "Device report signer-consistency state mismatch."

$installerPath = Join-Path $root ([string]$manifest.installer)
$portablePath = Join-Path $root ([string]$manifest.portable)

Assert-Equal ([string]$report.candidate.installer.file) ([string]$manifest.installer) "Device report installer filename mismatch."
Assert-Equal ([string]$report.candidate.portable.file) ([string]$manifest.portable) "Device report portable filename mismatch."
Assert-Equal ([string]$report.candidate.installer.sha256) (Digest $installerPath) "Device report installer hash mismatch."
Assert-Equal ([string]$report.candidate.portable.sha256) (Digest $portablePath) "Device report portable hash mismatch."
Assert-Equal ([string]$report.candidate.release_manifest_sha256) (Digest $manifestPath) "Device report release-manifest hash mismatch."
Assert-Equal ([string]$report.candidate.product_certification_sha256) (Digest $productCertificationPath) "Device report product-certification hash mismatch."
Assert-Equal ([string]$report.candidate.release_certification_sha256) (Digest $releaseCertificationPath) "Device report release-certification hash mismatch."

$temp = Join-Path $env:TEMP ("byte-device-report-verify-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temp -Force | Out-Null
try {
  Expand-Archive -Path $portablePath -DestinationPath $temp -Force
  $exe = Join-Path $temp "Byte.exe"
  if (!(Test-Path $exe)) {
    throw "Portable archive does not contain Byte.exe."
  }
  Assert-Equal ([string]$report.candidate.portable.executable_sha256) (Digest $exe) "Device report portable executable hash mismatch."
} finally {
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}

$osBuild = 0
[void][int]::TryParse([string]$report.device.os.build_number, [ref]$osBuild)
if ($osBuild -lt 22000) {
  throw "Physical device signoff must be performed on a supported Windows 11-class build (22000+)."
}
if ([string]$report.device.os.architecture -notmatch "64") {
  throw "Physical device signoff must use a 64-bit Windows installation."
}

$requiredManualGroups = @(
  "first_run_everyday",
  "companion",
  "lifecycle",
  "accessibility_visual",
  "notifications",
  "performance_observation"
)
foreach ($name in $requiredManualGroups) {
  if ($report.manual_checks.$name.status -ne "PASS") {
    throw "Required manual signoff group '$name' did not pass."
  }
}

$multiStatus = [string]$report.manual_checks.multimonitor_dpi.status
if ($multiStatus -notin @("PASS", "NA")) {
  throw "Multi-monitor/DPI signoff must be PASS or capability-backed NA."
}
if ($multiStatus -eq "NA" -and [int]$report.device.display_count -ge 2) {
  throw "Multi-monitor/DPI cannot be marked NA when two or more displays were detected."
}

$trustStatus = [string]$report.manual_checks.installer_trust.status
if ($trustStatus -notin @("PASS", "NA")) {
  throw "Installer/trust signoff must be PASS or unsigned-candidate NA."
}
if ($trustStatus -eq "NA" -and [bool]$manifest.signed) {
  throw "Installer/trust cannot be marked NA for a signed candidate."
}

$requiredStates = @("ACTIVE_CALM", "FULLSCREEN_REDUCED", "LOCKED_DISPLAY_OFF")
$performance = @($report.performance)
foreach ($state in $requiredStates) {
  $entry = @($performance | Where-Object { $_.state -eq $state })
  if ($entry.Count -ne 1) {
    throw "Physical device report must contain exactly one '$state' performance measurement."
  }

  $sample = $entry[0]
  if ([int]$sample.duration_seconds -lt 600) {
    throw "Performance state '$state' was measured for less than 10 minutes."
  }
  if ($sample.automatic_pass -ne $true) {
    throw "Performance state '$state' did not pass its automatic release targets."
  }

  if ([double]$sample.metrics.cpu_average_percent -gt 0.25) {
    throw "Performance state '$state' exceeds the 0.25% average CPU release target."
  }
  if ([double]$sample.metrics.working_set_average_mb -gt 60) {
    throw "Performance state '$state' exceeds the 60 MB average working-set release target."
  }
  if ($null -ne $sample.metrics.gpu_average_percent -and [double]$sample.metrics.gpu_average_percent -gt 1.0) {
    throw "Performance state '$state' exceeds the 1% average GPU release threshold."
  }
  if ($null -ne $sample.metrics.disk_write_average_bytes_per_sec -and [double]$sample.metrics.disk_write_average_bytes_per_sec -gt 4096) {
    throw "Performance state '$state' shows sustained disk writes above 4096 B/s."
  }
  if ($null -ne $sample.metrics.max_non_loopback_established_tcp_connections -and
      [int]$sample.metrics.max_non_loopback_established_tcp_connections -gt 0) {
    throw "Performance state '$state' observed a non-loopback established TCP connection owned by Byte."
  }
}

if ($report.device_ready -ne $true) {
  throw "Physical device report does not mark the tested device ready."
}

$gaps = @($report.capability_gaps)
if ($RequirePublicReleaseReady) {
  if ($report.public_release_ready -ne $true) {
    throw "Physical device report does not mark the candidate public-release ready."
  }
  if (-not [bool]$manifest.signed -or
      -not [bool]$manifest.signing.timestamped -or
      -not [bool]$manifest.signing.same_signer) {
    throw "Public-release device signoff requires signed, timestamped artifacts from the same signer."
  }
  if ($trustStatus -ne "PASS") {
    throw "Public-release device signoff requires installer/trust PASS."
  }
  if ($multiStatus -ne "PASS") {
    throw "Public-release device signoff requires multi-monitor/DPI PASS on at least a dual-display device."
  }
  if ($gaps.Count -ne 0) {
    throw "Public-release device signoff cannot contain capability gaps."
  }
}

Write-Host "Physical Windows device signoff verified for Byte v$version."
Write-Host "Device ready: $($report.device_ready)"
Write-Host "Public release ready: $($report.public_release_ready)"
