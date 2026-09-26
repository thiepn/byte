param(
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [string]$Executable = "",
  [string]$Output = "",
  [int]$PerformanceMinutes = 10,
  [switch]$RequireSigning,
  [switch]$DevelopmentDryRun
)

$ErrorActionPreference = "Stop"

if ($PerformanceMinutes -lt 10 -and -not $DevelopmentDryRun) {
  throw "Physical device certification requires at least 10 minutes per performance state."
}
if ($PerformanceMinutes -lt 1) {
  throw "PerformanceMinutes must be at least 1."
}

$root = (Resolve-Path $CandidateDir).Path
$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version

if ([string]::IsNullOrWhiteSpace($Output)) {
  $documents = [Environment]::GetFolderPath([Environment+SpecialFolder]::MyDocuments)
  $reportDirectory = Join-Path $documents "Byte"
  New-Item -ItemType Directory -Path $reportDirectory -Force | Out-Null
  $Output = Join-Path $reportDirectory ("device-certification-v" + $version + ".json")
}

$verifyArgs = @{
  OutputDir = $root
  RequireCertification = $true
}
if ($RequireSigning) {
  $verifyArgs.RequireSigning = $true
}
& (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") @verifyArgs

$manifestPath = Join-Path $root "release-manifest.json"
$productCertificationPath = Join-Path $root "product-certification.json"
$releaseCertificationPath = Join-Path $root "release-certification.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$productCertification = Get-Content $productCertificationPath -Raw | ConvertFrom-Json
$releaseCertification = Get-Content $releaseCertificationPath -Raw | ConvertFrom-Json

if ([string]$manifest.version -ne $version -or [string]$releaseCertification.version -ne $version) {
  throw "Candidate metadata does not match Byte v$version."
}
if ($releaseCertification.checks.real_world_product_runtime -ne $true) {
  throw "The candidate has not passed automated P4 real-world product certification."
}

$temp = Join-Path $env:TEMP ("byte-device-signoff-" + [guid]::NewGuid().ToString("N"))
$extract = Join-Path $temp "portable"
New-Item -ItemType Directory -Path $extract -Force | Out-Null
$portablePath = Join-Path $root ([string]$manifest.portable)
Expand-Archive -Path $portablePath -DestinationPath $extract -Force
$portableExe = Join-Path $extract "Byte.exe"
if (!(Test-Path $portableExe)) {
  throw "Certified portable artifact does not contain Byte.exe."
}

$portableHash = (Get-FileHash $portableExe -Algorithm SHA256).Hash.ToLowerInvariant()
$targetExe = $portableExe

if (-not [string]::IsNullOrWhiteSpace($Executable)) {
  $targetExe = (Resolve-Path $Executable).Path
  $targetHash = (Get-FileHash $targetExe -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($targetHash -ne $portableHash) {
    throw "The supplied Byte executable is not byte-for-byte identical to the certified portable Byte.exe."
  }
}

function Get-DeviceInfo {
  $os = Get-CimInstance Win32_OperatingSystem
  $computer = Get-CimInstance Win32_ComputerSystem
  $processors = @(Get-CimInstance Win32_Processor)
  $gpus = @(Get-CimInstance Win32_VideoController | Where-Object { $_.Name })
  $screens = @()

  try {
    Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
    $screens = @([System.Windows.Forms.Screen]::AllScreens | ForEach-Object {
      [ordered]@{
        device_name = $_.DeviceName
        primary = $_.Primary
        bounds = [ordered]@{
          x = $_.Bounds.X
          y = $_.Bounds.Y
          width = $_.Bounds.Width
          height = $_.Bounds.Height
        }
        working_area = [ordered]@{
          x = $_.WorkingArea.X
          y = $_.WorkingArea.Y
          width = $_.WorkingArea.Width
          height = $_.WorkingArea.Height
        }
      }
    })
  } catch {
    $screens = @()
  }

  return [ordered]@{
    os = [ordered]@{
      caption = [string]$os.Caption
      version = [string]$os.Version
      build_number = [string]$os.BuildNumber
      architecture = [string]$os.OSArchitecture
    }
    manufacturer = [string]$computer.Manufacturer
    model = [string]$computer.Model
    physical_memory_gb = [Math]::Round([double]$computer.TotalPhysicalMemory / 1GB, 1)
    cpu = @($processors | ForEach-Object {
      [ordered]@{
        name = [string]$_.Name
        cores = [int]$_.NumberOfCores
        logical_processors = [int]$_.NumberOfLogicalProcessors
      }
    })
    gpu = @($gpus | ForEach-Object { [string]$_.Name })
    display_count = $screens.Count
    displays = $screens
  }
}

function Read-Result(
  [string]$GroupName,
  [string[]]$Items,
  [bool]$AllowNotApplicable = $false
) {
  Write-Host ""
  Write-Host "============================================================" -ForegroundColor Cyan
  Write-Host $GroupName -ForegroundColor Cyan
  Write-Host "============================================================" -ForegroundColor Cyan
  foreach ($item in $Items) {
    Write-Host (" - " + $item)
  }

  $choices = if ($AllowNotApplicable) { "PASS / FAIL / NA" } else { "PASS / FAIL" }
  while ($true) {
    $answer = (Read-Host "Result [$choices]").Trim().ToUpperInvariant()
    if ($answer -eq "PASS" -or $answer -eq "FAIL" -or ($AllowNotApplicable -and $answer -eq "NA")) {
      break
    }
  }

  $notes = Read-Host "Notes (optional)"
  return [ordered]@{
    status = $answer
    notes = $notes
    items = $Items
  }
}

function Get-ByteProcess([string]$ExePath) {
  $resolved = (Resolve-Path $ExePath).Path
  $matching = @(Get-Process -Name "Byte" -ErrorAction SilentlyContinue | Where-Object {
    try {
      $_.Path -and ((Resolve-Path $_.Path).Path -eq $resolved)
    } catch {
      $false
    }
  })

  if ($matching.Count -gt 1) {
    throw "Multiple matching Byte processes are running."
  }

  if ($matching.Count -eq 1) {
    return $matching[0]
  }

  $process = Start-Process -FilePath $resolved -PassThru
  Start-Sleep -Seconds 4
  if ($process.HasExited) {
    throw "Byte exited while starting the physical-device performance run."
  }
  return $process
}

function Get-ProcessTreeIds([int]$RootPid) {
  try {
    $rows = @(Get-CimInstance Win32_Process -ErrorAction Stop |
      Select-Object ProcessId, ParentProcessId)
  } catch {
    return @($RootPid)
  }

  $children = @{}
  foreach ($row in $rows) {
    $parent = [int]$row.ParentProcessId
    if (-not $children.ContainsKey($parent)) {
      $children[$parent] = [System.Collections.Generic.List[int]]::new()
    }
    $children[$parent].Add([int]$row.ProcessId)
  }

  $seen = [System.Collections.Generic.HashSet[int]]::new()
  $queue = [System.Collections.Generic.Queue[int]]::new()
  [void]$seen.Add($RootPid)
  $queue.Enqueue($RootPid)

  while ($queue.Count -gt 0) {
    $current = $queue.Dequeue()
    if (-not $children.ContainsKey($current)) { continue }
    foreach ($child in $children[$current]) {
      if ($seen.Add($child)) {
        $queue.Enqueue($child)
      }
    }
  }

  return @($seen | ForEach-Object { [int]$_ })
}

function Get-GpuPercentForPids([int[]]$Pids) {
  try {
    $counter = Get-Counter '\GPU Engine(*)\Utilization Percentage' -ErrorAction Stop
    $pidSet = [System.Collections.Generic.HashSet[int]]::new()
    foreach ($pidValue in $Pids) { [void]$pidSet.Add([int]$pidValue) }

    $total = 0.0
    $found = $false
    foreach ($sample in $counter.CounterSamples) {
      if ($sample.InstanceName -match 'pid_(\d+)_') {
        $samplePid = [int]$matches[1]
        if ($pidSet.Contains($samplePid)) {
          $total += [Math]::Max(0, [double]$sample.CookedValue)
          $found = $true
        }
      }
    }

    if (-not $found) { return $null }
    return $total
  } catch {
    return $null
  }
}

function Get-ProcessPerfDataForPids([int[]]$Pids) {
  try {
    $pidSet = [System.Collections.Generic.HashSet[int]]::new()
    foreach ($pidValue in $Pids) { [void]$pidSet.Add([int]$pidValue) }
    return @(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -ErrorAction Stop |
      Where-Object { $pidSet.Contains([int]$_.IDProcess) })
  } catch {
    return @()
  }
}

function Get-RemoteTcpCountForPids([int[]]$Pids) {
  try {
    $pidSet = [System.Collections.Generic.HashSet[int]]::new()
    foreach ($pidValue in $Pids) { [void]$pidSet.Add([int]$pidValue) }
    $connections = @(Get-NetTCPConnection -State Established -ErrorAction Stop |
      Where-Object {
        $pidSet.Contains([int]$_.OwningProcess) -and
        $_.RemoteAddress -notin @("127.0.0.1", "::1", "0.0.0.0", "::")
      })
    return $connections.Count
  } catch {
    return $null
  }
}

function Measure-ByteState(
  [string]$Name,
  [System.Diagnostics.Process]$Process,
  [int]$Minutes,
  [string]$Instruction
) {
  Write-Host ""
  Write-Host "Performance state: $Name" -ForegroundColor Yellow
  Write-Host $Instruction
  Read-Host "Press Enter when the state is ready"

  if ($Name -eq "LOCKED_DISPLAY_OFF") {
    Write-Host "Lock Windows or turn the display off within the next 15 seconds. The sampler will continue while the session is locked."
    Start-Sleep -Seconds 15
  }

  $durationSeconds = $Minutes * 60
  $logicalProcessors = [Math]::Max(1, [Environment]::ProcessorCount)
  $workingSet = [System.Collections.Generic.List[double]]::new()
  $privateMemory = [System.Collections.Generic.List[double]]::new()
  $cpu = [System.Collections.Generic.List[double]]::new()
  $diskWrites = [System.Collections.Generic.List[double]]::new()
  $gpu = [System.Collections.Generic.List[double]]::new()
  $remoteTcp = [System.Collections.Generic.List[int]]::new()
  $handlePeak = 0
  $threadPeak = 0
  $processTreePeak = 0
  $previousCpuByPid = @{}
  $previousAt = [DateTime]::UtcNow
  $startedAt = [DateTime]::UtcNow

  while (([DateTime]::UtcNow - $startedAt).TotalSeconds -lt $durationSeconds) {
    Start-Sleep -Seconds 1
    if ($Process.HasExited) {
      throw "Byte exited during physical-device performance state $Name."
    }

    $pids = @(Get-ProcessTreeIds $Process.Id)
    $processTreePeak = [Math]::Max($processTreePeak, $pids.Count)
    $processes = @()
    foreach ($pidValue in $pids) {
      try {
        $processes += Get-Process -Id $pidValue -ErrorAction Stop
      } catch {
        # A short-lived WebView2 helper can disappear between tree discovery and sampling.
      }
    }

    $now = [DateTime]::UtcNow
    $elapsed = [Math]::Max(0.001, ($now - $previousAt).TotalSeconds)
    $previousAt = $now
    $sampleCpuSeconds = 0.0
    $sampleWorkingSet = 0.0
    $samplePrivate = 0.0
    $sampleHandles = 0
    $sampleThreads = 0
    $livePidSet = [System.Collections.Generic.HashSet[int]]::new()

    foreach ($item in $processes) {
      $item.Refresh()
      $pidValue = [int]$item.Id
      [void]$livePidSet.Add($pidValue)
      $currentCpu = $item.TotalProcessorTime.TotalSeconds
      if ($previousCpuByPid.ContainsKey($pidValue)) {
        $sampleCpuSeconds += [Math]::Max(0, $currentCpu - [double]$previousCpuByPid[$pidValue])
      }
      $previousCpuByPid[$pidValue] = $currentCpu
      $sampleWorkingSet += [double]$item.WorkingSet64 / 1MB
      $samplePrivate += [double]$item.PrivateMemorySize64 / 1MB
      $sampleHandles += [int]$item.HandleCount
      $sampleThreads += [int]$item.Threads.Count
    }

    foreach ($pidKey in @($previousCpuByPid.Keys)) {
      if (-not $livePidSet.Contains([int]$pidKey)) {
        $previousCpuByPid.Remove($pidKey)
      }
    }

    $cpuPercent = ($sampleCpuSeconds / $elapsed / $logicalProcessors) * 100.0
    $workingSet.Add($sampleWorkingSet)
    $privateMemory.Add($samplePrivate)
    $cpu.Add([Math]::Max(0, $cpuPercent))
    $handlePeak = [Math]::Max($handlePeak, $sampleHandles)
    $threadPeak = [Math]::Max($threadPeak, $sampleThreads)

    $perfRows = @(Get-ProcessPerfDataForPids $pids)
    if ($perfRows.Count -gt 0) {
      $diskWrites.Add([double](($perfRows | Measure-Object IOWriteBytesPerSec -Sum).Sum))
    }

    if (($workingSet.Count % 5) -eq 0) {
      $gpuPercent = Get-GpuPercentForPids $pids
      if ($null -ne $gpuPercent) {
        $gpu.Add([double]$gpuPercent)
      }
      $remoteCount = Get-RemoteTcpCountForPids $pids
      if ($null -ne $remoteCount) {
        $remoteTcp.Add([int]$remoteCount)
      }
    }
  }

  $avgWorkingSet = if ($workingSet.Count) { ($workingSet | Measure-Object -Average).Average } else { 0 }
  $peakWorkingSet = if ($workingSet.Count) { ($workingSet | Measure-Object -Maximum).Maximum } else { 0 }
  $avgPrivate = if ($privateMemory.Count) { ($privateMemory | Measure-Object -Average).Average } else { 0 }
  $peakPrivate = if ($privateMemory.Count) { ($privateMemory | Measure-Object -Maximum).Maximum } else { 0 }
  $avgCpu = if ($cpu.Count) { ($cpu | Measure-Object -Average).Average } else { 0 }
  $peakCpu = if ($cpu.Count) { ($cpu | Measure-Object -Maximum).Maximum } else { 0 }
  $avgDiskWrite = if ($diskWrites.Count) { ($diskWrites | Measure-Object -Average).Average } else { $null }
  $avgGpu = if ($gpu.Count) { ($gpu | Measure-Object -Average).Average } else { $null }
  $maxRemoteTcp = if ($remoteTcp.Count) { ($remoteTcp | Measure-Object -Maximum).Maximum } else { $null }

  $thresholds = [ordered]@{
    cpu_average_percent_max = 0.25
    working_set_average_mb_max = 60
    gpu_average_percent_max = 1.0
    disk_write_average_bytes_per_sec_max = 4096
    remote_tcp_connections_max = 0
  }

  $checks = [ordered]@{
    cpu = ($avgCpu -le $thresholds.cpu_average_percent_max)
    working_set = ($avgWorkingSet -le $thresholds.working_set_average_mb_max)
    gpu = if ($null -eq $avgGpu) { $null } else { $avgGpu -le $thresholds.gpu_average_percent_max }
    disk_writes = if ($null -eq $avgDiskWrite) { $null } else { $avgDiskWrite -le $thresholds.disk_write_average_bytes_per_sec_max }
    remote_tcp = if ($null -eq $maxRemoteTcp) { $null } else { $maxRemoteTcp -le $thresholds.remote_tcp_connections_max }
  }

  $automaticPass =
    $checks.cpu -eq $true -and
    $checks.working_set -eq $true -and
    ($checks.gpu -ne $false) -and
    ($checks.disk_writes -ne $false) -and
    ($checks.remote_tcp -ne $false)

  Write-Host ("{0}: CPU avg {1:N3}% | working set avg {2:N1} MB | private avg {3:N1} MB" -f $Name, $avgCpu, $avgWorkingSet, $avgPrivate)
  if ($null -ne $avgGpu) { Write-Host ("GPU avg: {0:N3}%" -f $avgGpu) }
  if ($null -ne $avgDiskWrite) { Write-Host ("Disk writes avg: {0:N0} B/s" -f $avgDiskWrite) }
  if ($null -ne $maxRemoteTcp) { Write-Host ("Max non-loopback established TCP connections: {0}" -f $maxRemoteTcp) }

  return [ordered]@{
    state = $Name
    duration_seconds = $durationSeconds
    measured_at_utc = [DateTime]::UtcNow.ToString("o")
    metrics = [ordered]@{
      cpu_average_percent = [Math]::Round($avgCpu, 4)
      cpu_peak_percent = [Math]::Round($peakCpu, 4)
      working_set_average_mb = [Math]::Round($avgWorkingSet, 2)
      working_set_peak_mb = [Math]::Round($peakWorkingSet, 2)
      private_memory_average_mb = [Math]::Round($avgPrivate, 2)
      private_memory_peak_mb = [Math]::Round($peakPrivate, 2)
      handle_peak = $handlePeak
      thread_peak = $threadPeak
      process_tree_peak = $processTreePeak
      gpu_average_percent = if ($null -eq $avgGpu) { $null } else { [Math]::Round($avgGpu, 4) }
      disk_write_average_bytes_per_sec = if ($null -eq $avgDiskWrite) { $null } else { [Math]::Round($avgDiskWrite, 2) }
      max_non_loopback_established_tcp_connections = $maxRemoteTcp
    }
    thresholds = $thresholds
    checks = $checks
    automatic_pass = $automaticPass
  }
}

function Get-FileDigest([string]$Path) {
  return (Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

$device = Get-DeviceInfo
$manual = [ordered]@{}
$performance = @()
$capabilityGaps = [System.Collections.Generic.List[string]]::new()
$process = $null

try {
  Write-Host ""
  Write-Host "Byte P4-M Physical Windows Device Signoff" -ForegroundColor Green
  Write-Host "Candidate: Byte v$version"
  Write-Host "Source commit: $($manifest.commit)"
  Write-Host "Executable: $targetExe"
  Write-Host ""
  Write-Host "This tool records evidence; it does not alter a FAIL or untested capability into a PASS."

  $manual.first_run_everyday = Read-Result "1. First-run and everyday flow" @(
    "Clean install launches without elevation.",
    "Onboarding fits the window and is understandable without prior Byte knowledge.",
    "Back/Continue/cancel/rerun onboarding behavior is correct.",
    "Completing onboarding starts runtime once without duplicate workers.",
    "Close-to-tray, tray reopen, Quick Panel, main window, and duplicate-launch activation behave naturally.",
    "A normal 30–60 minute session shows no recurring intrusive UI, flicker, stuck overlays, or reaction spam."
  )

  $manual.companion = Read-Result "2. Companion behavior" @(
    "Habitat, Perch, Mini, Edge, and Tray modes behave correctly.",
    "Move Mode works and cannot become stuck.",
    "Companion interactions feel varied rather than repetitive.",
    "Diagnostic/critical behavior visibly overrides personality behavior.",
    "Reduced Motion removes extra movement without removing diagnostic meaning.",
    "Click-through does not intercept desktop clicks."
  )

  $manual.lifecycle = Read-Result "3. Fullscreen, presentation, lock, and sleep" @(
    "A real fullscreen game hides/restores Byte correctly.",
    "Fullscreen video behaves correctly.",
    "Presentation/slide-show suppression behaves correctly.",
    "Lock/unlock restores a fresh state.",
    "Display-off/wake restores a fresh state.",
    "System sleep/resume restores a fresh state.",
    "Repeated lifecycle transitions do not duplicate windows, workers, notifications, or tray icons."
  )

  $allowMonitorNa = ([int]$device.display_count -lt 2)
  $manual.multimonitor_dpi = Read-Result "4. Multi-monitor and DPI" @(
    "Byte can move between displays.",
    "Mixed Windows display scaling behaves correctly.",
    "Hot-unplug recovers Byte to an available monitor.",
    "Reconnect/rearrangement keeps saved placement bounded and usable.",
    "Quick Panel remains visible near work-area edges.",
    "Main app remains usable at minimum size."
  ) $allowMonitorNa
  if ($allowMonitorNa) {
    if ($manual.multimonitor_dpi.status -ne "NA") {
      throw "Multi-monitor/DPI cannot be marked PASS when fewer than two active displays are available."
    }
    $capabilityGaps.Add("multi-monitor-mixed-dpi: fewer than two active displays were available on this device")
  }

  $manual.accessibility_visual = Read-Result "5. Accessibility and visual QA" @(
    "Light and Dark themes are visually coherent.",
    "Windows forced colors / High Contrast remains usable.",
    "Byte High contrast remains usable.",
    "Windows and Byte reduced-motion paths remain usable.",
    "Keyboard-only onboarding/main/Apps/Studio/Settings/Quick Panel works.",
    "Visible focus and focus transfer are sensible.",
    "100%, 110%, 125%, 150%, 175%, and 200% scales have no clipped or unreachable core controls.",
    "Companion pixel art retains authored proportions."
  )

  $manual.notifications = Read-Result "6. Windows notifications" @(
    "Allowed notification permission behaves normally.",
    "Denied permission does not break Byte.",
    "Quiet mode and snooze work.",
    "Category switches work.",
    "Sustained actionable conditions use concise cause + next-step wording.",
    "Normal/transient workload does not create notification spam.",
    "In-app diagnostic truth remains available when native notifications are unavailable."
  )

  $trustNa = -not [bool]$manifest.signed
  $manual.installer_trust = Read-Result "7. Installer, update, and trust UX" @(
    "Installer presents the expected publisher.",
    "Authenticode is valid and timestamped.",
    "SmartScreen/trust presentation is consistent with the publisher.",
    "Reinstall/upgrade preserves state and downgrade remains blocked.",
    "Uninstall removes program/startup registration while preserving Byte data.",
    "Stable/Beta channel persists and Check for updates opens the fixed destination.",
    "Freshly downloaded assets pass checksum/signature/provenance verification."
  ) $trustNa
  if ($trustNa) {
    if ($manual.installer_trust.status -ne "NA") {
      throw "Installer/trust cannot be marked PASS for an unsigned candidate."
    }
    $capabilityGaps.Add("signed-installer-trust: candidate is unsigned and cannot certify public trust UX")
  }

  $process = Get-ByteProcess $targetExe

  $performance += Measure-ByteState "ACTIVE_CALM" $process $PerformanceMinutes @"
Put Byte in its normal Habitat configuration. Do not interact with Byte or run a deliberate heavy workload during the measurement.
"@

  $performance += Measure-ByteState "FULLSCREEN_REDUCED" $process $PerformanceMinutes @"
Open a real fullscreen game, fullscreen video, or presentation that should suppress/reduce Byte. Confirm Byte has entered the intended suppressed state before continuing.
"@

  $performance += Measure-ByteState "LOCKED_DISPLAY_OFF" $process $PerformanceMinutes @"
Prepare to lock Windows or turn the display off. After pressing Enter, you have 15 seconds to lock/blank the session. Leave the machine untouched for the measurement duration, then unlock it.
"@

  foreach ($sample in $performance) {
    if ($null -eq $sample.metrics.gpu_average_percent) {
      $capabilityGaps.Add("gpu-counter-unavailable:" + $sample.state)
    }
    if ($null -eq $sample.metrics.disk_write_average_bytes_per_sec) {
      $capabilityGaps.Add("disk-write-counter-unavailable:" + $sample.state)
    }
    if ($null -eq $sample.metrics.max_non_loopback_established_tcp_connections) {
      $capabilityGaps.Add("tcp-ownership-counter-unavailable:" + $sample.state)
    }
  }

  $manual.performance_observation = Read-Result "8. Performance observation" @(
    "No visible animation/render loop remains active while Byte should be suppressed.",
    "No recurring disk churn is visible outside normal event/config persistence.",
    "No Byte-generated outbound network traffic was observed.",
    "Sleep/wake and lock/unlock did not leave stale diagnostics or duplicate runtime components.",
    "The device remained responsive and Byte did not materially affect the workload."
  )

  $requiredManualGroups = @(
    "first_run_everyday",
    "companion",
    "lifecycle",
    "accessibility_visual",
    "notifications",
    "performance_observation"
  )
  $manualPass = $true
  foreach ($name in $requiredManualGroups) {
    if ($manual[$name].status -ne "PASS") {
      $manualPass = $false
    }
  }
  if ($manual.multimonitor_dpi.status -eq "FAIL") {
    $manualPass = $false
  }
  if ($manual.installer_trust.status -eq "FAIL") {
    $manualPass = $false
  }

  $performancePass = (@($performance | Where-Object { -not $_.automatic_pass }).Count -eq 0)
  $dryRun = [bool]$DevelopmentDryRun
  $deviceReady = $manualPass -and $performancePass -and -not $dryRun
  $publicReleaseReady =
    $deviceReady -and
    [bool]$manifest.signed -and
    [bool]$manifest.signing.timestamped -and
    [bool]$manifest.signing.same_signer -and
    $manual.installer_trust.status -eq "PASS" -and
    [int]$device.display_count -ge 2 -and
    $capabilityGaps.Count -eq 0

  $report = [ordered]@{
    schema_version = 1
    product = "Byte"
    version = $version
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
    certification_scope = "physical-windows-device-signoff"
    development_dry_run = $dryRun
    candidate = [ordered]@{
      source_commit = [string]$manifest.commit
      signed = [bool]$manifest.signed
      timestamped = [bool]$manifest.signing.timestamped
      same_signer = [bool]$manifest.signing.same_signer
      installer = [ordered]@{
        file = [string]$manifest.installer
        sha256 = Get-FileDigest (Join-Path $root ([string]$manifest.installer))
      }
      portable = [ordered]@{
        file = [string]$manifest.portable
        sha256 = Get-FileDigest $portablePath
        executable_sha256 = $portableHash
      }
      release_manifest_sha256 = Get-FileDigest $manifestPath
      product_certification_sha256 = Get-FileDigest $productCertificationPath
      release_certification_sha256 = Get-FileDigest $releaseCertificationPath
    }
    device = $device
    manual_checks = $manual
    performance = $performance
    capability_gaps = @($capabilityGaps)
    device_ready = $deviceReady
    public_release_ready = $publicReleaseReady
  }

  $outputPath = [IO.Path]::GetFullPath($Output)
  $outputParent = Split-Path $outputPath -Parent
  if (-not [string]::IsNullOrWhiteSpace($outputParent)) {
    New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
  }
  $report | ConvertTo-Json -Depth 20 | Set-Content $outputPath -Encoding utf8

  Write-Host ""
  Write-Host "Device signoff written to $outputPath" -ForegroundColor Green
  Write-Host "device_ready: $deviceReady"
  Write-Host "public_release_ready: $publicReleaseReady"
  if ($capabilityGaps.Count -gt 0) {
    Write-Host "Capability gaps:" -ForegroundColor Yellow
    foreach ($gap in $capabilityGaps) { Write-Host (" - " + $gap) }
  }

  if (-not $deviceReady) {
    throw "Physical device signoff did not pass. Review the generated report; FAIL/NA and failed performance checks are preserved."
  }
} finally {
  if ($null -ne $process -and -not $process.HasExited -and $targetExe -eq $portableExe) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
  }
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
