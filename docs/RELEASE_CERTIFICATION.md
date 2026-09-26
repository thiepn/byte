# Byte Final Release Certification & Distribution

Phase 27 defines the final gate between a successful source build and a Byte binary that is ready for users to download.

## Distribution channels

Byte has two Windows x64 artifacts:

- **Installer** — recommended for normal use:
  `Byte-vX.Y.Z-windows-x64-setup.exe`
- **Portable** — a ZIP containing exactly one `Byte.exe`:
  `Byte-vX.Y.Z-windows-x64-portable.zip`

Official tagged builds are published through GitHub Releases.

The packaging workflow also produces certified **release candidate** artifacts for pull requests, manual runs, and every push to `main`. These are CI artifacts, not public releases.

## Certification layers

A build is distribution-ready only after all required layers pass.

### 1. Source and dependency verification

The existing CI gate must pass:

- frontend dependency audit
- TypeScript/Svelte checking
- frontend tests
- production frontend build
- Rust formatting
- Rust tests
- RustSec audit
- Clippy with warnings denied
- final locked Cargo check

### 2. Packaging verification

The production Tauri package must build successfully as:

- `Byte.exe`
- Windows x64
- NSIS current-user installer
- downgrade-disabled configuration

The installer and portable ZIP are staged from the same production build.

### 3. Artifact integrity verification

`scripts/verify-release-artifacts.ps1` certifies the staged files themselves.

It checks:

- required filenames
- release-manifest identity and policy fields
- SHA-256 values
- portable ZIP contents
- embedded executable version
- Windows PE signature and x64 machine type
- Authenticode state against the release manifest
- final certification metadata when requested

The portable ZIP must contain exactly one `Byte.exe`.

### 4. Portable runtime smoke test

The portable executable is extracted and started from an isolated roaming-app-data environment.

Certification requires **three consecutive launches** from that isolated environment. Each launch must remain alive for the observation window before the process is terminated and the next launch begins. A failure reports the signed/decimal exit status, captured process output, and matching Windows Application Error / Windows Error Reporting events when available.

Byte must remain alive for every observation window instead of crashing during:

- process startup
- WebView2 initialization
- Tauri initialization
- local state creation
- window/tray setup

Each process is terminated after its smoke window. After all three sequential starts succeed, certification performs one concurrent duplicate-launch handoff: the primary process must remain alive while a second Byte process exits cleanly within five seconds. This exercises the packaged single-instance boundary in addition to the Rust-level activation-event tests. After the duplicate-launch check succeeds, all temporary test data is removed.

### 5. Installer lifecycle certification

The real NSIS package is silently exercised.

The gate verifies:

1. clean install
2. installed executable discovery
3. installed version
4. same-version reinstall
5. upgrade from the previous standardized Byte release when available
6. downgrade prevention against that previous release
7. startup-registration cleanup
8. uninstall registration cleanup
9. installed executable removal
10. application-data preservation

A unique sentinel file is placed under Byte's roaming app-data directory before reinstall/upgrade. It must survive reinstall, downgrade-policy testing, and uninstall. The test removes only its own sentinel afterward.

This protects the intended ownership boundary:

- installer-owned binaries/registration → removable
- user configuration/history → preserved

### 6. Machine-readable release certification

After runtime and installer tests pass, `scripts/finalize-release-certification.ps1` writes:

`release-certification.json`

It records:

- Byte version
- target architecture
- source commit
- workflow/run identity
- signing state
- certification level
- all exercised release checks
- installer SHA-256
- portable ZIP SHA-256
- whether previous-release upgrade/downgrade testing was available

The certification is marked `distribution_ready: true` only because the file is generated after the preceding gates complete successfully.

`SHA256SUMS.txt` is then regenerated so it covers:

- installer
- portable ZIP
- release manifest
- release certification

### 7. GitHub build provenance

Tagged releases generate GitHub artifact attestations for:

- installer EXE
- portable ZIP

The workflow immediately verifies both attestations with GitHub CLI before publishing the release.

This provides cryptographic provenance linking the downloadable binary to the repository, workflow, and source commit.

It does not replace normal security review or code signing.

## Release candidate policy

`.github/workflows/package.yml` runs on:

- pull requests to `main`
- pushes to `main`
- manual dispatch

Only a fully successful workflow uploads the named release-candidate artifact.

This means the candidate attached to an exact `main` commit has passed the same package/runtime lifecycle gate rather than being inferred from a pre-merge PR run.

## Public release policy

A public Byte release requires a `vX.Y.Z` tag matching all project version declarations.

The tagged workflow must pass:

- version/tag check
- npm audit
- RustSec audit
- release-order check
- required public-release signing configuration validation
- production package build
- artifact-integrity verification
- portable launch smoke
- installer lifecycle certification
- previous-release upgrade/downgrade test when available
- final certification-file verification
- installer provenance attestation
- portable provenance attestation
- immediate attestation verification

Only then are the exact staged files uploaded to the GitHub Release. After publication, the workflow downloads those public assets again and repeats artifact/signing/provenance verification before the run can succeed.

## User verification

### SHA-256

On PowerShell:

```powershell
Get-FileHash .\Byte-vX.Y.Z-windows-x64-setup.exe -Algorithm SHA256
```

Compare the result with `SHA256SUMS.txt` from the same GitHub Release.

### Build provenance

With GitHub CLI:

```powershell
gh attestation verify .\Byte-vX.Y.Z-windows-x64-setup.exe --repo thiepn/byte
```

The same command can be used for the portable ZIP.

### Signing

Manifest/certification schema v2 records the full public trust state for the installer and portable executable: validity, signer subject/issuer/thumbprint, certificate validity window, timestamp presence, and timestamp signer metadata.

Unsigned pull-request/`main` builds may still be structurally certified as release candidates. They receive `public_distribution_ready: false`.

Tagged Stable and Beta releases require:

- valid Authenticode on the installer
- valid Authenticode on portable `Byte.exe`
- the same signer certificate on both
- trusted timestamps on both
- the installed executable to retain the same signer during NSIS lifecycle testing

A public release cannot be published if those checks fail.

Byte v0.1.0 predates this fail-closed public-signing policy and is recorded as unsigned. See [WINDOWS_TRUST.md](WINDOWS_TRUST.md).

## Release contents

A complete official release contains:

- `Byte-vX.Y.Z-windows-x64-setup.exe`
- `Byte-vX.Y.Z-windows-x64-portable.zip`
- `release-manifest.json`
- `release-certification.json`
- `SHA256SUMS.txt`

The installer and portable ZIP also have repository artifact attestations.

## Non-goals

Phase 27 does not add:

- an in-app auto-updater
- telemetry or crash reporting
- a cloud account
- a background release service
- silent self-update behavior

Distribution remains explicit and user-controlled.
