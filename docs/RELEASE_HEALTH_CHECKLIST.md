# Byte Release Health Checklist

Use this checklist for every stable release and for high-risk Beta or hotfix releases.

## Before tagging

- [ ] `CHANGELOG.md` has a concrete section for the exact version.
- [ ] Version matches in `package.json`, `package-lock.json`, `Cargo.toml`, and `tauri.conf.json`.
- [ ] `npm run maintenance:verify` passes.
- [ ] Normal Byte CI is green.
- [ ] Windows Packaging Certification is green for the exact source commit.
- [ ] P4 automated real-world product certification is green for the exact source commit.
- [ ] `product-certification.json` covers fresh first-run, all display modes, 100–200% interface scales, runtime migration, corrupt-config recovery, and sustained runtime.
- [ ] The exact `main` commit has a certified release-candidate artifact.
- [ ] A signed `Byte Device Signoff Candidate` has been generated from `main` for final physical trust/device testing.
- [ ] P4-M physical Windows signoff is complete on the signed candidate.
- [ ] The resulting device report verifies with `scripts/verify-physical-device-signoff.ps1 -RequireSigning -RequirePublicReleaseReady`.
- [ ] P5 release approval has produced `release-approval-vX.Y.Z.json` from exact current `main`.
- [ ] The approval receipt verifies with `scripts/verify-release-approval.ps1 -RequireTaggingApproval`.
- [ ] The approval receipt reports `approved_for_tagging: true`, `github_attestation_verified: true`, `automatic_tag_created: false`, and `automatic_release_published: false`.
- [ ] P6 dry-run preflight passes for the exact current `main` commit and version.
- [ ] P6 re-verifies exact remote `main`, local HEAD, branch, and clean working tree immediately before tag mutation.
- [ ] The version tag is created only through P6 controlled initiation; do not manually force/move/delete the release tag.
- [ ] Recent scheduled security audit is green.
- [ ] Recent Windows runner/WebView2 compatibility matrix is green.
- [ ] Configuration migration tests cover any schema changes.
- [ ] User-visible deprecations/migrations are in the changelog.
- [ ] Production Authenticode signing secrets are fully configured for any public Stable/Beta tag.
- [ ] Signing certificate is valid for code signing and not expired.
- [ ] Timestamp service is configured and reachable.

## Manual Windows check

On a currently supported physical Windows 11 desktop, use [PHYSICAL_DEVICE_SIGNOFF.md](PHYSICAL_DEVICE_SIGNOFF.md) and complete the full matrix in [REAL_WORLD_PRODUCT_CERTIFICATION.md](REAL_WORLD_PRODUCT_CERTIFICATION.md). At minimum:

- [ ] Install the current-user NSIS package.
- [ ] First launch completes without elevation.
- [ ] Tray icon and Quick Panel open.
- [ ] Main app opens and closes-to-tray correctly.
- [ ] Companion appears and behaves correctly in Habitat, Perch, Mini, Edge, and Tray modes.
- [ ] Move Mode works, including monitor changes/hot-unplug when multiple displays are available.
- [ ] Byte interface scaling is visually checked through 200%, including High Contrast and Reduce Motion.
- [ ] Fullscreen game/video, presentation, lock, display-off, and sleep/wake restore correctly.
- [ ] Representative-device CPU/memory/GPU/disk/network measurements satisfy the targets in `docs/PERFORMANCE.md`.
- [ ] Launching Byte again while it is already running activates/focuses the existing Byte process and does not leave a second long-lived process.
- [ ] Startup preference can be enabled/disabled.
- [ ] Fullscreen suppression restores correctly.
- [ ] Notification permission surface behaves normally.
- [ ] Stable/Beta channel setting persists.
- [ ] Check for updates opens the expected fixed GitHub destination.
- [ ] Reinstall preserves settings.
- [ ] Uninstall removes program/startup registration while preserving Byte app data.

## Release initiation

- [ ] P6 creates a lightweight `vX.Y.Z` tag pointing exactly to the approved `main` commit.
- [ ] `release-initiation-vX.Y.Z.json` is written and independently verifies with `scripts/verify-release-initiation.ps1 -RequireRemoteState`.
- [ ] P6 records `github_attestation_reverified: true` after independently rechecking installer and portable provenance.
- [ ] The initiation receipt references the expected **Byte Release** workflow run.
- [ ] P6 reports `forced_update: false`, `direct_release_publish_performed: false`, and `direct_asset_upload_performed: false`.
- [ ] If the local P6 process is interrupted after tag creation, use `-ResumeExistingTag`; never recreate, delete, or force-move the tag.

## After publishing

- [ ] GitHub Release title/version are correct.
- [ ] Installer, portable ZIP, release manifest, product certification, release certification, and checksums are present.
- [ ] Release notes match the changelog section.
- [ ] SHA-256 verification succeeds on a freshly downloaded installer.
- [ ] GitHub attestation verification succeeds for installer and portable ZIP.
- [ ] Installer and portable `Byte.exe` both have valid, timestamped Authenticode signatures.
- [ ] Installer and portable executable signer thumbprints match.
- [ ] Authenticode state and signer metadata match `release-manifest.json`.
- [ ] `release-certification.json` reports `public_distribution_ready: true`.
- [ ] Installer can be downloaded and launched from the public release page.
- [ ] Beta releases are marked prerelease; Stable releases are not.

## First 72 hours

- [ ] Review new GitHub issues for install/startup/config migration regressions.
- [ ] Review Dependabot/security alerts.
- [ ] Reproduce any high-severity report on a supported Windows 11 system.
- [ ] If a release-blocking regression is confirmed, use patch-forward hotfix procedure.

## Emergency response

- [ ] Do not force users onto an older installer.
- [ ] Revert/fix source in a reviewed PR.
- [ ] Prepare a higher patch version.
- [ ] Run full certification.
- [ ] Trigger the manual Byte Hotfix workflow.
- [ ] Confirm the resulting tag runs the normal release workflow and provenance gates.
