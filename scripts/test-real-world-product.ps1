param(
  [Parameter(Mandatory = $true)]
  [string]$Portable,
  [string]$OutputDir = "release-artifacts",
  [int]$ProfileObservationSeconds = 6,
  [int]$LongSessionSeconds = 30
)

$ErrorActionPreference = "Stop"

if ($ProfileObservationSeconds -lt 3) {
  throw "ProfileObservationSeconds must be at least 3 seconds."
}
if ($LongSessionSeconds -lt 15) {
  throw "LongSessionSeconds must be at least 15 seconds."
}

$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version
$identifier = [string]$config.identifier
$configSource = Get-Content "src-tauri/src/core/config.rs" -Raw
$schemaMatch = [regex]::Match($configSource, 'CURRENT_SCHEMA_VERSION:\s*u32\s*=\s*(\d+)')
if (-not $schemaMatch.Success) {
  throw "Could not determine Byte's current configuration schema."
}
$currentSchema = [int]$schemaMatch.Groups[1].Value

$portablePath = (Resolve-Path $Portable).Path
$outputRoot = (Resolve-Path $OutputDir).Path
$temp = Join-Path $env:RUNNER_TEMP ("byte-real-world-" + [guid]::NewGuid().ToString("N"))
$extract = Join-Path $temp "app"
New-Item -ItemType Directory -Path $extract -Force | Out-Null
Expand-Archive -Path $portablePath -DestinationPath $extract -Force
$exe = Join-Path $extract "Byte.exe"
if (!(Test-Path $exe)) {
  throw "Portable archive does not contain Byte.exe."
}

$roamingAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::ApplicationData)
$appConfigRoot = Join-Path $roamingAppData $identifier
$appConfigBackup = Join-Path $temp "preexisting-byte-app-config"
$hadPreexistingAppConfig = Test-Path $appConfigRoot
if ($hadPreexistingAppConfig) {
  Copy-Item $appConfigRoot $appConfigBackup -Recurse -Force
}

$scenarioResults = [System.Collections.Generic.List[object]]::new()
$activeProcess = $null

function Reset-AppConfigRoot {
  if (Test-Path $appConfigRoot) {
    Remove-Item $appConfigRoot -Recurse -Force
  }
  New-Item -ItemType Directory -Path $appConfigRoot -Force | Out-Null
}

function Get-ConfigPath {
  return Join-Path $appConfigRoot "config.json"
}

function Copy-Config([object]$Value) {
  return ($Value | ConvertTo-Json -Depth 20 | ConvertFrom-Json)
}

function Write-Config([object]$Value) {
  $path = Get-ConfigPath
  $Value | ConvertTo-Json -Depth 20 | Set-Content $path -Encoding utf8
}

function Read-Config {
  $path = Get-ConfigPath
  if (!(Test-Path $path)) {
    throw "Byte did not create or preserve config.json in its Windows app_config_dir: $appConfigRoot."
  }
  return Get-Content $path -Raw | ConvertFrom-Json
}

function Write-ProcessDiagnostics(
  [System.Diagnostics.Process]$Process,
  [string]$StdoutPath,
  [string]$StderrPath
) {
  if ($null -ne $Process -and $Process.HasExited) {
    $bits = [System.BitConverter]::ToUInt32(
      [System.BitConverter]::GetBytes([int]$Process.ExitCode),
      0
    )
    Write-Host ("Byte exited with code {0} (0x{1:X8})." -f $Process.ExitCode, $bits)
  }

  foreach ($item in @(
    @{ Label = "stdout"; Path = $StdoutPath },
    @{ Label = "stderr"; Path = $StderrPath }
  )) {
    if (!(Test-Path $item.Path)) { continue }
    $content = Get-Content $item.Path -Raw -ErrorAction SilentlyContinue
    if ([string]::IsNullOrWhiteSpace($content)) { continue }
    Write-Host "----- Byte $($item.Label) -----"
    Write-Host $content.Trim()
  }
}

function Stop-ByteProcess([System.Diagnostics.Process]$Process) {
  if ($null -eq $Process) { return }
  if (-not $Process.HasExited) {
    Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
    try { $Process.WaitForExit(5000) | Out-Null } catch {}
  }
}

function Invoke-Scenario(
  [string]$Name,
  [int]$ObservationSeconds,
  [switch]$RunawayGuardrail
) {
  $stdoutPath = Join-Path $temp ($Name + ".stdout.log")
  $stderrPath = Join-Path $temp ($Name + ".stderr.log")
  $activeProcess = Start-Process -FilePath $exe -PassThru `
    -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath

  $peakWorkingSet = 0L
  $peakPrivate = 0L
  $peakHandles = 0
  $peakThreads = 0
  $cpuSeconds = 0.0

  try {
    $samples = [Math]::Max(2, $ObservationSeconds * 2)
    for ($index = 0; $index -lt $samples; $index++) {
      Start-Sleep -Milliseconds 500
      if ($activeProcess.HasExited) {
        Write-ProcessDiagnostics $activeProcess $stdoutPath $stderrPath
        throw "Byte exited during real-world scenario '$Name'."
      }

      $activeProcess.Refresh()
      $peakWorkingSet = [Math]::Max($peakWorkingSet, [int64]$activeProcess.WorkingSet64)
      $peakPrivate = [Math]::Max($peakPrivate, [int64]$activeProcess.PrivateMemorySize64)
      $peakHandles = [Math]::Max($peakHandles, [int]$activeProcess.HandleCount)
      $peakThreads = [Math]::Max($peakThreads, [int]$activeProcess.Threads.Count)
      $cpuSeconds = $activeProcess.TotalProcessorTime.TotalSeconds
    }

    if ($RunawayGuardrail) {
      $workingSetMb = [Math]::Round($peakWorkingSet / 1MB, 1)
      $privateMb = [Math]::Round($peakPrivate / 1MB, 1)
      $cpuLimit = [Math]::Max(12.0, $ObservationSeconds * 0.5)

      if ($workingSetMb -gt 512) {
        throw "Long-session Byte working set exceeded the 512 MB runaway guardrail: $workingSetMb MB."
      }
      if ($privateMb -gt 512) {
        throw "Long-session Byte private memory exceeded the 512 MB runaway guardrail: $privateMb MB."
      }
      if ($peakHandles -gt 2500) {
        throw "Long-session Byte handle count exceeded the 2500 runaway guardrail: $peakHandles."
      }
      if ($peakThreads -gt 300) {
        throw "Long-session Byte thread count exceeded the 300 runaway guardrail: $peakThreads."
      }
      if ($cpuSeconds -gt $cpuLimit) {
        throw "Long-session Byte CPU time exceeded the runaway guardrail: $cpuSeconds s > $cpuLimit s."
      }
    }

    $result = [ordered]@{
      name = $Name
      observation_seconds = $ObservationSeconds
      working_set_peak_mb = [Math]::Round($peakWorkingSet / 1MB, 1)
      private_memory_peak_mb = [Math]::Round($peakPrivate / 1MB, 1)
      handle_peak = $peakHandles
      thread_peak = $peakThreads
      cpu_seconds = [Math]::Round($cpuSeconds, 3)
      remained_alive = $true
    }
    $scenarioResults.Add([pscustomobject]$result)
    Write-Host ("Real-world scenario passed: {0} ({1}s, {2} MB peak working set)." -f $Name, $ObservationSeconds, $result.working_set_peak_mb)
  } finally {
    Stop-ByteProcess $activeProcess
    $activeProcess = $null
  }
}

try {
  # 1. Fresh install / first-run boundary.
  Reset-AppConfigRoot
  Invoke-Scenario "fresh-first-run" $ProfileObservationSeconds
  $baseline = Read-Config

  if ([int]$baseline.schema_version -ne $currentSchema) {
    throw "Fresh config schema mismatch: $($baseline.schema_version) != $currentSchema."
  }
  if ([bool]$baseline.app.onboarding_completed) {
    throw "Fresh Byte install unexpectedly bypassed onboarding."
  }
  if ([bool]$baseline.app.launch_at_startup) {
    throw "Fresh Byte install unexpectedly enabled launch at startup."
  }
  if ([int]$baseline.app.text_scale_percent -ne 100) {
    throw "Fresh Byte install did not use 100% interface scale."
  }

  # 2. Returning-user runtime matrix. Together these profiles cover every
  # display mode and every supported interface scale from 100 through 200%.
  $profiles = @(
    @{ Name = "habitat-scale100"; Mode = "HABITAT"; Scale = 100; Size = "MEDIUM"; HighContrast = $false; ReduceMotion = $false; Monitoring = $true },
    @{ Name = "perch-scale110"; Mode = "PERCH"; Scale = 110; Size = "SMALL"; HighContrast = $false; ReduceMotion = $false; Monitoring = $true },
    @{ Name = "mini-scale125"; Mode = "MINI"; Scale = 125; Size = "LARGE"; HighContrast = $false; ReduceMotion = $true; Monitoring = $true },
    @{ Name = "edge-scale150-high-contrast"; Mode = "EDGE"; Scale = 150; Size = "MEDIUM"; HighContrast = $true; ReduceMotion = $false; Monitoring = $true },
    @{ Name = "habitat-scale175-reduced"; Mode = "HABITAT"; Scale = 175; Size = "LARGE"; HighContrast = $false; ReduceMotion = $true; Monitoring = $true },
    @{ Name = "tray-scale200-monitoring-off"; Mode = "TRAY"; Scale = 200; Size = "SMALL"; HighContrast = $true; ReduceMotion = $true; Monitoring = $false }
  )

  foreach ($profile in $profiles) {
    Reset-AppConfigRoot
    $profileConfig = Copy-Config $baseline
    $profileConfig.app.onboarding_completed = $true
    $profileConfig.app.launch_at_startup = $false
    $profileConfig.app.notifications_enabled = $false
    $profileConfig.app.notification_quiet_mode = $true
    $profileConfig.app.text_scale_percent = [int]$profile.Scale
    $profileConfig.app.high_contrast = [bool]$profile.HighContrast
    $profileConfig.app.reduce_motion = [bool]$profile.ReduceMotion
    $profileConfig.app.system_monitoring_enabled = [bool]$profile.Monitoring
    $profileConfig.companion.display_mode = [string]$profile.Mode
    $profileConfig.companion.size = [string]$profile.Size
    Write-Config $profileConfig

    Invoke-Scenario $profile.Name $ProfileObservationSeconds
    $persisted = Read-Config

    if (-not [bool]$persisted.app.onboarding_completed) {
      throw "Returning-user profile '$($profile.Name)' lost onboarding completion."
    }
    if ([int]$persisted.app.text_scale_percent -ne [int]$profile.Scale) {
      throw "Returning-user profile '$($profile.Name)' changed interface scale."
    }
    if ([string]$persisted.companion.display_mode -ne [string]$profile.Mode) {
      throw "Returning-user profile '$($profile.Name)' changed display mode."
    }
    if ([bool]$persisted.app.system_monitoring_enabled -ne [bool]$profile.Monitoring) {
      throw "Returning-user profile '$($profile.Name)' changed monitoring preference."
    }
  }

  # 3. Runtime migration from the immediately preceding schema.
  Reset-AppConfigRoot
  $legacy = Copy-Config $baseline
  $legacy.schema_version = 8
  $legacy.app.onboarding_completed = $true
  $legacy.app.notifications_enabled = $false
  [void]$legacy.app.PSObject.Properties.Remove("update_channel")
  Write-Config $legacy

  Invoke-Scenario "schema-v8-runtime-migration" $ProfileObservationSeconds
  $migrated = Read-Config
  if ([int]$migrated.schema_version -ne $currentSchema) {
    throw "Runtime config migration did not reach schema $currentSchema."
  }
  if ([string]$migrated.app.update_channel -ne "STABLE") {
    throw "Runtime config migration did not restore the Stable update channel."
  }

  # 4. Corrupt persisted state must quarantine and recover during a real launch.
  Reset-AppConfigRoot
  $corruptPath = Get-ConfigPath
  Set-Content $corruptPath "{ definitely not valid Byte configuration" -Encoding utf8

  Invoke-Scenario "corrupt-config-recovery" $ProfileObservationSeconds
  $recovered = Read-Config
  if ([int]$recovered.schema_version -ne $currentSchema) {
    throw "Corrupt config recovery did not restore the current schema."
  }
  $quarantinePath = [IO.Path]::ChangeExtension($corruptPath, "corrupt.json")
  if (!(Test-Path $quarantinePath)) {
    throw "Corrupt config was not quarantined during runtime recovery."
  }

  # 5. Sustained returning-user session. The limits here are intentionally
  # runaway guardrails for CI, not claims about the tighter representative-PC
  # performance targets documented in docs/PERFORMANCE.md.
  Reset-AppConfigRoot
  $longConfig = Copy-Config $baseline
  $longConfig.app.onboarding_completed = $true
  $longConfig.app.launch_at_startup = $false
  $longConfig.app.notifications_enabled = $false
  $longConfig.app.notification_quiet_mode = $true
  $longConfig.app.text_scale_percent = 125
  $longConfig.app.system_monitoring_enabled = $true
  $longConfig.companion.display_mode = "HABITAT"
  Write-Config $longConfig
  Invoke-Scenario "sustained-session" $LongSessionSeconds -RunawayGuardrail

  $certification = [ordered]@{
    schema_version = 1
    product = "Byte"
    version = $version
    target = "x86_64-pc-windows-msvc"
    commit = if ($env:GITHUB_SHA) { $env:GITHUB_SHA } else { $null }
    certified_at_utc = [DateTime]::UtcNow.ToString("o")
    certification_scope = "automated-real-world-runtime-and-product-state"
    runner = [ordered]@{
      os = $env:RUNNER_OS
      image = $env:ImageOS
      repository = $env:GITHUB_REPOSITORY
      run_id = $env:GITHUB_RUN_ID
      run_attempt = $env:GITHUB_RUN_ATTEMPT
    }
    automated_checks = [ordered]@{
      fresh_first_run = $true
      onboarding_boundary = $true
      all_display_modes = $true
      interface_scales_100_through_200 = $true
      high_contrast_profile = $true
      reduced_motion_profile = $true
      monitoring_disabled_profile = $true
      runtime_schema_migration = $true
      corrupt_config_quarantine_and_recovery = $true
      sustained_runtime = $true
      runaway_resource_guardrails = $true
    }
    supported_interface_scales = @(100, 110, 125, 150, 175, 200)
    exercised_display_modes = @("HABITAT", "PERCH", "MINI", "EDGE", "TRAY")
    sustained_runtime_seconds = $LongSessionSeconds
    scenarios = @($scenarioResults)
    automated_ready = $true
    manual_device_signoff_required = $true
    manual_device_checks = @(
      "first-run interaction and onboarding completion UX",
      "fullscreen game/video/presentation suppression and restore",
      "lock, display-off, system sleep, and wake recovery",
      "multi-monitor placement, hot-unplug, rearrangement, and mixed Windows DPI",
      "Windows notification allow/deny behavior and notification quality",
      "representative-PC 10-minute CPU, memory, GPU, disk-write, and outbound-network measurements",
      "visual QA in light, dark, high contrast, reduced motion, and large interface scales",
      "signed installer, SmartScreen/trust presentation, update-link, reinstall, and uninstall UX"
    )
  }

  $outputPath = Join-Path $outputRoot "product-certification.json"
  $certification | ConvertTo-Json -Depth 12 | Set-Content $outputPath -Encoding utf8
  Write-Host "Automated real-world product certification passed for Byte v$version."
  Write-Host "Manual device signoff remains required before a public release is approved."
} finally {
  Stop-ByteProcess $activeProcess

  if (Test-Path $appConfigRoot) {
    Remove-Item $appConfigRoot -Recurse -Force -ErrorAction SilentlyContinue
  }
  if ($hadPreexistingAppConfig -and (Test-Path $appConfigBackup)) {
    Copy-Item $appConfigBackup $appConfigRoot -Recurse -Force
  }

  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
