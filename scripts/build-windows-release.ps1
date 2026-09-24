$ErrorActionPreference = "Stop"

$arguments = @("run", "tauri:build", "--", "--bundles", "nsis")
if (Test-Path "src-tauri/tauri.signing.conf.json") {
  $arguments += @("--config", "src-tauri/tauri.signing.conf.json")
  Write-Host "Building signed NSIS release."
} else {
  Write-Host "Building unsigned NSIS release."
}

& npm @arguments
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}
