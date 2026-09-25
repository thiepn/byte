param(
  [string]$Executable = "src-tauri/target/release/Byte.exe",
  [int]$ObservationSeconds = 5
)

$ErrorActionPreference = "Stop"

$exe = (Resolve-Path $Executable).Path
$temp = Join-Path $env:RUNNER_TEMP ("byte-maintenance-launch-" + [guid]::NewGuid().ToString("N"))
$isolatedAppData = Join-Path $temp "appdata"
New-Item -ItemType Directory -Path $isolatedAppData -Force | Out-Null

$oldAppData = $env:APPDATA
$process = $null
try {
  $env:APPDATA = $isolatedAppData
  $process = Start-Process -FilePath $exe -PassThru
  Start-Sleep -Seconds $ObservationSeconds
  if ($process.HasExited) {
    throw "Byte exited during WebView2 maintenance smoke with code $($process.ExitCode)."
  }
  Write-Host "WebView2/runtime maintenance smoke passed for $env:RUNNER_OS."
} finally {
  $env:APPDATA = $oldAppData
  if ($null -ne $process -and -not $process.HasExited) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
  }
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
