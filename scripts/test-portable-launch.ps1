param(
  [Parameter(Mandatory = $true)]
  [string]$Portable,
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

  # Windows Error Reporting can be written a fraction of a second after the
  # process exits. Give it one bounded opportunity to land before inspecting
  # the local Application log.
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

$portablePath = (Resolve-Path $Portable).Path
$temp = Join-Path $env:RUNNER_TEMP ("byte-portable-launch-" + [guid]::NewGuid().ToString("N"))
$extract = Join-Path $temp "app"
$isolatedAppData = Join-Path $temp "appdata"
New-Item -ItemType Directory -Path $extract -Force | Out-Null
New-Item -ItemType Directory -Path $isolatedAppData -Force | Out-Null

$oldAppData = $env:APPDATA
$process = $null
$duplicateProcess = $null

try {
  Expand-Archive -Path $portablePath -DestinationPath $extract -Force
  $exe = Join-Path $extract "Byte.exe"
  if (!(Test-Path $exe)) {
    throw "Portable archive does not contain Byte.exe."
  }

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
      throw "Portable Byte.exe exited during launch certification attempt $attempt/$LaunchAttempts with code $($process.ExitCode) ($exitHex)."
    }

    Write-Host "Portable launch attempt $attempt/$LaunchAttempts passed; Byte remained alive for $ObservationSeconds seconds."
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    try { $process.WaitForExit(5000) | Out-Null } catch {}
    $process = $null
  }

  Write-Host "Portable repeat-start certification passed across $LaunchAttempts consecutive launches."

  $primaryStdoutPath = Join-Path $temp "byte.duplicate-primary.stdout.log"
  $primaryStderrPath = Join-Path $temp "byte.duplicate-primary.stderr.log"
  $duplicateStdoutPath = Join-Path $temp "byte.duplicate-secondary.stdout.log"
  $duplicateStderrPath = Join-Path $temp "byte.duplicate-secondary.stderr.log"
  $startedAt = Get-Date

  $process = Start-Process -FilePath $exe -PassThru `
    -RedirectStandardOutput $primaryStdoutPath `
    -RedirectStandardError $primaryStderrPath
  Start-Sleep -Seconds 2

  if ($process.HasExited) {
    Write-LaunchFailureDiagnostics `
      -Process $process `
      -StartedAt $startedAt `
      -StdoutPath $primaryStdoutPath `
      -StderrPath $primaryStderrPath
    throw "Primary Byte process exited before duplicate-launch certification."
  }

  $duplicateProcess = Start-Process -FilePath $exe -PassThru `
    -RedirectStandardOutput $duplicateStdoutPath `
    -RedirectStandardError $duplicateStderrPath

  if (-not $duplicateProcess.WaitForExit(5000)) {
    throw "Duplicate Byte process did not hand off to the existing instance within 5 seconds."
  }
  if ($duplicateProcess.ExitCode -ne 0) {
    throw "Duplicate Byte process exited with code $($duplicateProcess.ExitCode) instead of handing off cleanly."
  }

  Start-Sleep -Milliseconds 500
  if ($process.HasExited) {
    Write-LaunchFailureDiagnostics `
      -Process $process `
      -StartedAt $startedAt `
      -StdoutPath $primaryStdoutPath `
      -StderrPath $primaryStderrPath
    throw "Primary Byte process exited during duplicate-launch certification."
  }

  Write-Host "Portable duplicate-launch handoff passed; the second process exited and the original instance remained alive."
  Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
  try { $process.WaitForExit(5000) | Out-Null } catch {}
  $process = $null
  $duplicateProcess = $null
} finally {
  $env:APPDATA = $oldAppData
  if ($null -ne $duplicateProcess -and -not $duplicateProcess.HasExited) {
    Stop-Process -Id $duplicateProcess.Id -Force -ErrorAction SilentlyContinue
    try { $duplicateProcess.WaitForExit(5000) | Out-Null } catch {}
  }
  if ($null -ne $process -and -not $process.HasExited) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    try { $process.WaitForExit(5000) | Out-Null } catch {}
  }
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
