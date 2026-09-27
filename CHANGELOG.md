# Changelog

Byte follows a human-readable changelog for user-visible and release-engineering changes.

## [Unreleased]

### Added

- Added P5 one-command release approval orchestration: exact-main preflight, signed device-candidate dispatch/reuse, workflow watching, exact-artifact download, local GitHub provenance verification, P4-M execution, independent evidence verification, and candidate-bound `release-approval-vX.Y.Z.json` receipts.
- Added a strict release-approval verifier and synthetic tamper harness covering device-report hash changes, source-commit changes, and false automatic-tag claims.

- Added P4-M physical Windows device signoff tooling: guided PASS/FAIL/NA evidence capture, exact-candidate hash binding, three 10-minute representative performance states, full Byte + WebView2 process-tree measurement, strict report verification, and capability-gap tracking.
- Added a main-only, manually dispatched signed `Byte Device Signoff Candidate` workflow so Authenticode/SmartScreen/trust UX can be tested before a public tag without publishing a GitHub Release.

- Added P4 Real-World Product Certification: packaged first-run and returning-user runtime profiles, all display modes, 100–200% interface-scale coverage, runtime migration, corrupt-state recovery, sustained-session runaway guardrails, and machine-readable `product-certification.json` evidence.
- Added a mandatory physical-Windows signoff matrix for fullscreen/presentation, lock/sleep, multi-monitor/DPI, notifications, representative-device performance, accessibility/visual QA, and installer/update/trust UX.

- Added P3 companion polish: bounded interaction-driven habitat ambience in the live companion and Customization Studio preview, with reduced-motion suppression and the existing global particle cap preserved.
- Added Application UI/UX V2: icon-led navigation, clearer Overview/Activity/Apps hierarchy, companion presence in the application shell, richer empty/error/privacy states, and interface-scale choices through 200%.
- Added a fail-closed Windows trust pipeline for future public releases: Authenticode signer metadata, timestamp verification, installed-binary signer checks, and fresh public-download verification.
- Added a dedicated Windows trust/code-signing runbook with certificate setup, rotation, SmartScreen expectations, and user verification commands.

### Changed

- Stable pre-tag readiness now ends at a verified `approved_for_tagging: true` receipt; P5 deliberately never creates/pushes tags or publishes GitHub Releases.
- CI, packaging, tagged releases, and signed device-candidate workflows now syntax/policy-check P5 tooling, while artifact-producing workflows also exercise the candidate-bound approval verifier.

- Public-release physical signoff now distinguishes ordinary `device_ready` evidence from `public_release_ready`: the latter requires a signed/timestamped same-signer candidate, dual-display coverage, trust PASS, and zero capability gaps.
- CI, packaging, public release, and signed-device-candidate workflows now syntax/policy-check the P4-M PowerShell tooling.

- Release certification schema advances to v3 and now binds the P4 product-certification hash; certified release checksums include the product-certification artifact.
- PR/main packaging and public tagged releases now fail closed if the P4 automated real-world runtime gate does not pass.
- Windows launch smokes now isolate the actual Tauri app-config directory via backup/reset/restore instead of assuming an `APPDATA` environment override redirects the Windows known-folder path.

- Refined companion behavior so incidental idles avoid immediate repetition and pointer/typing personality reactions rotate through authored alternatives with short anti-spam cooldowns.
- Polished the Customization Studio around a larger live-preview focus, clearer section navigation, richer choice cards, and a more useful current-look summary.
- Refined the application visual system with stronger typography, spacing, surfaces, status hierarchy, responsive behavior, and automatic shell compaction at 150–200% interface scaling.
- Future tagged Stable and Beta releases now require valid timestamped Authenticode signatures on both the NSIS installer and portable `Byte.exe`; unsigned PR/`main` candidates remain supported.
- Windows package metadata now uses publisher `THIEPN` and the Byte product homepage at `https://thiepn.dev/byte/`.
- Release manifest and certification formats advance to schema v2 and distinguish structurally certified candidates from signed public-distribution-ready builds.

### Security

- Portable `Byte.exe` is explicitly signed after Tauri bundling, preventing the restored standalone binary from bypassing the public signing policy.
- Public release publication now requires the same Authenticode signer on installer and portable executable, timestamps on both, and a successful re-download verification after GitHub Release upload.

## [0.1.0]

### Added

- Initial Windows x64 Byte desktop companion with local system-health monitoring, companion modes, customization, collection extras, Settings, notifications, accessibility, and lifecycle-aware performance behavior.
- Certified NSIS installer and portable distribution with checksums, release manifests, release certification, and GitHub build provenance.
- Stable and Beta release-channel selection with explicit user-initiated update discovery.
- Release preparation, weekly maintenance certification, compatibility checks, and certified hotfix tooling.

### Changed

- Fresh installs do not start telemetry, desktop-awareness, or global-input background workers until onboarding is completed; finishing setup starts the normal local runtime while still respecting monitoring preferences and fullscreen, lock, and sleep suppression.
- Release and maintenance certification now require three consecutive healthy Byte launches instead of a single startup probe.
- Fresh installs now make Smart Notifications an explicit onboarding choice that defaults off, and native alerts remain suppressed until onboarding is complete.
- Windows launch certification and maintenance smokes now capture process output, hexadecimal exit status, and matching Windows crash events when Byte exits unexpectedly.

### Fixed

- Launching Byte while it is already running now activates the existing process and brings its main window forward instead of silently exiting; the session-scoped Windows event also removes the stale-file-lock edge case.
- "Run onboarding again" is now a UI-only tutorial mode: it preserves saved notification/runtime state, can be exited without saving, and no longer toggles the first-run completion flag behind the scenes.
- Re-running onboarding no longer spawns duplicate desktop-awareness, telemetry, or global-input workers when setup is completed again.
- Tray/Quick Panel actions can no longer bypass first-run setup: tray clicks return to onboarding, and Move Mode stays unavailable until onboarding completes.
- Eliminated an intermittent startup race by deferring all configured WebViews until after `AppState` is managed; early frontend IPC can no longer abort Byte with Windows status `0xC0000409`.
- Startup failures now exit with a diagnosable error instead of escalating through a top-level panic.

### Security

- Per-window Tauri command permissions, local-only content policy, bounded persistence reads, privacy-minimized notifications/history, locked dependency graphs, and dependency audits.
