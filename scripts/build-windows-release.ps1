param(
  [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\windows-signing.ps1")

$signingConfig = "src-tauri/tauri.signing.conf.json"
$signingConfigured = Test-Path $signingConfig

if ($RequireSigning -and -not $signingConfigured) {
  throw "This build requires Windows signing, but no signing configuration was prepared."
}

$arguments = @("run", "tauri:build", "--", "--bundles", "nsis")
if ($signingConfigured) {
  $arguments += @("--config", $signingConfig)
  Write-Host "Building signed NSIS release."
} else {
  Write-Host "Building unsigned NSIS release candidate."
}

& npm @arguments
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}

if (-not $signingConfigured) {
  return
}

$metadataPath = Get-ByteSigningMetadataPath
if (!(Test-Path $metadataPath)) {
  throw "Signing configuration exists but Byte signing metadata is missing."
}
$metadata = Get-Content $metadataPath -Raw | ConvertFrom-Json
$thumbprint = [string]$metadata.thumbprint
$timestampUrl = [string]$metadata.timestamp_url

$binaryPath = Join-Path (Get-Location) "src-tauri\target\release\Byte.exe"
if (!(Test-Path $binaryPath)) {
  throw "Production Byte.exe is missing after the Tauri build."
}

# Tauri signs the bundle-specific patched executable, then restores the original
# main binary after bundling. Byte's portable ZIP is staged from target/release,
# so explicitly sign that restored binary as part of the trust chain.
Invoke-ByteAuthenticodeSign -Path $binaryPath -Thumbprint $thumbprint -TimestampUrl $timestampUrl | Out-Null

$bundleRoot = Join-Path (Get-Location) "src-tauri\target\release\bundle\nsis"
$installer = Get-ChildItem $bundleRoot -Filter "*-setup.exe" -File |
  Sort-Object LastWriteTimeUtc -Descending |
  Select-Object -First 1

if ($null -eq $installer) {
  throw "No NSIS installer was produced."
}

Assert-ByteAuthenticodeSignature -Path $installer.FullName -ExpectedThumbprint $thumbprint -RequireTimestamp | Out-Null
Write-Host "Signed build verification passed for portable Byte.exe and NSIS installer."
