# Byte Windows Trust & Code Signing

Byte's Windows distribution has two independent trust layers:

1. **Authenticode** — Windows verifies who signed the executable and whether the signed bytes changed.
2. **GitHub artifact provenance** — GitHub attestations link the downloadable installer/portable archive to this repository, workflow, and source commit.

SHA-256 checksums remain a third integrity check for users who want to compare exact file hashes.

## Why v0.1.0 shows SmartScreen

Byte v0.1.0 was released before public-release signing became mandatory. Its installer is structurally certified and has GitHub provenance, but it is not Authenticode-signed.

Windows SmartScreen can therefore present an **unknown publisher / unrecognized app** warning. That warning is about publisher identity and reputation; it is not, by itself, a malware finding.

P1 changes the policy for subsequent public releases: tagged Stable and Beta releases must be signed and timestamped or the release workflow fails before publication.

## Public release invariant

Every future tagged public release requires:

- a valid Authenticode signature on the NSIS installer
- a valid Authenticode signature on the standalone portable `Byte.exe`
- the same signer certificate on both
- a trusted timestamp on both
- release-manifest signer metadata matching the binaries
- installer lifecycle certification proving the installed `Byte.exe` has the same signer
- GitHub provenance attestations for installer and portable ZIP
- a fresh re-download of the published release followed by another trust/integrity verification

Ordinary pull-request and `main` packaging may remain unsigned so contributors and CI do not receive production signing credentials.

## Repository secrets

The tagged release workflow expects all three secrets together:

- `WINDOWS_CERTIFICATE` — the base64-encoded PFX/PKCS#12 file
- `WINDOWS_CERTIFICATE_PASSWORD` — password protecting that PFX
- `WINDOWS_TIMESTAMP_URL` — timestamp service URL supplied by the certificate provider

If none are configured, ordinary release-candidate packaging remains unsigned.

For a tagged public release, missing secrets are a hard failure. Partially configured secrets are always a hard failure.

### Preparing the PFX secret

On a trusted local Windows machine:

```powershell
$pfx = [IO.File]::ReadAllBytes(".\byte-code-signing.pfx")
[Convert]::ToBase64String($pfx) | Set-Clipboard
```

Paste that base64 value into the repository secret `WINDOWS_CERTIFICATE`. Store the PFX password separately in `WINDOWS_CERTIFICATE_PASSWORD`.

Never commit:

- the PFX
- its base64 representation
- its password
- exported private-key material

## Certificate requirements

The signing certificate must:

- contain a private key
- be currently valid
- permit code signing, or be unrestricted by EKU
- chain successfully when Windows verifies the produced signature

The P1 preparation step rejects certificates that are not yet valid, expired, missing a private key, or explicitly restricted away from code signing.

The workflow emits a warning when the certificate has fewer than 30 days remaining.

## Publisher identity

Byte's package metadata uses:

- publisher: `THIEPN`
- homepage: `https://thiepn.dev/byte/`

That metadata improves Windows application/installer presentation, but it does **not** create a verified publisher identity.

The publisher shown by Windows in Authenticode/SmartScreen UI comes from the actual certificate subject issued by the certificate authority. That subject must be truthful and will be recorded in `release-manifest.json`.

## Portable executable signing

Tauri signs the bundle-specific executable during packaging. Its bundler can restore the original main executable after a bundle target is created.

Byte stages its portable ZIP from:

`src-tauri/target/release/Byte.exe`

P1 therefore explicitly signs that restored production binary after the Tauri bundle finishes. The portable archive is created only after this extra signing step.

This prevents a signed installer from being published next to an unsigned portable executable.

## Timestamping

Timestamping is mandatory for public Byte releases.

A timestamp proves that the artifact was signed while the signing certificate was valid. This allows an existing valid signature to remain verifiable after the signing certificate itself later expires.

The release gate checks that both the installer and portable executable expose a timestamper certificate through Windows Authenticode verification.

## Release manifest

Manifest schema v2 records:

- whether signing was required for that build
- whether both artifacts are validly signed
- whether both use the same signer
- whether both signatures are timestamped
- signer subject
- signer issuer
- signer certificate thumbprint
- signer validity period
- timestamper subject/thumbprint
- Byte publisher/homepage metadata

The certificate private key is never recorded.

## Release certification

Certification schema v2 distinguishes:

- `distribution_ready` — package/runtime/integrity certification succeeded
- `public_distribution_ready` — the build also satisfies Byte's signed-public-release trust policy

Unsigned PR/main artifacts can therefore remain useful certified candidates without being misrepresented as public-release-ready.

Tagged release workflows require `public_distribution_ready: true`.

## Installed-binary verification

For a signed tagged release, the real NSIS lifecycle test:

1. validates the installer signature and timestamp
2. installs Byte
3. locates the installed `Byte.exe`
4. verifies its version
5. verifies the installed executable is signed by the same certificate as the installer
6. repeats that signer verification after upgrade/downgrade-policy testing

This verifies the binary users actually run, not only the files in the build directory.

## User verification

### Windows Authenticode

```powershell
Get-AuthenticodeSignature .\Byte-vX.Y.Z-windows-x64-setup.exe |
  Format-List Status,StatusMessage,SignerCertificate,TimeStamperCertificate
```

For a signed public release, `Status` should be `Valid`.

For the portable build:

```powershell
Expand-Archive .\Byte-vX.Y.Z-windows-x64-portable.zip .\byte-portable
Get-AuthenticodeSignature .\byte-portable\Byte.exe |
  Format-List Status,SignerCertificate,TimeStamperCertificate
```

### SHA-256

```powershell
Get-FileHash .\Byte-vX.Y.Z-windows-x64-setup.exe -Algorithm SHA256
```

Compare it with `SHA256SUMS.txt` from the same release.

### GitHub provenance

```powershell
gh attestation verify .\Byte-vX.Y.Z-windows-x64-setup.exe --repo thiepn/byte
```

## SmartScreen expectations

Authenticode signing is necessary for a professional Windows distribution, but it does not guarantee that a brand-new app or brand-new certificate will never receive a SmartScreen reputation warning.

SmartScreen reputation can still take time to establish. P1's goal is to remove Byte's avoidable **unknown-publisher** condition and provide a verifiable publisher/integrity chain; it does not attempt to bypass Windows reputation protections.

## Certificate rotation

When replacing an expiring or revoked certificate:

1. obtain the replacement code-signing certificate
2. replace all three signing secrets atomically
3. run a tagged release only after the new secret set is complete
4. inspect `release-manifest.json` and confirm the new signer subject/thumbprint
5. confirm installer and portable executable use the same signer
6. confirm both signatures are timestamped
7. retain older signed releases unchanged

Do not re-sign old release files in place. Publish a higher Byte version when source or binaries change.

## Failure policy

A tagged release must stop before publication if any of these occur:

- signing secrets are missing or partial
- PFX import fails
- certificate is invalid for signing
- timestamp URL is invalid
- Tauri signing fails
- portable post-bundle signing fails
- installer or portable signature is invalid
- signer thumbprints differ
- either signature lacks a timestamp
- installed `Byte.exe` does not match the installer signer
- published downloadable files fail the final re-download verification
