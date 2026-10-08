param(
  [string]$Step = "all",
  [int]$MonitorIndex = 0,
  [string]$OutputDir = (Join-Path ([Environment]::GetFolderPath("MyDocuments")) "Byte\P1-visual-evidence"),
  [switch]$List
)

$ErrorActionPreference = "Stop"

# User-operated physical-display screenshots only. Nothing is uploaded.
$steps = @(
  @{ Id="01-onboarding"; Instruction="Show first-run onboarding, step 1. Capture the real Byte window." },
  @{ Id="02-companion-byte"; Instruction="Show Byte in Habitat mode, neutral idle pose." },
  @{ Id="03-companion-mochi"; Instruction="Show Mochi in Habitat mode, neutral idle pose." },
  @{ Id="04-companion-pip"; Instruction="Show Pip in Habitat mode, neutral idle pose." },
  @{ Id="05-companion-kiwi"; Instruction="Show Kiwi in Habitat mode, neutral idle pose." },
  @{ Id="06-habitat-meadow"; Instruction="Show Meadow with a character and no extra decoration." },
  @{ Id="07-habitat-desk"; Instruction="Show Cozy Desk with a character and no extra decoration." },
  @{ Id="08-habitat-bedroom"; Instruction="Show Bedroom with a character and no extra decoration." },
  @{ Id="09-habitat-space"; Instruction="Show Space with a character and no extra decoration." },
  @{ Id="10-habitat-aquarium"; Instruction="Show Aquarium with a character and no extra decoration." },
  @{ Id="11-habitat-rooftop"; Instruction="Show Rooftop with a character and no extra decoration." },
  @{ Id="12-mode-habitat"; Instruction="Show Habitat display mode with enough desktop context for placement assessment." },
  @{ Id="13-mode-perch"; Instruction="Show Perch mode near the taskbar." },
  @{ Id="14-mode-mini"; Instruction="Show Mini mode at actual intended size." },
  @{ Id="15-mode-edge"; Instruction="Show Edge mode at the screen boundary." },
  @{ Id="16-mode-tray"; Instruction="Show Tray-only mode and tray menu." },
  @{ Id="17-quick-panel"; Instruction="Open the Quick Panel and ensure it is fully visible." },
  @{ Id="18-app-overview"; Instruction="Open Overview with available live status." },
  @{ Id="19-app-activity"; Instruction="Open Activity with realistic existing data or its empty state." },
  @{ Id="20-app-apps"; Instruction="Open Apps after an on-demand diagnostic scan, or its empty state." },
  @{ Id="21-app-customize"; Instruction="Open Customization Studio, character selection." },
  @{ Id="22-app-settings"; Instruction="Open Settings and show the main options." },
  @{ Id="23-studio-outfit"; Instruction="In Studio, open outfit editing with two or more accessories." },
  @{ Id="24-studio-habitat"; Instruction="In Studio, open habitat editing with decorations." },
  @{ Id="25-issue-state"; Instruction="Only capture a genuine issue/status warning if naturally available; otherwise skip." },
  @{ Id="26-reduced-motion"; Instruction="Show the companion with reduced motion enabled." },
  @{ Id="27-high-contrast"; Instruction="Show the main app with Byte high contrast enabled." },
  @{ Id="28-multi-monitor"; Instruction="Capture an actual secondary-display scenario if available; otherwise skip." }
)

if ($List) {
  $steps | ForEach-Object { Write-Host ($_.Id + ": " + $_.Instruction) }
  exit 0
}

if ($env:OS -ne "Windows_NT") { throw "Run this visual-evidence capture on a physical Windows desktop." }

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$monitors = [System.Windows.Forms.Screen]::AllScreens
if ($MonitorIndex -lt 0 -or $MonitorIndex -ge $monitors.Length) {
  throw "Invalid MonitorIndex. This machine has $($monitors.Length) screen(s)."
}
$selected = if ($Step -eq "all") { $steps } else { @($steps | Where-Object { $_.Id -eq $Step }) }
if ($selected.Count -eq 0) {
  throw "Unknown step '$Step'. Run with -List to see the available IDs."
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$indexPath = Join-Path $OutputDir "p1-evidence-index.json"
$existing = @()
if (Test-Path $indexPath) {
  $existing = @(Get-Content $indexPath -Raw | ConvertFrom-Json)
}

Write-Host "P1 screenshots are saved locally to: $OutputDir"
Write-Host "WARNING: a full-monitor capture may contain private application content."
Write-Host "Close private apps. If Byte is blank in captures, temporarily disable its capture-exclusion setting."
Write-Host "Restore your usual settings immediately after visual QA."
Write-Host "Screens: $($monitors.Length); chosen monitor: $MonitorIndex."

foreach ($item in $selected) {
  Write-Host ""
  Write-Host ("[" + $item.Id + "] " + $item.Instruction)
  $answer = Read-Host "Arrange the screen, then Enter=capture / s=skip / q=quit"
  if ($answer -eq "q") { break }
  if ($answer -eq "s") { continue }

  Start-Sleep -Milliseconds 300
  $bounds = $monitors[$MonitorIndex].Bounds
  $bitmap = [Drawing.Bitmap]::new($bounds.Width, $bounds.Height)
  $graphics = [Drawing.Graphics]::FromImage($bitmap)
  $filename = $item.Id + ".png"
  try {
    $graphics.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, $bitmap.Size)
    $bitmap.Save((Join-Path $OutputDir $filename), [Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $graphics.Dispose()
    $bitmap.Dispose()
  }

  $existing = @($existing | Where-Object { $_.step -ne $item.Id })
  $existing += [pscustomobject]@{
    step = $item.Id
    file = $filename
    captured_utc = (Get-Date).ToUniversalTime().ToString("o")
    monitor_index = $MonitorIndex
    width = $bounds.Width
    height = $bounds.Height
    capture_type = "physical-desktop-full-monitor"
    manually_inspected = $false
  }
  $existing | ConvertTo-Json -Depth 5 | Set-Content -Path $indexPath -Encoding utf8
  Write-Host ("Saved " + (Join-Path $OutputDir $filename) + " — inspect for blank Byte windows or private content.")
}

Write-Host "Done. Inspect every PNG and edit manually_inspected in the local index only after viewing the exact image."
Write-Host "Animation timings, feel, and interactive response require separate manual screen recordings and observation."
