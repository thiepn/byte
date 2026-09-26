function Get-ByteSigningMetadataPath {
  if ([string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)) {
    throw "RUNNER_TEMP is required for Byte signing metadata."
  }
  return (Join-Path $env:RUNNER_TEMP "byte-code-signing-metadata.json")
}

function Get-ByteSigningCertificate {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Thumbprint
  )

  $normalized = ($Thumbprint -replace "\s", "").ToUpperInvariant()
  $certificate = Get-Item "Cert:\CurrentUser\My\$normalized" -ErrorAction SilentlyContinue
  if ($null -eq $certificate) {
    throw "Byte signing certificate '$normalized' is not present in Cert:\CurrentUser\My."
  }
  if (-not $certificate.HasPrivateKey) {
    throw "Byte signing certificate '$normalized' does not expose a private key."
  }
  return $certificate
}

function Test-ByteCodeSigningEku {
  param(
    [Parameter(Mandatory = $true)]
    [System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate
  )

  foreach ($extension in $Certificate.Extensions) {
    if ($extension -is [System.Security.Cryptography.X509Certificates.X509EnhancedKeyUsageExtension]) {
      foreach ($usage in $extension.EnhancedKeyUsages) {
        if ($usage.Value -eq "1.3.6.1.5.5.7.3.3") {
          return $true
        }
      }
      return $false
    }
  }

  # An absent EKU extension means the certificate is not restricted to a
  # narrower purpose. Public code-signing certificates normally include the
  # Code Signing EKU, but do not reject an otherwise valid unrestricted cert.
  return $true
}

function Get-ByteSignatureDetails {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path
  )

  $resolved = (Resolve-Path $Path).Path
  $signature = Get-AuthenticodeSignature -FilePath $resolved
  $signer = $signature.SignerCertificate
  $timestamp = $signature.TimeStamperCertificate

  return [ordered]@{
    status = [string]$signature.Status
    status_message = [string]$signature.StatusMessage
    valid = $signature.Status -eq "Valid"
    signer_subject = if ($null -ne $signer) { [string]$signer.Subject } else { $null }
    signer_issuer = if ($null -ne $signer) { [string]$signer.Issuer } else { $null }
    signer_thumbprint = if ($null -ne $signer) { ([string]$signer.Thumbprint).ToUpperInvariant() } else { $null }
    signer_not_before_utc = if ($null -ne $signer) { $signer.NotBefore.ToUniversalTime().ToString("o") } else { $null }
    signer_not_after_utc = if ($null -ne $signer) { $signer.NotAfter.ToUniversalTime().ToString("o") } else { $null }
    timestamped = $null -ne $timestamp
    timestamp_subject = if ($null -ne $timestamp) { [string]$timestamp.Subject } else { $null }
    timestamp_thumbprint = if ($null -ne $timestamp) { ([string]$timestamp.Thumbprint).ToUpperInvariant() } else { $null }
  }
}

function Assert-ByteAuthenticodeSignature {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [string]$ExpectedThumbprint = "",
    [switch]$RequireTimestamp
  )

  $details = Get-ByteSignatureDetails -Path $Path
  if (-not $details.valid) {
    throw "Authenticode verification failed for '$Path': $($details.status) - $($details.status_message)"
  }

  if (-not [string]::IsNullOrWhiteSpace($ExpectedThumbprint)) {
    $expected = ($ExpectedThumbprint -replace "\s", "").ToUpperInvariant()
    if ($details.signer_thumbprint -ne $expected) {
      throw "Authenticode signer mismatch for '$Path'. Expected $expected, received $($details.signer_thumbprint)."
    }
  }

  if ($RequireTimestamp -and -not $details.timestamped) {
    throw "Authenticode signature for '$Path' is valid but is not timestamped."
  }

  return $details
}

function Get-ByteSignToolPath {
  $command = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
  if ($null -ne $command) {
    return $command.Source
  }

  $roots = @(
    (Join-Path ([Environment]::GetFolderPath("ProgramFilesX86")) "Windows Kits\10\bin"),
    (Join-Path ([Environment]::GetFolderPath("ProgramFiles")) "Windows Kits\10\bin")
  ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) -and (Test-Path $_) }

  $candidates = @()
  foreach ($root in $roots) {
    $candidates += Get-ChildItem $root -Filter "signtool.exe" -File -Recurse -ErrorAction SilentlyContinue |
      Where-Object { $_.DirectoryName -match "\\x64$" }
  }

  $selected = $candidates | Sort-Object FullName -Descending | Select-Object -First 1
  if ($null -eq $selected) {
    throw "signtool.exe was not found. Install the Windows SDK signing tools."
  }
  return $selected.FullName
}

function Invoke-ByteAuthenticodeSign {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [Parameter(Mandatory = $true)]
    [string]$Thumbprint,
    [Parameter(Mandatory = $true)]
    [string]$TimestampUrl
  )

  $resolved = (Resolve-Path $Path).Path
  $normalizedThumbprint = ($Thumbprint -replace "\s", "").ToUpperInvariant()
  $signTool = Get-ByteSignToolPath

  Write-Host "Signing $resolved with certificate $normalizedThumbprint."
  & $signTool sign /sha1 $normalizedThumbprint /s My /fd SHA256 /tr $TimestampUrl /td SHA256 /v $resolved
  if ($LASTEXITCODE -ne 0) {
    throw "signtool.exe failed to sign '$resolved' (exit code $LASTEXITCODE)."
  }

  & $signTool verify /pa /all /v $resolved
  if ($LASTEXITCODE -ne 0) {
    throw "signtool.exe failed to verify '$resolved' after signing (exit code $LASTEXITCODE)."
  }

  return Assert-ByteAuthenticodeSignature -Path $resolved -ExpectedThumbprint $normalizedThumbprint -RequireTimestamp
}
