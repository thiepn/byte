param()

$ErrorActionPreference = "Stop"

$files = @(
  "scripts/finalize-public-release-certification.ps1",
  "scripts/verify-public-release-certification.ps1",
  "scripts/run-release-completion.ps1",
  "scripts/verify-release-completion.ps1",
  "scripts/test-public-release-certification-verifier.ps1"
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

$finalizer = Get-Content "scripts/finalize-public-release-certification.ps1" -Raw
$publicVerifier = Get-Content "scripts/verify-public-release-certification.ps1" -Raw
$completionRunner = Get-Content "scripts/run-release-completion.ps1" -Raw
$completionVerifier = Get-Content "scripts/verify-release-completion.ps1" -Raw
$releaseWorkflow = Get-Content ".github/workflows/release.yml" -Raw
$publicHarness = Get-Content "scripts/test-public-release-certification-verifier.ps1" -Raw

foreach ($fragment in @(
  "verify-release-artifacts.ps1",
  "RequireCertification",
  "RequireSigning",
  "test-portable-launch.ps1",
  "test-windows-installer.ps1",
  "gh attestation verify",
  "releases/tags/",
  "releases/latest",
  "git/ref/tags/",
  "actions/runs/",
  "release_notes_match",
  "channel_metadata",
  "exact_lightweight_tag_target",
  "public_installer_lifecycle",
  "public_portable_launch",
  "public_distribution_ready = $true",
  "receipt_self_hashed = $false"
)) {
  if (-not $finalizer.Contains($fragment)) {
    throw "P7 public certification finalizer is missing required invariant: $fragment"
  }
}

foreach ($fragment in @(
  "public-release-certification.json",
  "seven expected P7 assets",
  "RequireRemoteState",
  "RequireWorkflowSuccess",
  "RecheckRuntime",
  "gh attestation verify",
  "Completed release tag must remain lightweight",
  "Byte Release",
  "public_distribution_ready"
)) {
  if (-not $publicVerifier.Contains($fragment)) {
    throw "P7 public certification verifier is missing required invariant: $fragment"
  }
}

foreach ($fragment in @(
  "verify-release-initiation.ps1",
  "RequireWorkflowSuccess",
  "gh release download",
  "public-release-certification.json",
  "verify-public-release-certification.ps1",
  "RecheckRuntime = $true",
  "release-completion-v",
  "tag_mutation_performed = $false",
  "release_mutation_performed = $false",
  "asset_upload_performed = $false",
  "complete = $true"
)) {
  if (-not $completionRunner.Contains($fragment)) {
    throw "P7 completion runner is missing required invariant: $fragment"
  }
}

foreach ($fragment in @(
  "unsigned candidate cannot qualify as a public release",
  "public asset hash tampering",
  "public_distribution_ready"
)) {
  if (-not $publicHarness.Contains($fragment)) {
    throw "P7 candidate-bound public verifier harness is missing required case: $fragment"
  }
}

foreach ($fragment in @(
  "verify-release-initiation.ps1",
  "verify-public-release-certification.ps1",
  "post-publish-release-closure",
  "successful completed Byte Release workflow",
  "P7 completion must not mutate the release tag",
  "P7 completion must not mutate the GitHub Release",
  "P7 completion must not upload release assets",
  "Release closure complete: True"
)) {
  if (-not $completionVerifier.Contains($fragment)) {
    throw "P7 completion verifier is missing required invariant: $fragment"
  }
}

foreach ($fragment in @(
  "gh release create",
  "gh release upload",
  "git push",
  "git tag",
  "git push --force",
  "--force",
  "gh release delete",
  "gh release edit"
)) {
  if ($completionRunner.Contains($fragment)) {
    throw "P7 local completion runner contains forbidden remote mutation: $fragment"
  }
}

$requiredWorkflowFragments = @(
  "Certify freshly published public release",
  "finalize-public-release-certification.ps1",
  "Publish public release certification",
  "public-release-certification.json",
  "Verify completed public release",
  "verify-public-release-certification.ps1",
  "Retain public release certification"
)
foreach ($fragment in $requiredWorkflowFragments) {
  if (-not $releaseWorkflow.Contains($fragment)) {
    throw "Byte Release workflow is missing required P7 post-publish gate: $fragment"
  }
}

$publishIndex = $releaseWorkflow.IndexOf("Publish GitHub Release")
$certifyIndex = $releaseWorkflow.IndexOf("Certify freshly published public release")
$certUploadIndex = $releaseWorkflow.IndexOf("Publish public release certification")
$verifyIndex = $releaseWorkflow.IndexOf("Verify completed public release")
$retainIndex = $releaseWorkflow.IndexOf("Retain public release certification")

if ($publishIndex -lt 0 -or
    $certifyIndex -le $publishIndex -or
    $certUploadIndex -le $certifyIndex -or
    $verifyIndex -le $certUploadIndex -or
    $retainIndex -le $verifyIndex) {
  throw "P7 post-publish workflow gates are missing or out of order."
}

Write-Host "P7 release completion/post-publish tooling syntax and policy harness passed."
