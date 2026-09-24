param(
  [Parameter(Mandatory = $true)]
  [string]$Installer,
  [string]$PreviousInstaller = ""
)

$ErrorActionPreference = "Stop"

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$expectedVersion = [string]$config.version

function Invoke-SilentInstaller([string]$Path) {
  Write-Host "Running installer: $Path"
  $process = Start-Process -FilePath $Path -ArgumentList "/S" -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    throw "Installer exited with code $($process.ExitCode): $Path"
  }
}

function Get-ByteUninstallEntry {
  $roots = @(
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
  )

  foreach ($root in $roots) {
    $entry = Get-ItemProperty $root -ErrorAction SilentlyContinue |
      Where-Object { $_.DisplayName -eq "Byte" } |
      Select-Object -First 1
    if ($null -ne $entry) {
      return $entry
    }
  }

  return $null
}

function Wait-ForByteInstall {
  for ($attempt = 0; $attempt -lt 20; $attempt++) {
    $entry = Get-ByteUninstallEntry
    if ($null -ne $entry) {
      return $entry
    }
    Start-Sleep -Milliseconds 500
  }
  throw "Byte did not register as installed."
}

function Normalize-RegistryPath([string]$Value) {
  if ([string]::IsNullOrWhiteSpace($Value)) {
    return ""
  }
  return $Value.Trim().Trim('"')
}

function Find-InstalledByte([object]$Entry) {
  $candidates = @()
  if ($Entry.InstallLocation) {
    $installLocation = Normalize-RegistryPath ([string]$Entry.InstallLocation)
    $candidates += Join-Path $installLocation "Byte.exe"
  }
  $candidates += Join-Path $env:LOCALAPPDATA "Byte\Byte.exe"

  foreach ($candidate in $candidates) {
    if (Test-Path $candidate) {
      return (Resolve-Path $candidate).Path
    }
  }

  throw "Installed Byte.exe could not be found."
}

function Get-UninstallerPath([object]$Entry) {
  if ($Entry.InstallLocation) {
    $installLocation = Normalize-RegistryPath ([string]$Entry.InstallLocation)
    $candidate = Join-Path $installLocation "uninstall.exe"
    if (Test-Path $candidate) {
      return $candidate
    }
  }

  $command = [string]$Entry.UninstallString
  if ($command -match '^"([^"]+)"') {
    return $matches[1]
  }
  if (-not [string]::IsNullOrWhiteSpace($command)) {
    return ($command -split "\s+")[0]
  }

  throw "Byte uninstaller path is unavailable."
}

function Assert-CurrentVersion([string]$ExePath) {
  $info = (Get-Item $ExePath).VersionInfo
  $reported = [string]$info.ProductVersion
  if ([string]::IsNullOrWhiteSpace($reported)) {
    $reported = [string]$info.FileVersion
  }
  if ([string]::IsNullOrWhiteSpace($reported) -or -not $reported.StartsWith($expectedVersion)) {
    throw "Installed Byte version '$reported' does not match expected version '$expectedVersion'."
  }
}

$installerPath = (Resolve-Path $Installer).Path
$previousPath = if ([string]::IsNullOrWhiteSpace($PreviousInstaller)) {
  ""
} else {
  (Resolve-Path $PreviousInstaller).Path
}

$dataDirectory = Join-Path $env:APPDATA ([string]$config.identifier)
$sentinelPath = Join-Path $dataDirectory ("release-certification-" + [guid]::NewGuid().ToString("N") + ".txt")
$sentinelValue = [guid]::NewGuid().ToString("N")
$installed = $false

try {
  if ($previousPath) {
    Invoke-SilentInstaller $previousPath
    $installed = $true
    $previousEntry = Wait-ForByteInstall
    [void](Find-InstalledByte $previousEntry)

    New-Item -ItemType Directory -Path $dataDirectory -Force | Out-Null
    Set-Content $sentinelPath $sentinelValue -Encoding ascii

    Invoke-SilentInstaller $installerPath
  } else {
    Invoke-SilentInstaller $installerPath
    $installed = $true

    New-Item -ItemType Directory -Path $dataDirectory -Force | Out-Null
    Set-Content $sentinelPath $sentinelValue -Encoding ascii

    Invoke-SilentInstaller $installerPath
  }

  if (!(Test-Path $sentinelPath) -or (Get-Content $sentinelPath -Raw).Trim() -ne $sentinelValue) {
    throw "Byte app-data sentinel did not survive reinstall/upgrade."
  }

  $installed = $true
  $entry = Wait-ForByteInstall
  $exePath = Find-InstalledByte $entry
  Assert-CurrentVersion $exePath

  if ($previousPath) {
    Write-Host "Attempting older installer to certify downgrade protection."
    $downgrade = Start-Process -FilePath $previousPath -ArgumentList "/S" -Wait -PassThru
    Write-Host "Older installer returned exit code $($downgrade.ExitCode)."
    $entry = Wait-ForByteInstall
    $exePath = Find-InstalledByte $entry
    Assert-CurrentVersion $exePath

    if (!(Test-Path $sentinelPath) -or (Get-Content $sentinelPath -Raw).Trim() -ne $sentinelValue) {
      throw "Byte app-data sentinel did not survive downgrade-policy certification."
    }
  }

  $runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
  New-Item $runKey -Force | Out-Null
  New-ItemProperty -Path $runKey -Name "Byte" -Value ('"' + $exePath + '"') -PropertyType String -Force | Out-Null

  $uninstaller = Get-UninstallerPath $entry
  $uninstallProcess = Start-Process -FilePath $uninstaller -ArgumentList "/S" -Wait -PassThru
  if ($uninstallProcess.ExitCode -ne 0) {
    throw "Uninstaller exited with code $($uninstallProcess.ExitCode)."
  }
  $installed = $false

  Start-Sleep -Seconds 1

  if ($null -ne (Get-ByteUninstallEntry)) {
    throw "Byte uninstall registration still exists after uninstall."
  }
  if (Test-Path $exePath) {
    throw "Byte.exe still exists after uninstall."
  }

  $runProperties = Get-ItemProperty -Path $runKey -ErrorAction SilentlyContinue
  if ($null -ne $runProperties -and $runProperties.PSObject.Properties.Name -contains "Byte") {
    throw "Byte startup registration survived uninstall."
  }

  if (!(Test-Path $sentinelPath) -or (Get-Content $sentinelPath -Raw).Trim() -ne $sentinelValue) {
    throw "Byte app data was removed or changed by uninstall."
  }

  Write-Host "Installer smoke certification passed, including app-data preservation."
} finally {
  if ($installed) {
    $entry = Get-ByteUninstallEntry
    if ($null -ne $entry) {
      try {
        $uninstaller = Get-UninstallerPath $entry
        Start-Process -FilePath $uninstaller -ArgumentList "/S" -Wait | Out-Null
      } catch {
        Write-Warning "Cleanup uninstall failed: $($_.Exception.Message)"
      }
    }
  }

  Remove-Item $sentinelPath -Force -ErrorAction SilentlyContinue
}
