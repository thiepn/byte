param(
  [Parameter(Mandatory = $true)]
  [string]$Portable,
  [int]$ObservationSeconds = 5
)

$ErrorActionPreference = "Stop"

$portablePath = (Resolve-Path $Portable).Path
$temp = Join-Path $env:RUNNER_TEMP ("byte-portable-launch-" + [guid]::NewGuid().ToString("N"))
$extract = Join-Path $temp "app"
$isolatedAppData = Join-Path $temp "appdata"
New-Item -ItemType Directory -Path $extract -Force | Out-Null
New-Item -ItemType Directory -Path $isolatedAppData -Force | Out-Null

$oldAppData = $env:APPDATA
$process = $null

try {
  Expand-Archive -Path $portablePath -DestinationPath $extract -Force
  $exe = Join-Path $extract "Byte.exe"
  if (!(Test-Path $exe)) {
    throw "Portable archive does not contain Byte.exe."
  }

  $env:APPDATA = $isolatedAppData
  $process = Start-Process -FilePath $exe -PassThru
  Start-Sleep -Seconds $ObservationSeconds

  if ($process.HasExited) {
    throw "Portable Byte.exe exited during launch certification with code $($process.ExitCode)."
  }

  Write-Host "Portable launch smoke passed; Byte remained alive for $ObservationSeconds seconds."
} finally {
  $env:APPDATA = $oldAppData
  if ($null -ne $process -and -not $process.HasExited) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    try { $process.WaitForExit(5000) | Out-Null } catch {}
  }
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
