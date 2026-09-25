param(
  [string]$Executable = "src-tauri/target/release/Byte.exe",
  [int]$ObservationSeconds = 5,
  [int]$LaunchAttempts = 3
)

$ErrorActionPreference = "Stop"

if ($LaunchAttempts -lt 1) {
  throw "LaunchAttempts must be at least 1."
}

function Write-LaunchFailureDiagnostics(
  [System.Diagnostics.Process]$Process,
  [datetime]$StartedAt,
  [string]$StdoutPath,
  [string]$StderrPath
) {
  $exitBits = [System.BitConverter]::ToUInt32(
    [System.BitConverter]::GetBytes([int]$Process.ExitCode),
    0
  )
  $exitHex = "0x{0:X8}" -f $exitBits

  Write-Host "Byte launch failed with exit code $($Process.ExitCode) ($exitHex)."

  foreach ($stream in @(
    @{ Label = "stdout"; Path = $StdoutPath },
    @{ Label = "stderr"; Path = $StderrPath }
  )) {
    if (!(Test-Path $stream.Path)) {
      continue
    }

    $content = Get-Content $stream.Path -Raw -ErrorAction SilentlyContinue
    if ([string]::IsNullOrWhiteSpace($content)) {
      continue
    }

    Write-Host "----- Byte $($stream.Label) -----"
    Write-Host $content.Trim()
  }

  Start-Sleep -Seconds 1

  try {
    $events = Get-WinEvent -FilterHashtable @{
      LogName = "Application"
      StartTime = $StartedAt.AddSeconds(-2)
    } -ErrorAction Stop |
      Where-Object {
        $_.ProviderName -in @("Application Error", "Windows Error Reporting") -and
        $_.Message -match "(?i)Byte\.exe"
      } |
      Sort-Object TimeCreated -Descending |
      Select-Object -First 8

    if ($events) {
      Write-Host "----- Windows crash events -----"
      foreach ($event in $events) {
        $timestamp = if ($event.TimeCreated) {
          $event.TimeCreated.ToUniversalTime().ToString("o")
        } else {
          "unknown-time"
        }
        Write-Host "[$timestamp] $($event.ProviderName) / event $($event.Id)"
        Write-Host $event.Message
      }
    } else {
      Write-Host "No matching Application Error or Windows Error Reporting event was found."
    }
  } catch {
    Write-Warning "Could not query Windows crash events: $($_.Exception.Message)"
  }
}

$exe = (Resolve-Path $Executable).Path
$temp = Join-Path $env:RUNNER_TEMP ("byte-maintenance-launch-" + [guid]::NewGuid().ToString("N"))
$isolatedAppData = Join-Path $temp "appdata"
New-Item -ItemType Directory -Path $isolatedAppData -Force | Out-Null

$oldAppData = $env:APPDATA
$process = $null
try {
  $env:APPDATA = $isolatedAppData

  for ($attempt = 1; $attempt -le $LaunchAttempts; $attempt++) {
    $stdoutPath = Join-Path $temp ("byte.stdout." + $attempt + ".log")
    $stderrPath = Join-Path $temp ("byte.stderr." + $attempt + ".log")
    $startedAt = Get-Date
    $process = Start-Process -FilePath $exe -PassThru `
      -RedirectStandardOutput $stdoutPath `
      -RedirectStandardError $stderrPath
    Start-Sleep -Seconds $ObservationSeconds

    if ($process.HasExited) {
      Write-LaunchFailureDiagnostics `
        -Process $process `
        -StartedAt $startedAt `
        -StdoutPath $stdoutPath `
        -StderrPath $stderrPath

      $exitBits = [System.BitConverter]::ToUInt32(
        [System.BitConverter]::GetBytes([int]$process.ExitCode),
        0
      )
      $exitHex = "0x{0:X8}" -f $exitBits
      throw "Byte exited during WebView2 maintenance smoke attempt $attempt/$LaunchAttempts with code $($process.ExitCode) ($exitHex)."
    }

    Write-Host "Runtime launch attempt $attempt/$LaunchAttempts passed on $env:RUNNER_OS."
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    try { $process.WaitForExit(5000) | Out-Null } catch {}
    $process = $null
  }

  Write-Host "WebView2/runtime repeat-start smoke passed for $env:RUNNER_OS across $LaunchAttempts launches."
} finally {
  $env:APPDATA = $oldAppData
  if ($null -ne $process -and -not $process.HasExited) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    try { $process.WaitForExit(5000) | Out-Null } catch {}
  }
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
