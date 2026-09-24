$ErrorActionPreference = "Stop"

$certificate = $env:WINDOWS_CERTIFICATE
$password = $env:WINDOWS_CERTIFICATE_PASSWORD
$timestampUrl = $env:WINDOWS_TIMESTAMP_URL

$values = @($certificate, $password, $timestampUrl)
$provided = ($values | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count

if ($provided -eq 0) {
  Write-Host "Windows signing secrets are not configured; producing an unsigned release."
  exit 0
}

if ($provided -ne $values.Count) {
  throw "Windows signing is partially configured. Provide WINDOWS_CERTIFICATE, WINDOWS_CERTIFICATE_PASSWORD, and WINDOWS_TIMESTAMP_URL together."
}

$pfxPath = Join-Path $env:RUNNER_TEMP "byte-code-signing.pfx"
$thumbprintPath = Join-Path $env:RUNNER_TEMP "byte-code-signing-thumbprint.txt"
$normalized = $certificate -replace "\s", ""
[IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($normalized))

$securePassword = ConvertTo-SecureString -String $password -AsPlainText -Force
$imported = Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation "Cert:\CurrentUser\My" -Password $securePassword

$signingCertificate = $imported | Where-Object { $_.HasPrivateKey } | Select-Object -First 1
if ($null -eq $signingCertificate) {
  throw "The imported PFX did not expose a certificate with a private key."
}

$thumbprint = $signingCertificate.Thumbprint
Set-Content $thumbprintPath $thumbprint -Encoding ascii

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

Write-Host "Optional Windows signing configured for this runner."
