param()

$ErrorActionPreference = "Stop"

$files = @(
  "scripts/run-release-approval.ps1",
  "scripts/verify-release-approval.ps1",
  "scripts/test-release-approval-verifier.ps1"
)

foreach ($file in $files) {
  $tokens = $null
  $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path $file).Path,
    [ref]$tokens,
    [ref]$errors
  )

  if ($errors.Count -gt 0) {
    $detail = ($errors | ForEach-Object {
      "$($_.Extent.StartLineNumber):$($_.Extent.StartColumnNumber) $($_.Message)"
    }) -join [Environment]::NewLine
    throw ("PowerShell parse errors in " + $file + [Environment]::NewLine + $detail)
  }
}

$runner = Get-Content "scripts/run-release-approval.ps1" -Raw
$verifier = Get-Content "scripts/verify-release-approval.ps1" -Raw
$harness = Get-Content "scripts/test-release-approval-verifier.ps1" -Raw

$requiredRunnerFragments = @(
  'ValidateSet("signed", "unsigned")',
  'workflow", "run"',
  'run watch',
  'byte-device-signoff-candidate-',
  'RequirePublicReleaseReady',
  'approved_for_tagging',
  'automatic_tag_created = $false',
  'automatic_release_published = $false',
  'working tree is not clean',
  'local main',
  'VerifyOnly is side-effect-free',
  'Remote main moved during physical signoff',
  'More than one new workflow_dispatch run appeared',
  'thiepn/byte repository'
)
foreach ($fragment in $requiredRunnerFragments) {
  if (-not $runner.Contains($fragment)) {
    throw "Release approval runner is missing required invariant: $fragment"
  }
}

$forbiddenRunnerFragments = @(
  "git tag",
  "git push --tags",
  "gh release create",
  "gh release upload",
  "Publish GitHub Release"
)
foreach ($fragment in $forbiddenRunnerFragments) {
  if ($runner.Contains($fragment)) {
    throw "Release approval runner must not perform automatic tagging/publishing: $fragment"
  }
}

$requiredVerifierFragments = @(
  "RequireTaggingApproval",
  "public-release-pretag",
  'automatic_tag_created -ne $false',
  'automatic_release_published -ne $false',
  "device-report hash mismatch",
  "signed, timestamped artifacts from the same signer",
  "public-release-ready physical device report",
  "candidate-type mismatch"
)
foreach ($fragment in $requiredVerifierFragments) {
  if (-not $verifier.Contains($fragment)) {
    throw "Release approval verifier is missing required invariant: $fragment"
  }
}

foreach ($fragment in @(
  "device-report hash tampering",
  "automatic tag claim",
  "source commit tampering"
)) {
  if (-not $harness.Contains($fragment)) {
    throw "Release approval verifier harness is missing tamper case: $fragment"
  }
}

Write-Host "Release approval tooling syntax and policy harness passed."
