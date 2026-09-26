param(
  [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\windows-signing.ps1")

$certificate = $env:WINDOWS_CERTIFICATE
$password = $env:WINDOWS_CERTIFICATE_PASSWORD
$timestampUrl = $env:WINDOWS_TIMESTAMP_URL

$values = @($certificate, $password, $timestampUrl)
$provided = @($values | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count

if ($provided -eq 0) {
  if ($RequireSigning) {
    throw "Public Byte releases require Windows Authenticode signing. Configure WINDOWS_CERTIFICATE, WINDOWS_CERTIFICATE_PASSWORD, and WINDOWS_TIMESTAMP_URL."
  }
  Write-Host "Windows signing secrets are not configured; producing an unsigned release candidate."
  exit 0
}

if ($provided -ne $values.Count) {
  throw "Windows signing is partially configured. Provide WINDOWS_CERTIFICATE, WINDOWS_CERTIFICATE_PASSWORD, and WINDOWS_TIMESTAMP_URL together."
}

$timestampUri = $null
if (-not [Uri]::TryCreate($timestampUrl, [UriKind]::Absolute, [ref]$timestampUri) -or
    ($timestampUri.Scheme -ne "http" -and $timestampUri.Scheme -ne "https")) {
  throw "WINDOWS_TIMESTAMP_URL must be an absolute HTTP or HTTPS timestamp-server URL."
}

$pfxPath = Join-Path $env:RUNNER_TEMP "byte-code-signing.pfx"
$thumbprintPath = Join-Path $env:RUNNER_TEMP "byte-code-signing-thumbprint.txt"
$metadataPath = Get-ByteSigningMetadataPath

try {
  $normalized = $certificate -replace "\s", ""
  [IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($normalized))
} catch {
  throw "WINDOWS_CERTIFICATE is not a valid base64-encoded PFX: $($_.Exception.Message)"
}

$securePassword = ConvertTo-SecureString -String $password -AsPlainText -Force
$imported = Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation "Cert:\CurrentUser\My" -Password $securePassword

$signingCertificate = $imported |
  Where-Object { $_.HasPrivateKey -and (Test-ByteCodeSigningEku -Certificate $_) } |
  Select-Object -First 1

if ($null -eq $signingCertificate) {
  throw "The imported PFX did not expose a certificate with a private key that is valid for code signing."
}

$now = [DateTime]::UtcNow
if ($signingCertificate.NotBefore.ToUniversalTime() -gt $now) {
  throw "The Windows signing certificate is not valid yet."
}
if ($signingCertificate.NotAfter.ToUniversalTime() -le $now) {
  throw "The Windows signing certificate has expired."
}

$daysRemaining = ($signingCertificate.NotAfter.ToUniversalTime() - $now).TotalDays
if ($daysRemaining -lt 30) {
  Write-Warning ("Windows signing certificate expires in {0:N0} days." -f $daysRemaining)
}

$thumbprint = ([string]$signingCertificate.Thumbprint).ToUpperInvariant()
Set-Content $thumbprintPath $thumbprint -Encoding ascii

$metadata = [ordered]@{
  schema_version = 1
  thumbprint = $thumbprint
  subject = [string]$signingCertificate.Subject
  issuer = [string]$signingCertificate.Issuer
  not_before_utc = $signingCertificate.NotBefore.ToUniversalTime().ToString("o")
  not_after_utc = $signingCertificate.NotAfter.ToUniversalTime().ToString("o")
  timestamp_url = $timestampUrl
}
$metadata | ConvertTo-Json -Depth 4 | Set-Content $metadataPath -Encoding utf8

$override = @{
  bundle = @{
    windows = @{
      certificateThumbprint = $thumbprint
      digestAlgorithm = "sha256"
      timestampUrl = $timestampUrl
    }
  }
}

$override | ConvertTo-Json -Depth 5 | Set-Content "src-tauri/tauri.signing.conf.json" -Encoding utf8

Write-Host "Windows signing configured for '$($signingCertificate.Subject)' ($thumbprint)."
