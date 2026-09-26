# P4 — Real-World Product Certification

P4 is the product-quality gate after Byte's functional, security, packaging, trust, UI, and companion-polish phases.

Its purpose is to answer a different question from ordinary CI:

> Does the packaged application remain coherent across the states a normal Windows user will actually encounter?

P4 deliberately separates **automated packaged-runtime evidence** from **manual real-device signoff**. GitHub-hosted Windows runners are useful for repeatable runtime/state coverage, but they are not a substitute for a representative physical Windows PC, multiple displays, real fullscreen software, Windows notification UX, or human visual review.

## Automated packaged-runtime gate

Every pull request/main packaging candidate and every public tagged release runs:

`scripts/test-real-world-product.ps1`

against the staged portable production build.

The harness exercises the real packaged `Byte.exe`, not a mocked frontend. On Windows, Tauri resolves `app_config_dir()` through the OS known-folder API rather than an arbitrary `APPDATA` environment override, so P4 backs up the runner's actual Byte app-config directory, resets it between scenarios, and restores any pre-existing state afterward.

### Fresh first run

The harness launches Byte without existing state and verifies:

- Byte remains alive through the observation window
- current-schema `config.json` is created
- onboarding remains incomplete
- startup registration is not enabled by default
- default interface scale remains 100%

This protects the first-launch boundary that prevents background integrations from starting before setup is completed.

### Returning-user state matrix

P4 launches returning-user configurations covering every desktop mode:

- Habitat
- Perch
- Mini
- Edge
- Tray only

Across those launches it also covers every supported Byte interface scale:

- 100%
- 110%
- 125%
- 150%
- 175%
- 200%

The matrix additionally includes:

- High contrast
- Reduce motion
- monitoring enabled
- monitoring disabled
- Small / Medium / Large companion sizes
- notifications suppressed for deterministic CI execution

Each packaged launch must remain alive and preserve the requested configuration.

### Runtime migration

P4 creates a schema-v8 returning-user configuration, launches the packaged current build, and requires migration to the current schema with the Stable release channel restored.

Unit migration tests remain authoritative for all historical schema fixtures. This runtime scenario proves that packaged startup actually traverses the migration boundary.

### Corrupt-state recovery

P4 starts Byte with a malformed `config.json`.

Certification requires the packaged application to:

- remain alive
- quarantine the corrupt source as `config.corrupt.json`
- create a valid current-schema replacement

### Sustained-session guardrail

P4 runs a returning-user Habitat session for at least 30 seconds and samples the Byte process.

CI guardrails detect catastrophic/runaway regressions only:

- working set <= 512 MB
- private memory <= 512 MB
- handles <= 2500
- threads <= 300
- process CPU time <= the bounded CI guardrail

These are **not** Byte's representative-device performance targets. The tighter targets in [PERFORMANCE.md](PERFORMANCE.md) still require measurement on a real Windows 11 machine.

## Machine-readable evidence

A passing run writes:

`product-certification.json`

It records:

- Byte version and source commit
- workflow/runner identity
- all automated P4 checks
- exercised display modes
- exercised interface scales
- sustained-session duration
- per-scenario process metrics
- explicit declaration that manual device signoff is still required

`finalize-release-certification.ps1` refuses to create final release certification without this file.

Release certification schema v3 binds the product-certification SHA-256 into `release-certification.json`.

`SHA256SUMS.txt` covers:

- installer
- portable ZIP
- release manifest
- product certification
- release certification

## Manual real-device signoff

Before approving a public Stable release, test the exact certified candidate on a currently supported physical Windows 11 machine.

### 1. First-run and everyday flow

- [ ] Clean install launches without elevation.
- [ ] First-run onboarding fits the window and is understandable without prior Byte knowledge.
- [ ] Back/Continue/cancel/rerun behavior is correct.
- [ ] Completing onboarding starts the intended runtime once, without duplicate workers.
- [ ] Close-to-tray, tray reopen, Quick Panel, main window, and duplicate-launch activation all behave naturally.
- [ ] A normal 30–60 minute desktop session produces no recurring intrusive UI, flicker, stuck overlays, or reaction spam.

### 2. Companion behavior

- [ ] Habitat, Perch, Mini, Edge, and Tray modes behave correctly.
- [ ] Move Mode works and cannot become stuck.
- [ ] Companion interactions feel varied rather than repetitive.
- [ ] Critical/system diagnostic behavior visibly overrides cute/personality behavior.
- [ ] Reduced Motion removes extra movement without removing diagnostic meaning.
- [ ] Companion does not intercept clicks when click-through is enabled.

### 3. Fullscreen, presentation, lock, and sleep

- [ ] Real fullscreen game restores Byte correctly after exit.
- [ ] Fullscreen video behaves correctly.
- [ ] Presentation/slide-show suppression behaves correctly.
- [ ] Lock/unlock restores a fresh state.
- [ ] Display-off/wake restores a fresh state.
- [ ] System sleep/resume restores a fresh state.
- [ ] Repeated transitions do not duplicate windows, workers, notifications, or tray icons.

### 4. Multi-monitor and DPI

Test at least two displays when available.

- [ ] Move Byte between displays.
- [ ] Mixed Windows display scaling behaves correctly.
- [ ] Disconnect the display containing Byte; Byte recovers to an available monitor.
- [ ] Reconnect/rearrange displays; saved placement remains bounded and usable.
- [ ] Quick Panel remains adjacent and visible near screen/work-area edges.
- [ ] Main app remains usable at minimum size.

### 5. Accessibility and visual QA

Check Byte interface scale at 100%, 125%, 150%, 175%, and 200%; include 110% when checking intermediate behavior.

- [ ] Light theme.
- [ ] Dark theme.
- [ ] Windows High Contrast / forced colors.
- [ ] Byte High contrast.
- [ ] Windows reduced motion.
- [ ] Byte Reduce motion.
- [ ] Keyboard-only onboarding/main/Apps/Studio/Settings/Quick Panel.
- [ ] Visible focus states and sensible focus transfer.
- [ ] No clipped controls, unreachable actions, overlapping text, or unusable horizontal overflow.
- [ ] Companion pixel art keeps authored proportions.

### 6. Notifications

Test both allowed and denied Windows notification permission.

- [ ] Permission request/denial does not break the app.
- [ ] Quiet mode and snooze work.
- [ ] Category switches work.
- [ ] A sustained actionable condition produces concise cause + next-step wording.
- [ ] Normal/transient workload does not create notification spam.
- [ ] In-app diagnostic truth remains available when native notifications are unavailable.

### 7. Representative-device performance

Use an optimized packaged build, not a debug build.

Measure at least:

- 10 minutes ACTIVE/CALM
- 10 minutes FULLSCREEN_REDUCED
- 10 minutes locked/display-off
- repeated sleep/wake cycles
- one sustained CPU/memory pressure scenario

Record:

- CPU
- private/resident memory
- GPU engine utilization
- disk writes
- outbound network activity
- observable wakeup/power behavior where available

Acceptance targets remain those in [PERFORMANCE.md](PERFORMANCE.md), including preferably below 0.25% idle CPU and target below 60 MB resident memory on the representative machine.

### 8. Installer, update, and trust UX

For a public signed candidate:

- [ ] Installer presents the expected publisher.
- [ ] Authenticode is valid and timestamped.
- [ ] SmartScreen/trust presentation is consistent with the signed publisher.
- [ ] Reinstall preserves user state.
- [ ] Upgrade from previous supported release preserves user state.
- [ ] Downgrade remains blocked.
- [ ] Uninstall removes program/startup registration and preserves Byte data.
- [ ] Stable/Beta channel persists.
- [ ] Check for updates opens the correct fixed release destination.
- [ ] Freshly downloaded public assets pass checksum/signature/provenance verification.

## Certification rule

A green automated P4 artifact means:

**the exact packaged build passed Byte's repeatable real-world runtime/state matrix.**

It does **not** mean CI has physically observed every Windows interaction.

A public Stable release should be approved only when:

1. normal CI is green,
2. Packaging Certification is green,
3. P4 automated product certification is green,
4. the exact candidate has passed the manual real-device signoff above,
5. public signing/trust/provenance gates are green.

This distinction is intentional and prevents Byte from claiming evidence it does not have.
