param(
  [Parameter(Mandatory = $true)]
  [string]$CandidateDir,
  [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path -LiteralPath $CandidateDir).Path
$manifestPath = Join-Path $root "release-manifest.json"
$productPath = Join-Path $root "product-certification.json"
$releasePath = Join-Path $root "release-certification.json"
$checksumPath = Join-Path $root "SHA256SUMS.txt"

$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$version = [string]$manifest.version
$tag = "v$version"
$signed = [bool]$manifest.signed

$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$temp = Join-Path $tempRoot ("byte-p7-public-verifier-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temp -Force | Out-Null

function Digest([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Expect-Rejected([string]$Reason) {
  $rejected = $false
  try {
    & (Join-Path $PSScriptRoot "verify-public-release-certification.ps1") -PublishedDir $temp
  } catch {
    $rejected = $true
    Write-Host "Expected P7 verifier rejection ($Reason): $($_.Exception.Message)"
  }
  if (-not $rejected) {
    throw "P7 public release verifier accepted invalid evidence: $Reason"
  }
}

try {
  foreach ($file in Get-ChildItem -LiteralPath $root -File) {
    Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $temp $file.Name)
  }

  $baseAssets = @(
    [string]$manifest.installer,
    [string]$manifest.portable,
    "release-manifest.json",
    "product-certification.json",
    "release-certification.json",
    "SHA256SUMS.txt"
  )

  $assetEvidence = @()
  foreach ($name in ($baseAssets | Sort-Object)) {
    $path = Join-Path $temp $name
    if (!(Test-Path -LiteralPath $path)) {
      throw "P7 verifier harness is missing candidate asset '$name'."
    }
    $assetEvidence += [ordered]@{
      name = $name
      size_bytes = [long](Get-Item -LiteralPath $path).Length
      sha256 = Digest $path
      remote_digest = "sha256:" + (Digest $path)
    }
  }

  $isBeta = $version -match '-beta\.'
  $publicCert = [ordered]@{
    schema_version = 1
    product = "Byte"
    version = $version
    tag = $tag
    source_commit = [string]$manifest.commit
    certified_at_utc = [DateTime]::UtcNow.ToString("o")
    certification_scope = "fresh-public-download-post-publish"
    channel = if ($isBeta) { "beta" } else { "stable" }
    release = [ordered]@{
      database_id = 1
      tag_name = $tag
      name = "Byte $version"
      url = "https://github.com/thiepn/byte/releases/tag/$tag"
      draft = $false
      prerelease = [bool]$isBeta
      latest = -not [bool]$isBeta
      published_at = [DateTime]::UtcNow.ToString("o")
      target_commitish = "main"
      body_sha256 = ("0" * 64)
    }
    workflow = [ordered]@{
      run_id = 1
      run_number = 1
      run_attempt = 1
      name = "Byte Release"
      event = "push"
      head_sha = [string]$manifest.commit
      head_branch = $tag
      url = "https://github.com/thiepn/byte/actions/runs/1"
      observed_status = "in_progress"
      observed_conclusion = ""
    }
    tag_identity = [ordered]@{
      type = "lightweight"
      ref = "refs/tags/$tag"
      target_commit = [string]$manifest.commit
    }
    checks = [ordered]@{
      exact_asset_set = $true
      sha256_and_manifest = $true
      authenticode_and_timestamp = $true
      same_signer = $true
      release_certification_public_distribution_ready = $true
      github_provenance_installer = $true
      github_provenance_portable = $true
      public_portable_launch = $true
      public_installer_lifecycle = $true
      release_title_and_tag = $true
      release_notes_match = $true
      channel_metadata = $true
      exact_lightweight_tag_target = $true
    }
    assets = $assetEvidence
    release_manifest_sha256 = Digest (Join-Path $temp "release-manifest.json")
    product_certification_sha256 = Digest (Join-Path $temp "product-certification.json")
    release_certification_sha256 = Digest (Join-Path $temp "release-certification.json")
    checksum_file_sha256 = Digest (Join-Path $temp "SHA256SUMS.txt")
    signing = [ordered]@{
      signed = [bool]$manifest.signed
      timestamped = [bool]$manifest.signing.timestamped
      same_signer = [bool]$manifest.signing.same_signer
      application_signer_thumbprint = [string]$manifest.signing.application.signer_thumbprint
      installer_signer_thumbprint = [string]$manifest.signing.installer.signer_thumbprint
    }
    public_distribution_ready = $true
    receipt_self_hashed = $false
  }

  $certPath = Join-Path $temp "public-release-certification.json"
  $publicCert | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $certPath -Encoding utf8

  if (-not $signed) {
    if ($RequireSigning) {
      throw "P7 signed-path verifier harness was requested for an unsigned candidate."
    }
    Expect-Rejected "unsigned candidate cannot qualify as a public release"
    Write-Host "P7 public release verifier correctly rejected the unsigned candidate."
    exit 0
  }

  if (-not [bool]$manifest.signing.timestamped -or -not [bool]$manifest.signing.same_signer) {
    throw "Signed P7 verifier harness requires timestamped same-signer candidate artifacts."
  }

  $releaseCertification = Get-Content -LiteralPath $releasePath -Raw | ConvertFrom-Json
  if ($releaseCertification.public_distribution_ready -ne $true) {
    throw "Signed P7 verifier harness requires public_distribution_ready release certification."
  }

  & (Join-Path $PSScriptRoot "verify-public-release-certification.ps1") -PublishedDir $temp

  $tampered = Get-Content -LiteralPath $certPath -Raw | ConvertFrom-Json
  $tampered.assets[0].sha256 = ("0" * 64)
  $tampered | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $certPath -Encoding utf8
  Expect-Rejected "public asset hash tampering"

  Write-Host "P7 public release verifier signed candidate-bound harness passed."
} finally {
  Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
