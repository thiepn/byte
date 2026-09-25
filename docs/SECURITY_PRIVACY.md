# Byte Security & Privacy Hardening

Phase 25 defines Byte's security model for the Windows desktop build.

## Threat model

Byte treats the frontend WebViews as lower-trust than the Rust core.

The design assumes that a frontend bug, injected script, corrupted local state file, malformed persisted preference, unexpected process name, or optional Windows API failure must not automatically become arbitrary native capability.

Byte does **not** claim to defend against:

- an attacker who already has arbitrary code execution as the same Windows user
- an administrator or kernel-level compromise
- a malicious replacement Byte binary
- an unpatched WebView2 or Windows vulnerability
- supply-chain compromise outside the controls described below

## IPC authority

All application commands are registered with Tauri's application manifest so command permissions are generated and enforced through capabilities.

Capabilities are split by window label:

### Main window

May read local state and use the explicit settings/diagnostic actions required by the full application.

It does not receive generic window, filesystem, process, shell, tray, webview, menu, image, resource, or notification-plugin permissions.

### Quick Panel

May:

- read cached snapshot/preferences/shell state
- execute Byte's allowlisted recommended actions
- change display mode
- enter Move Mode
- toggle click-through
- hide itself

It cannot inspect apps, mutate application settings, clear history, request notification permission, or open arbitrary URLs.

### Companion

May:

- read cached snapshot/preferences/awareness/shell state
- report the two known rare-idle discovery enums
- perform the existing Move Mode drag/finish flow
- request the Quick Panel

It cannot mutate general preferences, inspect processes, execute system actions, clear data, or request OS notification privileges.

All three WebViews receive only event **listen/unlisten** permissions. Frontend event emission is intentionally not granted.

No remote origin capability exists.

## Content Security Policy

Production CSP is local-only:

- scripts: self
- styles: self plus inline style support required by Svelte style bindings
- images/fonts: self
- IPC connections: Tauri IPC only
- object/frame/form/base targets: disabled

Tauri CSP rewriting remains enabled.

`freezePrototype` is enabled so `Object.prototype` is frozen before frontend code runs.

The asset protocol is explicitly disabled.

Development adds only the local Vite WebSocket endpoint required for HMR.

## Native action boundary

Byte does not expose a generic shell command or URL opener.

Rust's action layer contains a fixed allowlist only:

- Task Manager
- Windows Storage Settings
- Windows Battery Settings
- Byte's fixed Stable GitHub release destination
- Byte's fixed Beta GitHub release destination
- Byte's own main window

No frontend string becomes an executable path, URI, shell argument, or command line.

## IPC input validation

Rust validates data again at the native boundary even though TypeScript has narrower types.

### App preferences

- text scale is restricted to 100 / 110 / 125
- notification snooze is limited to seven days
- foreground exclusions are limited to 32 entries
- exclusion names are bounded, normalized, deduplicated and cannot contain paths/control characters

### Companion preferences

Rust allowlists:

- four character IDs
- six habitat IDs
- character-specific palette IDs
- each cosmetic in its correct semantic slot
- each decoration in its correct fixed habitat slot

Saved placement coordinates must be finite normalized values in `[0, 1]`. Monitor names are bounded and reject control characters.

Collection gating remains a second authorization layer for optional locked cosmetics/palettes/decorations.

Semantically invalid on-disk configuration is treated as corrupt and recovered instead of trusted merely because its JSON shape deserializes.

## Local persistence hardening

Persistent JSON inputs are read through bounded readers before deserialization:

- config: 256 KiB
- collection: 256 KiB
- notification state: 128 KiB
- Activity events: 512 KiB

Oversized or invalid UTF-8 state cannot trigger an unbounded `read_to_string`.

Writes remain same-directory temporary-file replacements.

Corrupt configuration is quarantined before defaults are restored.

## Privacy minimization

### Native notifications

Native Windows notifications intentionally omit:

- process/app names
- PIDs
- exact CPU percentages
- exact memory figures
- exact battery percentages
- exact temperature readings

They state the category and next step only. Detailed local diagnostics remain inside Byte.

### Activity history

CPU and memory Activity events do not persist process attribution or process names. The current Apps view may still inspect process groups on demand, but that attribution stays point-in-time and local rather than being written into Activity history.

### Global input

Phase 25 does not widen the existing anonymous input contract. No key identity, typed text, clipboard content, window title, mouse coordinates, or input history is stored.

Fresh installs do not create the global-input hook runtime until onboarding is completed. If onboarding is intentionally reopened later, capture is disabled immediately and stays disabled until the existing runtime is reactivated by completing setup again.

## Unsafe/native code

The Rust crate denies unsafe operations inside unsafe functions unless those operations are placed in explicit unsafe blocks.

Existing Windows FFI remains narrowly wrapped and documented at call sites.

## Dependency and CI controls

Security-sensitive Tauri dependencies are pinned to exact versions:

- `tauri = 2.11.5`
- `tauri-build = 2.6.3`
- `tauri-plugin-notification = 2.3.3`
- `@tauri-apps/api = 2.11.1`
- `@tauri-apps/cli = 2.11.4`

Both `package-lock.json` and `src-tauri/Cargo.lock` are committed. CI uses `npm ci` plus Cargo's `--locked` mode so dependency resolution cannot silently drift during verification.

CI pins its GitHub Actions by commit SHA instead of mutable major-version tags.

Every CI run adds:

- `npm audit --audit-level=high`
- `cargo audit` using cargo-audit 0.22.2

Dependabot watches npm, Cargo and GitHub Actions weekly.

These checks reduce known-vulnerability and dependency-drift risk; they do not make third-party dependencies intrinsically trusted.

## Security invariants

Phase 25 regression tests certify that:

- unexpected character/palette/customization IDs are rejected
- cosmetic/decor IDs cannot be smuggled into the wrong slot
- monitor placements are normalized and bounded
- excluded executable names cannot contain filesystem paths
- oversized persisted state is rejected before unbounded reads
- native notifications omit process names and exact readings
- persisted CPU/memory Activity details omit process attribution

Capability/CSP configuration is additionally validated by the Tauri build in Windows CI.


## Release provenance

Phase 27 adds GitHub artifact attestations to tagged installer and portable releases.

The release workflow grants the minimum extra permissions required for provenance generation:

- `attestations: write`
- `id-token: write`

The installer and portable ZIP are attested only after package/runtime certification succeeds. The workflow then verifies both attestations before publishing the GitHub Release.

Release provenance links a binary to its source workflow and commit; it is an integrity/provenance control, not a claim that the binary is free of vulnerabilities.


## Post-release maintenance security

Phase 28 keeps update discovery user-initiated. Selecting Stable or Beta stores only a local enum; Byte performs no background release polling and exposes no arbitrary update URL.

Weekly maintenance automation re-runs npm and RustSec dependency audits even when the source tree is otherwise quiet. Dependabot groups routine minor/patch maintenance, while all resulting changes still pass the normal security and packaging gates.

Emergency recovery is patch-forward. Byte does not weaken installer downgrade protection to perform a rollback. A corrective higher patch is certified and released through the normal provenance pipeline.
