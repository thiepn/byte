param(
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [Parameter(Mandatory = $true)]
  [string]$DeviceReport,
  [Parameter(Mandatory = $true)]
  [string]$Approval,
  [switch]$RequireTaggingApproval
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path $CandidateDir).Path
$reportPath = (Resolve-Path $DeviceReport).Path
$approvalPath = (Resolve-Path $Approval).Path
$config = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$version = [string]$config.version

$manifestPath = Join-Path $root "release-manifest.json"
$productCertificationPath = Join-Path $root "product-certification.json"
$releaseCertificationPath = Join-Path $root "release-certification.json"

foreach ($required in @($manifestPath, $productCertificationPath, $releaseCertificationPath)) {
  if (!(Test-Path $required)) {
    throw "Release approval verification is missing candidate evidence: $required"
  }
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$report = Get-Content $reportPath -Raw | ConvertFrom-Json
$approval = Get-Content $approvalPath -Raw | ConvertFrom-Json

function Digest([string]$Path) {
  return (Get-FileHash $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-Equal([object]$Actual, [object]$Expected, [string]$Message) {
  if ($Actual -ne $Expected) {
    throw "$Message Expected '$Expected', got '$Actual'."
  }
}

$artifactVerifyArgs = @{
  OutputDir = $root
  RequireCertification = $true
}
if ([bool]$manifest.signed) {
  $artifactVerifyArgs.RequireSigning = $true
}
& (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") @artifactVerifyArgs

Assert-Equal $approval.schema_version 1 "Release approval schema mismatch."
Assert-Equal $approval.product "Byte" "Release approval product mismatch."
Assert-Equal ([string]$approval.version) $version "Release approval version mismatch."
if ([string]$approval.approval_scope -notin @("device-qa", "public-release-pretag")) {
  throw "Release approval scope is invalid."
}
Assert-Equal ([string]$approval.source.repository) "thiepn/byte" "Release approval repository mismatch."
Assert-Equal ([string]$approval.source.branch) "main" "Release approval branch mismatch."
Assert-Equal ([string]$approval.source.commit) ([string]$manifest.commit) "Release approval source commit mismatch."
Assert-Equal ([string]$approval.source.local_head) ([string]$manifest.commit) "Release approval local-head mismatch."

$installerPath = Join-Path $root ([string]$manifest.installer)
$portablePath = Join-Path $root ([string]$manifest.portable)

Assert-Equal ([string]$approval.candidate.installer.file) ([string]$manifest.installer) "Release approval installer filename mismatch."
Assert-Equal ([string]$approval.candidate.portable.file) ([string]$manifest.portable) "Release approval portable filename mismatch."
Assert-Equal ([string]$approval.candidate.installer.sha256) (Digest $installerPath) "Release approval installer hash mismatch."
Assert-Equal ([string]$approval.candidate.portable.sha256) (Digest $portablePath) "Release approval portable hash mismatch."
Assert-Equal ([string]$approval.candidate.release_manifest_sha256) (Digest $manifestPath) "Release approval manifest hash mismatch."
Assert-Equal ([string]$approval.candidate.product_certification_sha256) (Digest $productCertificationPath) "Release approval product-certification hash mismatch."
Assert-Equal ([string]$approval.candidate.release_certification_sha256) (Digest $releaseCertificationPath) "Release approval release-certification hash mismatch."
Assert-Equal ([bool]$approval.candidate.signed) ([bool]$manifest.signed) "Release approval signing-state mismatch."
Assert-Equal ([bool]$approval.candidate.timestamped) ([bool]$manifest.signing.timestamped) "Release approval timestamp-state mismatch."
Assert-Equal ([bool]$approval.candidate.same_signer) ([bool]$manifest.signing.same_signer) "Release approval same-signer mismatch."
$expectedCandidateType = if ([bool]$manifest.signed) { "signed" } else { "unsigned" }
Assert-Equal ([string]$approval.candidate.type) $expectedCandidateType "Release approval candidate-type mismatch."
$expectedAttestationState = [bool]$manifest.signed
Assert-Equal ([bool]$approval.candidate.github_attestation_verified) $expectedAttestationState "Release approval provenance-verification state mismatch."

Assert-Equal ([string]$approval.physical_device_report.file) ([IO.Path]::GetFileName($reportPath)) "Release approval device-report filename mismatch."
Assert-Equal ([string]$approval.physical_device_report.sha256) (Digest $reportPath) "Release approval device-report hash mismatch."
Assert-Equal ([bool]$approval.physical_device_report.device_ready) ([bool]$report.device_ready) "Release approval device-ready state mismatch."
Assert-Equal ([bool]$approval.physical_device_report.public_release_ready) ([bool]$report.public_release_ready) "Release approval public-ready state mismatch."
Assert-Equal ([string]$approval.physical_device_report.generated_at_utc) ([string]$report.generated_at_utc) "Release approval device-report timestamp mismatch."

$deviceVerifyArgs = @{
  CandidateDir = $root
  Report = $reportPath
}
if ([bool]$manifest.signed) {
  $deviceVerifyArgs.RequireSigning = $true
}
if ($RequireTaggingApproval -or [bool]$approval.approved_for_tagging) {
  $deviceVerifyArgs.RequirePublicReleaseReady = $true
}
& (Join-Path $PSScriptRoot "verify-physical-device-signoff.ps1") @deviceVerifyArgs

$expectedTagApproval =
  ([string]$approval.approval_scope -eq "public-release-pretag") -and
  [bool]$manifest.signed -and
  [bool]$manifest.signing.timestamped -and
  [bool]$manifest.signing.same_signer -and
  [bool]$report.public_release_ready

Assert-Equal ([bool]$approval.approved_for_tagging) ([bool]$expectedTagApproval) "Release approval tagging state is inconsistent with its evidence."

if ($approval.automatic_tag_created -ne $false) {
  throw "Release approval receipts must not claim an automatically created tag."
}
if ($approval.automatic_release_published -ne $false) {
  throw "Release approval receipts must not claim an automatically published release."
}

if ($RequireTaggingApproval) {
  if ($approval.approval_scope -ne "public-release-pretag") {
    throw "Tagging approval requires public-release-pretag scope."
  }
  if ($approval.approved_for_tagging -ne $true) {
    throw "Release approval receipt does not approve tagging."
  }
  if (-not [bool]$manifest.signed -or
      -not [bool]$manifest.signing.timestamped -or
      -not [bool]$manifest.signing.same_signer) {
    throw "Tagging approval requires signed, timestamped artifacts from the same signer."
  }
  if ($report.public_release_ready -ne $true) {
    throw "Tagging approval requires a public-release-ready physical device report."
  }
  if ($approval.candidate.github_attestation_verified -ne $true) {
    throw "Tagging approval requires successful GitHub provenance verification for the downloaded candidate."
  }
}

Write-Host "Release approval receipt verified for Byte v$version."
Write-Host "Source commit: $($manifest.commit)"
Write-Host "Approved for tagging: $($approval.approved_for_tagging)"
