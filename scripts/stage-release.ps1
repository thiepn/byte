param(
  [string]$OutputDir = "release-artifacts",
  [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\windows-signing.ps1")

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

$binarySignature = Get-ByteSignatureDetails -Path $binaryPath
$installerSignature = Get-ByteSignatureDetails -Path $installerOut
$binarySigned = [bool]$binarySignature.valid
$installerSigned = [bool]$installerSignature.valid

if ($binarySigned -ne $installerSigned) {
  throw "Windows signing is incomplete: Byte.exe and the NSIS installer must either both be signed or both be unsigned."
}

$signed = $binarySigned -and $installerSigned
$sameSigner = $signed -and ($binarySignature.signer_thumbprint -eq $installerSignature.signer_thumbprint)
$timestamped = $signed -and [bool]$binarySignature.timestamped -and [bool]$installerSignature.timestamped

if ($signed -and -not $sameSigner) {
  throw "Byte.exe and the NSIS installer were signed by different certificates."
}

$metadataPath = Get-ByteSigningMetadataPath
if (Test-Path $metadataPath) {
  $prepared = Get-Content $metadataPath -Raw | ConvertFrom-Json
  $expectedThumbprint = ([string]$prepared.thumbprint).ToUpperInvariant()
  if (-not $signed) {
    throw "Windows signing was prepared, but the staged Byte artifacts are not both validly signed."
  }
  if ($binarySignature.signer_thumbprint -ne $expectedThumbprint -or
      $installerSignature.signer_thumbprint -ne $expectedThumbprint) {
    throw "Staged artifacts do not match the certificate prepared for this build."
  }
}

if ($RequireSigning) {
  if (-not $signed) {
    throw "Public release staging requires valid Authenticode signatures on Byte.exe and the NSIS installer."
  }
  if (-not $timestamped) {
    throw "Public release staging requires timestamped Authenticode signatures on Byte.exe and the NSIS installer."
  }
}

$manifest = [ordered]@{
  schema_version = 2
  product = "Byte"
  version = $version
  identifier = [string]$config.identifier
  publisher = [string]$config.bundle.publisher
  homepage = [string]$config.bundle.homepage
  target = "x86_64-pc-windows-msvc"
  installer = $installerName
  portable = $portableName
  installer_scope = "currentUser"
  downgrades_allowed = $false
  signed = $signed
  commit = if ($env:GITHUB_SHA) { $env:GITHUB_SHA } else { $null }
  signing = [ordered]@{
    required_for_this_build = [bool]$RequireSigning
    same_signer = [bool]$sameSigner
    timestamped = [bool]$timestamped
    application = $binarySignature
    installer = $installerSignature
  }
}

$manifestPath = Join-Path $releaseRoot "release-manifest.json"
$manifest | ConvertTo-Json -Depth 8 | Set-Content $manifestPath -Encoding utf8

$checksumTargets = @($installerOut, $portableOut, $manifestPath)
$checksumLines = foreach ($file in $checksumTargets) {
  $hash = (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant()
  "$hash  $([IO.Path]::GetFileName($file))"
}
$checksumLines | Set-Content (Join-Path $releaseRoot "SHA256SUMS.txt") -Encoding ascii

Remove-Item $portableWork -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "Release artifacts staged in $releaseRoot"
Write-Host "Authenticode: signed=$signed timestamped=$timestamped same_signer=$sameSigner"
Get-ChildItem $releaseRoot -File | ForEach-Object {
  Write-Host (" - {0} ({1:N0} bytes)" -f $_.Name, $_.Length)
}
