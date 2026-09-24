param(
  [string]$OutputDir = "release-artifacts"
)

$ErrorActionPreference = "Stop"

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$releaseRoot = Join-Path (Get-Location) $OutputDir
$bundleRoot = Join-Path (Get-Location) "src-tauri/target/release/bundle/nsis"
$binaryPath = Join-Path (Get-Location) "src-tauri/target/release/Byte.exe"

if (!(Test-Path $bundleRoot)) {
  throw "NSIS bundle directory does not exist: $bundleRoot"
}
if (!(Test-Path $binaryPath)) {
  throw "Production binary does not exist: $binaryPath"
}

$installer = Get-ChildItem $bundleRoot -Filter "*-setup.exe" -File |
  Sort-Object LastWriteTimeUtc -Descending |
  Select-Object -First 1

if ($null -eq $installer) {
  throw "No NSIS setup executable was produced"
}

if (Test-Path $releaseRoot) {
  Remove-Item $releaseRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $releaseRoot | Out-Null

$installerName = "Byte-v$version-windows-x64-setup.exe"
$installerOut = Join-Path $releaseRoot $installerName
Copy-Item $installer.FullName $installerOut

$portableWork = Join-Path $env:RUNNER_TEMP "byte-portable"
if (Test-Path $portableWork) {
  Remove-Item $portableWork -Recurse -Force
}
New-Item -ItemType Directory -Path $portableWork | Out-Null
Copy-Item $binaryPath (Join-Path $portableWork "Byte.exe")

$portableName = "Byte-v$version-windows-x64-portable.zip"
$portableOut = Join-Path $releaseRoot $portableName
Compress-Archive -Path (Join-Path $portableWork "Byte.exe") -DestinationPath $portableOut -CompressionLevel Optimal

$binarySignature = Get-AuthenticodeSignature $binaryPath
$installerSignature = Get-AuthenticodeSignature $installerOut
$signed = $binarySignature.Status -eq "Valid" -and $installerSignature.Status -eq "Valid"

$manifest = [ordered]@{
  product = "Byte"
  version = $version
  identifier = [string]$config.identifier
  target = "x86_64-pc-windows-msvc"
  installer = $installerName
  portable = $portableName
  installer_scope = "currentUser"
  downgrades_allowed = $false
  signed = $signed
  commit = if ($env:GITHUB_SHA) { $env:GITHUB_SHA } else { $null }
}

$manifestPath = Join-Path $releaseRoot "release-manifest.json"
$manifest | ConvertTo-Json -Depth 5 | Set-Content $manifestPath -Encoding utf8

$checksumTargets = @($installerOut, $portableOut, $manifestPath)
$checksumLines = foreach ($file in $checksumTargets) {
  $hash = (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant()
  "$hash  $([IO.Path]::GetFileName($file))"
}
$checksumLines | Set-Content (Join-Path $releaseRoot "SHA256SUMS.txt") -Encoding ascii

Remove-Item $portableWork -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "Release artifacts staged in $releaseRoot"
Get-ChildItem $releaseRoot -File | ForEach-Object {
  Write-Host (" - {0} ({1:N0} bytes)" -f $_.Name, $_.Length)
}
