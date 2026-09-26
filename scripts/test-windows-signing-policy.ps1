$ErrorActionPreference = "Stop"

$scriptFiles = @(
  "scripts/lib/windows-signing.ps1",
  "scripts/prepare-windows-signing.ps1",
  "scripts/cleanup-windows-signing.ps1",
  "scripts/build-windows-release.ps1",
  "scripts/stage-release.ps1",
  "scripts/verify-release-artifacts.ps1",
  "scripts/finalize-release-certification.ps1",
  "scripts/test-windows-installer.ps1"
)

foreach ($file in $scriptFiles) {
  $tokens = $null
  $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path $file).Path,
    [ref]$tokens,
    [ref]$errors
  )
  if ($errors.Count -gt 0) {
    $messages = $errors | ForEach-Object { $_.Message }
    throw "PowerShell parse errors in ${file}: $($messages -join '; ')"
  }
}

. (Join-Path $PSScriptRoot "lib\windows-signing.ps1")
$signTool = Get-ByteSignToolPath
if (!(Test-Path $signTool)) {
  throw "signtool.exe discovery returned a missing path."
}
Write-Host "signtool.exe available at $signTool"

$names = @(
  "WINDOWS_CERTIFICATE",
  "WINDOWS_CERTIFICATE_PASSWORD",
  "WINDOWS_TIMESTAMP_URL"
)
$backup = @{}
foreach ($name in $names) {
  $backup[$name] = [Environment]::GetEnvironmentVariable($name)
  [Environment]::SetEnvironmentVariable($name, $null)
}

try {
  $failedClosed = $false
  try {
    & (Join-Path $PSScriptRoot "prepare-windows-signing.ps1") -RequireSigning
  } catch {
    if ($_.Exception.Message -notmatch "require Windows Authenticode signing") {
      throw
    }
    $failedClosed = $true
  }

  if (-not $failedClosed) {
    throw "Required signing unexpectedly succeeded without signing secrets."
  }
} finally {
  foreach ($name in $names) {
    [Environment]::SetEnvironmentVariable($name, $backup[$name])
  }
}

Write-Host "Windows signing policy smoke test passed."
