$ErrorActionPreference = "Continue"

$thumbprintPath = Join-Path $env:RUNNER_TEMP "byte-code-signing-thumbprint.txt"
$pfxPath = Join-Path $env:RUNNER_TEMP "byte-code-signing.pfx"

if (Test-Path $thumbprintPath) {
  $thumbprint = (Get-Content $thumbprintPath -Raw).Trim()
  if ($thumbprint) {
    Remove-Item "Cert:\CurrentUser\My\$thumbprint" -Force -ErrorAction SilentlyContinue
  }
}

Remove-Item $thumbprintPath -Force -ErrorAction SilentlyContinue
Remove-Item $pfxPath -Force -ErrorAction SilentlyContinue
Remove-Item "src-tauri/tauri.signing.conf.json" -Force -ErrorAction SilentlyContinue
