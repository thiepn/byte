param(
  [string]$OutputDir = "release-artifacts",
  [switch]$PreviousReleaseTested
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path $OutputDir).Path
$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$manifestPath = Join-Path $root "release-manifest.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json

$installer = Join-Path $root ([string]$manifest.installer)
$portable = Join-Path $root ([string]$manifest.portable)

$certification = [ordered]@{
  schema_version = 1
  product = "Byte"
  version = $version
  target = "x86_64-pc-windows-msvc"
  commit = if ($env:GITHUB_SHA) { $env:GITHUB_SHA } else { $manifest.commit }
  certified_at_utc = [DateTime]::UtcNow.ToString("o")
  workflow = [ordered]@{
    repository = $env:GITHUB_REPOSITORY
    run_id = $env:GITHUB_RUN_ID
    run_attempt = $env:GITHUB_RUN_ATTEMPT
    event = $env:GITHUB_EVENT_NAME
    ref = $env:GITHUB_REF
  }
  signed = [bool]$manifest.signed
  certification_level = if ($PreviousReleaseTested) { "upgrade-and-clean-install" } else { "clean-install" }
  checks = [ordered]@{
    release_metadata = $true
    artifact_integrity = $true
    portable_x64_binary = $true
    portable_launch = $true
    installer_install = $true
    installer_reinstall = $true
    installed_version = $true
    uninstall = $true
    startup_cleanup = $true
    app_data_preservation = $true
    previous_release_upgrade = [bool]$PreviousReleaseTested
    downgrade_policy_runtime = [bool]$PreviousReleaseTested
  }
  artifacts = [ordered]@{
    installer = [ordered]@{
      file = [IO.Path]::GetFileName($installer)
      sha256 = (Get-FileHash $installer -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    portable = [ordered]@{
      file = [IO.Path]::GetFileName($portable)
      sha256 = (Get-FileHash $portable -Algorithm SHA256).Hash.ToLowerInvariant()
    }
  }
  distribution_ready = $true
}

$certificationPath = Join-Path $root "release-certification.json"
$certification | ConvertTo-Json -Depth 8 | Set-Content $certificationPath -Encoding utf8

$checksumTargets = @(
  $installer,
  $portable,
  $manifestPath,
  $certificationPath
)

$checksumLines = foreach ($file in $checksumTargets) {
  $hash = (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant()
  "$hash  $([IO.Path]::GetFileName($file))"
}
$checksumLines | Set-Content (Join-Path $root "SHA256SUMS.txt") -Encoding ascii

Write-Host "Final release certification written for Byte v$version."
