# Byte Packaging, Installer & Release Engineering

Phase 26 turns the development build into a repeatable Windows distribution pipeline.

## Production bundle

Byte's production package is an **NSIS current-user installer**.

Configuration invariants:

- product name: `Byte`
- executable name: `Byte.exe`
- identifier: `io.github.thiepn.byte`
- architecture: Windows x64 for the current release workflow
- installer target: NSIS
- install scope: current user
- downgrades: blocked
- WebView2 recovery: silent download bootstrapper when required
- installer/uninstaller icon: Byte's production `.ico`
- compression: LZMA
- Start Menu folder: Byte

Current-user installation matches Byte's existing HKCU-only startup model and avoids requiring administrator privileges for normal installation.

## Version authority

Three project manifests intentionally contain the same version:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

`scripts/verify-release-version.mjs` fails if they diverge.

For a tagged release, the tag must be exactly:

```
v<tauri version>
```

For example, version `0.1.0` releases only from `v0.1.0`.

The tag workflow also refuses to publish a version that is not newer than the latest published Byte release.

## Installer lifecycle

### Install

The NSIS installer uses current-user scope and therefore installs without elevation under the user's local application area.

### Reinstall / upgrade

The packaging certification runs the current installer twice to exercise the in-place replacement path.

For tagged releases, the workflow also downloads the previous Phase-26-compatible installer when one exists:

1. install previous release
2. install new release over it
3. verify the installed executable reports the new version
4. attempt the previous installer again
5. verify the new version remains installed

This converts Byte's `allowDowngrades: false` setting into an exercised release invariant once an earlier standardized installer exists.

### Uninstall

Tauri owns removal of the installed binary, shortcuts, and installer registration.

Byte additionally owns an HKCU Run entry when **Start Byte with Windows** is enabled. The NSIS post-uninstall hook removes that entry so uninstall cannot leave a dead startup command.

Application config/history under the user's app-data directory is intentionally **not** deleted by the installer hook. That preserves preferences across uninstall/reinstall and avoids destroying user data during installer-driven upgrade flows.

## Packaging certification

`.github/workflows/package.yml` runs on pull requests and manual dispatch.

It performs:

1. frozen frontend dependency install
2. release-metadata validation
3. real Tauri production build
4. real NSIS bundling
5. release-artifact staging
6. silent installer install
7. same-version reinstall
8. installed version verification
9. silent uninstall
10. startup-registry cleanup verification
11. workflow artifact upload

Packaging therefore has an execution test separate from compilation/unit CI.

## Release artifacts

`scripts/stage-release.ps1` creates:

- `Byte-vX.Y.Z-windows-x64-setup.exe`
- `Byte-vX.Y.Z-windows-x64-portable.zip`
- `release-manifest.json`
- `SHA256SUMS.txt`

The portable ZIP contains the production `Byte.exe` built from the same tagged source.

The release manifest records:

- product/version
- identifier
- target architecture
- installer and portable filenames
- install scope
- downgrade policy
- whether both app executable and installer have valid Authenticode signatures
- Git commit when built in GitHub Actions

SHA-256 values are generated after all signing/build operations are complete.

## GitHub Release workflow

Pushing `v*` runs `.github/workflows/release.yml`.

The workflow:

1. verifies tag/version consistency
2. audits npm dependencies
3. audits Cargo dependencies through RustSec
4. checks release ordering
5. requires and validates Windows code signing
6. builds the signed NSIS release from locked dependencies
7. stages portable/installer/checksum artifacts
8. runs upgrade/downgrade/uninstall certification
9. retains the exact artifacts as a workflow artifact
10. creates or updates the matching GitHub Release
11. re-downloads the published assets and re-verifies Authenticode, checksums, certification, and provenance

The release is created only after installer certification succeeds.

## Windows code signing

Unsigned builds remain supported for contributors, pull requests, and ordinary `main` release-candidate certification.

**Tagged public releases are no longer allowed to publish unsigned.** Stable and Beta tag workflows require all three repository secrets:

- `WINDOWS_CERTIFICATE` — base64 PFX
- `WINDOWS_CERTIFICATE_PASSWORD`
- `WINDOWS_TIMESTAMP_URL`

For tagged releases, missing signing configuration is a hard failure. For non-public candidate builds, none may be present and the candidate remains intentionally unsigned. Partial configuration always fails.

The certificate is imported into the current-user certificate store only for the build. A temporary Tauri configuration supplies the certificate thumbprint, SHA-256 digest, and timestamp URL. After Tauri creates the NSIS bundle, Byte explicitly signs the restored `target/release/Byte.exe` that becomes the portable build.

Staging records signer subject/issuer/thumbprint and timestamp state. Public release certification requires the installer and portable binary to be validly signed, timestamped, and signed by the same certificate. The installed executable is checked again during real NSIS lifecycle testing.

After GitHub Release publication, the workflow freshly downloads the public assets and repeats checksum, Authenticode, certification, and provenance verification. Signing material and the imported certificate are cleaned after the run.

See [WINDOWS_TRUST.md](WINDOWS_TRUST.md) for certificate setup, verification, and rotation.

## Reproducibility

Release/package builds use the committed:

- `package-lock.json`
- `src-tauri/Cargo.lock`

Normal CI already enforces `npm ci` and Cargo `--locked`.

Release automation uses the same SHA-pinned GitHub Actions established in Phase 25.

## Manual release procedure

1. Update all three version fields together.
2. Open a PR and require normal CI + Packaging Certification to pass.
3. Merge to `main`.
4. Create and push tag `vX.Y.Z` on the intended main commit.
5. Confirm the **Byte Release** workflow passes.
6. Verify the GitHub Release contains the installer, portable ZIP, manifest, and SHA-256 file.

Do not manually upload an installer built from a different commit under the same release tag.


## Phase 27 certification layer

Phase 27 adds a final artifact-level gate after Phase 26 packaging succeeds.

The packaging workflow now also runs on pushes to `main`, producing a release candidate for the exact merged commit only when all certification checks pass.

After staging, Byte verifies the installer/portable filenames, SHA-256 values, manifest metadata, embedded executable version, portable archive contents, x64 PE machine type, and signing-state consistency.

The portable executable is then launched for a bounded runtime smoke test.

Installer certification now additionally verifies that Byte's roaming application data survives reinstall/upgrade and uninstall. Installer-owned registration and the HKCU startup value must still be removed.

Successful builds receive `release-certification.json`, after which `SHA256SUMS.txt` is finalized across the installer, portable ZIP, release manifest, and certification file.

Tagged release builds additionally generate and immediately verify GitHub artifact attestations for the installer and portable ZIP before GitHub Release publication.

See [RELEASE_CERTIFICATION.md](RELEASE_CERTIFICATION.md).
