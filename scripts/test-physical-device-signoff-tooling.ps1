param()

$ErrorActionPreference = "Stop"

$files = @(
  "scripts/run-physical-device-signoff.ps1",
  "scripts/verify-physical-device-signoff.ps1"
)

foreach ($file in $files) {
  $tokens = $null
  $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path $file).Path,
    [ref]$tokens,
    [ref]$errors
  )

  if ($errors.Count -gt 0) {
    $detail = ($errors | ForEach-Object {
      "$($_.Extent.StartLineNumber):$($_.Extent.StartColumnNumber) $($_.Message)"
    }) -join [Environment]::NewLine
    throw ("PowerShell parse errors in " + $file + [Environment]::NewLine + $detail)
  }
}

$runner = Get-Content "scripts/run-physical-device-signoff.ps1" -Raw
$verifier = Get-Content "scripts/verify-physical-device-signoff.ps1" -Raw

$requiredRunnerFragments = @(
  "PerformanceMinutes -lt 10",
  "ACTIVE_CALM",
  "FULLSCREEN_REDUCED",
  "LOCKED_DISPLAY_OFF",
  "cpu_average_percent_max = 0.25",
  "working_set_average_mb_max = 60",
  "gpu_average_percent_max = 1.0",
  "remote_tcp_connections_max = 0",
  "process_tree_peak",
  "Get-ProcessTreeIds",
  "public_release_ready"
)
foreach ($fragment in $requiredRunnerFragments) {
  if (-not $runner.Contains($fragment)) {
    throw "Physical signoff runner is missing required policy fragment: $fragment"
  }
}

$requiredVerifierFragments = @(
  "duration_seconds -lt 600",
  "cpu_average_percent -gt 0.25",
  "working_set_average_mb -gt 60",
  "RequirePublicReleaseReady",
  "display_count -lt 2",
  "capability gaps",
  "portable executable hash mismatch"
)
foreach ($fragment in $requiredVerifierFragments) {
  if (-not $verifier.Contains($fragment)) {
    throw "Physical signoff verifier is missing required policy fragment: $fragment"
  }
}

Write-Host "Physical-device signoff tooling syntax and policy harness passed."
