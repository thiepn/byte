param()

$ErrorActionPreference = "Stop"

$files = @(
  "scripts/run-release-initiation.ps1",
  "scripts/verify-release-initiation.ps1"
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

$runner = Get-Content "scripts/run-release-initiation.ps1" -Raw
$verifier = Get-Content "scripts/verify-release-initiation.ps1" -Raw

$requiredRunner = @(
  "RequireTaggingApproval",
  "approved_for_tagging",
  "github_attestation_verified",
  "Get-SuccessfulWorkflowRun \"ci.yml\"",
  "Get-SuccessfulWorkflowRun \"package.yml\"",
  "check-release-order.mjs",
  "Assert-TagAndReleaseAbsent",
  "Assert-LocalTagState",
  "ConfirmTag",
  "Read-Host",
  "git push origin",
  ":refs/tags/",
  "ResumeExistingTag",
  "Wait-ForReleaseRun",
  "release.yml",
  "direct_release_publish_performed = $false",
  "direct_asset_upload_performed = $false",
  "P6 does not retry with force"
)
foreach ($fragment in $requiredRunner) {
  if (-not $runner.Contains($fragment)) {
    throw "P6 runner is missing required invariant: $fragment"
  }
}

$forbiddenRunner = @(
  "--force",
  "git push --force",
  "git tag -d",
  "git push origin --delete",
  "gh release create",
  "gh release upload",
  "release create",
  "release upload"
)
foreach ($fragment in $forbiddenRunner) {
  if ($runner.Contains($fragment)) {
    throw "P6 runner contains forbidden tag/release mutation: $fragment"
  }
}

$requiredVerifier = @(
  "RequireTaggingApproval",
  "controlled-tag-and-release-workflow-start",
  "P6 tags must be lightweight exact-commit tags",
  "P6 must never force-update a tag",
  "P6 must not directly publish a GitHub Release",
  "P6 must not directly upload release assets",
  "Byte Release",
  "release.yml",
  "RequireRemoteState",
  "RequireWorkflowSuccess",
  "object.type",
  "workflowName"
)
foreach ($fragment in $requiredVerifier) {
  if (-not $verifier.Contains($fragment)) {
    throw "P6 verifier is missing required invariant: $fragment"
  }
}

Write-Host "P6 controlled tagging/release-initiation tooling syntax and policy harness passed."
