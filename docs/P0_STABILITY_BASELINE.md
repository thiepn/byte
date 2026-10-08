# Byte 2.0 — P0 Stability Baseline

**Scope:** Establish a reliable Windows desktop runtime before the Byte 2.0 art and UX redesign. P0 does **not** certify a new public release or assert physical-device outcomes that have not been tested.

## Baseline

- Base: `main`, after merged PR #34 (`53629ce`), which repaired Quick Panel/native-action handling, stale IPC reads, startup recovery, customization preview errors, dependency audit and bounded Quick Panel placement.
- P0 development: PR #36. Automated test runs on the PR and on the eventual merge commit are the reproducible implementation evidence.
- Published v0.1.0 installers are older than the September 28 scroll fix and the P0 repair set. Never describe a working tree/CI build as a shipped installer.

## Fixed in P0

1. **User-hidden companion unexpectedly reappears.** A native layout apply formerly always called `show()`, even for cosmetic, size or edge-anchor changes. Layout changes now preserve the prior visibility. Explicit display-mode changes can reveal the character; Tray mode and desktop suppression always win.
2. **Rollback could lose visibility intent.** Failed settings writes now restore both the previous saved preferences and the previous visible/hidden state.
3. **Opaque Rust panic diagnostics.** A 64 KiB rotating local log records only epoch seconds and source basename/line after the Tauri app's config directory is available. No panic payload, absolute paths, typed content, hardware telemetry or network upload. Diagnostic writing must never prevent the application from running.

Panic logging is intentionally a narrow diagnostic aid: native access violations, sudden power loss, terminated processes and early failures before Tauri setup need not produce a panic entry. It is not crash-dump or automatic recovery infrastructure.

## Automated gates

| Gate | How to verify |
| --- | --- |
| TypeScript / Svelte integrity | `npm run check` |
| UI / behavioral regressions | `npm run test` |
| Production frontend | `npm run build` |
| Frontend dependency vulnerabilities | `npm audit --audit-level=high` |
| Rust style / tests | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --locked --manifest-path src-tauri/Cargo.toml` |
| Rust lint, static check, audit | `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo check --locked --manifest-path src-tauri/Cargo.toml`, `cargo audit` |
| Portable + installer + runtime packaging | `Byte Packaging Certification` GitHub Actions workflow |

Never merge P0 with a failing required workflow, and never infer successful physical behavior solely from host-runner tests.

## Physical-device acceptance — Windows 11 x64

Use the exact candidate source revision, follow `docs/PHYSICAL_DEVICE_SIGNOFF.md`, and record the result for **every** scenario rather than claiming pass from code review.

| Scenario | Expected result |
| --- | --- |
| New installation and onboarding | Companion hidden until onboarding is completed; no duplicate instance |
| Secondary launch while Byte runs | Existing Byte is focused; second process exits without duplicate tray/hooks |
| Close main window, reopen from tray | Byte keeps running, then returns with settings intact |
| Hide companion, change its palette/size/habitat | Companion stays hidden; tray Open/Show works |
| Hidden → new display mode | Explicit mode change reveals only when not Tray/fullscreen-suppressed |
| Failed layout write | Previous layout, preferences and visibility recovered |
| Tray-only → Habitat and back | Predictable show/hide with no ghost panel |
| Begin Move Mode, drag, finish | Move completes, correct monitor placement persists after restart |
| Click-through on/off | Mouse input passes through when on; Move Mode safely disables it |
| Mixed-DPI multi-monitor and hot-unplug | Companion and panel remain onscreen, restore on surviving monitor |
| Fullscreen/presentation and desktop lock | Hide promptly; restore only if previously visible and permitted |
| Sleep / display-off / wake | No stale resource status, runaway worker, or stuck invisible companion |
| Startup with corrupt config or collection | Corrupt file quarantined; safe defaults recover without data leak |
| Crash diagnostics | Panic record is bounded and local; no private payload; no false claim of catching native crashes |
| Reduced motion, keyboard, 100–200% text scale | Controls remain reachable and readable |
| Long idle session | Meets measurable CPU/RAM/GPU/disk/network release thresholds |

**Status:** automated CI/packaging must be verified against the P0 head; physical acceptance remains **NOT RUN** until performed on a real Windows machine.

## Exit criteria

- P0 PR merged only after successful CI and packaging.
- No known blocker-level defects in Windows windowing, IPC startup or local persistence.
- Local diagnostics verified; no telemetry/account/cloud requirement introduced.
- Clear physical test record. Without that record, the branch can become a development baseline but **not** a certified public release.
- Freeze architecture only after regression evidence; P1 can assess visuals independently of public release.
