# Changelog

Byte follows a human-readable changelog for user-visible and release-engineering changes.

## [Unreleased]

### Fixed

- Launching Byte while it is already running now activates the existing process and brings its main window forward instead of silently exiting; the session-scoped Windows event also removes the stale-file-lock edge case.

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

- "Run onboarding again" is now a UI-only tutorial mode: it preserves saved notification/runtime state, can be exited without saving, and no longer toggles the first-run completion flag behind the scenes.
- Re-running onboarding no longer spawns duplicate desktop-awareness, telemetry, or global-input workers when setup is completed again.
- Tray/Quick Panel actions can no longer bypass first-run setup: tray clicks return to onboarding, and Move Mode stays unavailable until onboarding completes.
- Eliminated an intermittent startup race by deferring all configured WebViews until after `AppState` is managed; early frontend IPC can no longer abort Byte with Windows status `0xC0000409`.
- Startup failures now exit with a diagnosable error instead of escalating through a top-level panic.

### Security

- Per-window Tauri command permissions, local-only content policy, bounded persistence reads, privacy-minimized notifications/history, locked dependency graphs, and dependency audits.
