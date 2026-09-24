# Byte Architecture — Current Baseline

## Boundaries

Byte has four layers:

1. Platform core: Windows integration, lifecycle, anonymous input, telemetry.
2. Application core: state, diagnostics, activity, configuration.
3. Companion engine: character/habitat/cosmetic behavior and rendering.
4. Product UI: Quick Panel and main application features.

The frontend never independently polls hardware. The backend never selects sprite frames.

## Authoritative telemetry

Rust owns one cached SystemSnapshot. Frontend IPC only reads that cache.

Telemetry has three layers:

- TelemetrySource: raw platform measurements.
- TelemetryEngine: smoothing, normalization, units, optional-signal handling.
- telemetry runtime: exactly one lifecycle-aware worker that updates AppState.

On Windows, WindowsTelemetrySource uses sysinfo for CPU, RAM, disks, networks, and best-effort component temperature, plus GetSystemPowerStatus for battery state.

See [TELEMETRY.md](TELEMETRY.md).

## Human-friendly diagnostics

The DiagnosticEngine sits after telemetry normalization and before the snapshot cache. It owns sustained-condition timing, hysteresis, recovery, issue priority, process culprit attribution, confidence, and safe recommendations. It never performs destructive system actions.

See [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Global input reactions

One dedicated low-level hook thread observes keyboard, left/right mouse button, and wheel activity. Hook callbacks only enqueue anonymous activity and immediately return. A separate interpreter worker owns debounce, alternating typing taps, fast-typing detection, and idle detection.

Only semantic reaction events cross into Svelte. No key identity, scan code, typed text, mouse position, or input history crosses the platform boundary.

See [INPUT_REACTIONS.md](INPUT_REACTIONS.md).

## Windows shell

The shell service owns one reusable transparent companion surface plus Quick Panel and main windows. It handles monitor selection, DPI-aware sizing, taskbar-aware work areas, normalized placement persistence, off-screen recovery, Move Mode, native dragging, click-through state, and tray-driven display modes.

See [WINDOWING.md](WINDOWING.md).

## Character animation runtime

The companion uses a manifest-driven character engine in Svelte. One shared scheduler drives semantic animation state, transition clips, idle selection, reduced-motion behavior, frame anchors, and canvas rendering.

See [ANIMATION_RUNTIME.md](ANIMATION_RUNTIME.md).

## Habitat rendering runtime

The companion scene is habitat back canvas → grounded character canvas → habitat front canvas. Habitat geometry, four local-time palettes, semantic system-reaction layers, fixed decoration slots, and sparse particles are data-driven.

Phase 12 extends the declarative vocabulary with time-gated layers, ellipses, and polygons and promotes all six shipped habitats to production. The shared scheduler, 15-particle cap, reduced-motion behavior, semantic reaction boundary, and Mini/Edge/Perch rules remain unchanged.

See [HABITAT_RUNTIME.md](HABITAT_RUNTIME.md) and [HABITAT_PRODUCTION.md](HABITAT_PRODUCTION.md).

## Persistence and IPC

ByteConfig is versioned. Invalid config is quarantined and Byte recovers to defaults. Writes use a same-directory temporary file before persistence.

Snapshot reads are cache-only. There is no frontend hardware polling, arbitrary shell command, generic filesystem bridge, cleaner, RAM trimmer, or process-kill IPC.

## Validation

Windows CI runs frontend type checks/tests/build plus Rust format/tests/clippy/check. Production habitat tests certify manifest validity, scene density, time-specific art, and semantic reaction coverage.


## Main-application activity history

Phase 16 adds a local ActivityStore beside the authoritative SystemSnapshot.

The telemetry worker still samples exactly once. After diagnostics evaluate a snapshot, AppState passes that already-produced snapshot into ActivityStore.

ActivityStore has two deliberately different retention classes:

- meaningful events: issue opened, issue resolved, charging started/stopped; capped at 200 and persisted atomically in activity.json
- trend points: CPU, memory, storage, battery, aggregate network, and best-effort thermal; sampled at most every 15 seconds, capped at 240, and kept in memory for the current session only

The full application reads these cached structures through get_activity_history. Opening Overview or Activity never creates another hardware sampler or continuous process scan.


## On-demand app inspection

Phase 17 adds AppInspector as a mutex-owned application-core service.

It is not part of the telemetry loop. The only IPC entrypoint, inspect_apps, causes a point-in-time local process refresh when the user deliberately opens or refreshes Apps. Results are cached briefly to suppress accidental rapid duplicate scans.

AppInspector shares Byte's process-name aggregation and CPU normalization rules but never mutates the authoritative SystemSnapshot or DiagnosticEngine state.

The frontend receives an AppDiagnosticsSnapshot containing at most 12 aggregate app rows plus optional CPU and memory leader attribution. No executable paths, command lines, window titles, or process-management capability cross IPC.


## Customization Studio

Phase 18 remains frontend-driven over the existing CompanionPreferences IPC.

CustomizationStudio owns an optimistic in-memory view of the current preferences and serializes writes through update_companion_preferences. Rapid user changes are queued so older full-preference writes cannot finish after and overwrite a newer choice.

The preview is not a second companion engine. It instantiates the same CharacterAnimator, CharacterCanvasRenderer, HabitatCanvasRenderer, HabitatParticleEngine, semantic attachment system, personality idle profile, and local-time habitat state used by the desktop companion.

Studio Undo/Redo is session-only UI history and is not persisted as a second source of truth.


## Lightweight collection and progression

Phase 19 adds a local CollectionStore persisted in `collection.json`.

CollectionStore is separate from ByteConfig so customization preferences remain the authoritative selected look while collection state records only optional progression. It tracks:

- first-seen timestamp
- anonymous typing-event count
- completed charging transitions
- sparse normal-network moments
- rare idle discoveries
- permanently unlocked collection IDs

Time-based unlocks are evaluated when collection state is read and while normal signals arrive.

Anonymous typing events reuse the existing low-level input pipeline but still contain no key identity. Typing progress is checkpointed every 250 events and on clean shutdown instead of writing to disk on every keypress.

Charging progress counts only a false → true charging transition. Starting Byte while already charging does not count.

Network progress counts at most one moment per 60 seconds while aggregate throughput is at least 0.5 Mbps and stops counting after its single finite unlock threshold. The Studio deliberately hides a numeric network target to avoid encouraging artificial traffic.

Rare idle discovery is reported by the production companion only when an existing `rare_a` or `rare_b` idle occurs naturally. Discovery unlocks replay access in the Studio; it does not remove those rare idles from normal companion behavior.

The backend validates known Phase 19 gated selections before persisting CompanionPreferences. Core Phase 18 items are never gated.


## Phase 20 application settings and Windows integration

Phase 20 introduced onboarding, monitoring, notification, and accessibility state. The current ByteConfig schema is version 8 after later Phase 21–22 preference additions.

New installs default to onboarding incomplete. Installations predating Phase 20 migrate with onboarding marked complete so existing users are not forced through first-run setup.

The `update_app_preferences` command is the single persisted app-settings mutation path. It validates supported text-scale values, applies current-user startup registration when that setting changes, clears the cached system snapshot when monitoring is disabled, and broadcasts `byte://app-preferences-changed` to all Byte windows.

### Startup

Windows startup uses the current-user Run key only:

`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

No administrator privilege, scheduled task, service, or machine-wide registry entry is used.

### Fullscreen awareness

Phase 20 introduced basic foreground-rectangle fullscreen hiding. Phase 22 replaces that narrow watcher with the broader desktop-awareness service described below.

### Monitoring

The existing telemetry worker remains authoritative. When system monitoring is disabled, it stops sampling the system and publishes an unavailable cached snapshot instead. No second monitoring path is introduced.

### Notifications

The Tauri notification integration may send one native notification when a sustained diagnostic reaches `NEEDS_ATTENTION`. The worker suppresses duplicate notifications for the same active issue and clears the suppression when the system leaves NEEDS_ATTENTION.

Notifications never replace in-app diagnostics and contain only Byte's existing issue headline.

### Accessibility

`byte://app-preferences-changed` applies reduced-motion, high-contrast, and text-scale preferences across every webview. The companion animation runtime combines Byte's explicit Reduce Motion setting with the operating-system media preference.

High-contrast mode changes UI tokens only; semantic health states remain represented by text as well as color.


## Smart Notification engine

Phase 21 adds `SmartNotificationEngine` to application core.

The diagnostic engine retains the full ordered list of currently active sustained issues for internal consumers while continuing to publish one primary issue plus a secondary count to the existing SystemSnapshot UI contract.

The notification engine consumes that full active-issue list and applies a second conservative eligibility layer:

- memory: CRITICAL only
- thermal: HIGH or CRITICAL
- storage: CRITICAL only
- battery: CRITICAL only
- runaway process: critical CPU issue, medium/high culprit confidence, named culprit, and at least 10 minutes since the sustained condition began

Only one OS alert is selected from conditions that become eligible together. Priority is thermal → memory → battery → storage → runaway process.

Per-category cooldown timestamps are persisted in `notifications.json` so restarting Byte does not reset alert cooldowns:

- thermal: 30 minutes
- battery: 1 hour
- memory: 4 hours
- runaway process: 4 hours
- storage: 24 hours

The engine separately suppresses duplicate notifications for an active incident until that incident recovers.

Quiet mode, snooze, the master notification switch, and per-category switches are ordinary AppPreferences. Snooze is bounded by the backend to at most seven days.

Notification delivery occurs only when the OS notification permission is granted. A successful notification send commits its cooldown. Failed/blocked delivery does not pretend an alert was delivered.

Notification sound obeys Byte's existing Sound setting.

Phase 21 does not implement app-hang detection because Byte currently has no reliable local signal that can distinguish a hung application from a deliberately non-responsive/background application without increasing false positives.


## Desktop awareness

Phase 22 replaces one-purpose fullscreen hiding with a native `DesktopAwarenessSnapshot` owned by AppState.

The Windows awareness worker combines:

- foreground window/monitor geometry
- `SHQueryUserNotificationState` fullscreen Direct3D / busy / presentation / not-present state
- foreground executable name when an app-specific exclusion check is needed
- `GUID_CONSOLE_DISPLAY_STATE` power-setting notifications through a message-only Windows window

Visibility suppression reasons are explicit:

- FULLSCREEN
- PRESENTATION
- LOCKED
- DISPLAY_SLEEP
- EXCLUDED_APP

AppState stores both the current awareness snapshot and whether the companion was actually visible when suppression began. On recovery, Byte restores only when it had hidden a visible companion. This prevents suppression from overriding Tray mode or a companion the user had already hidden.

Windowing treats suppression as an invariant:

- `show_companion` cannot reveal Byte while suppressed
- `show_quick_panel` cannot reveal the panel while suppressed
- Move Mode cannot start while suppressed
- `apply_companion_layout` keeps the companion hidden while suppressed
- Smart Notifications do not fire while desktop awareness says Byte should remain unobtrusive

When Byte's own main window temporarily becomes foreground, the awareness service preserves the prior external suppression state rather than concluding that a game/presentation ended.

### Capture exclusion

Phase 22 applies Windows `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` by default to companion, Quick Panel, and main windows.

This avoids relying on heuristics such as named Zoom/Teams/OBS processes to infer when screen sharing is active. The setting can be disabled. It is a best-effort Windows capture exclusion for supported capture paths, not a content-security guarantee.

### Foreground exclusions

Users may store up to 32 executable-name exclusions. The backend:

- strips a trailing `.exe`
- lowercases and deduplicates names
- rejects paths and colon/slash/backslash input
- stores no window title or executable path

Foreground process paths are queried transiently only long enough to extract the executable stem; the full path is discarded.

### Startup flash prevention

The companion Tauri window is configured initially hidden. Desktop awareness takes its first observation before normal companion layout is shown, preventing a one-frame overlay flash when Byte launches while a fullscreen/presentation/lock condition is already active.

### Power boundary

Console display-off immediately hides Byte. Phase 23 extends that boundary into coordinated worker suspension and reduced cadences; see the power/performance section below.


## Phase 22 display-sleep suspension

Desktop awareness now maps display-off into the existing `DISPLAY_SLEEP` lifecycle state.

The telemetry worker blocks through `wait_until_sampling_allowed()` while the display is off. On recovery it creates a fresh DiagnosticEngine and marks the cached snapshot unavailable before sampling again, preventing stale pre-sleep sustained timers from immediately producing diagnostic/notification behavior.

The companion receives `byte://lifecycle-changed` and stops animation/particle advancement for FULLSCREEN_REDUCED, LOCKED, DISPLAY_SLEEP, SYSTEM_SLEEP, and SHUTTING_DOWN states.

Fullscreen/presentation suppression remains `FULLSCREEN_REDUCED`: visibility, input reactions, and rendering are suspended while telemetry continues at a reduced cadence.

Restore from any suppression reason requires a 1.5-second continuously clear observation window before the shell may show a previously visible companion again.


## Phase 23 power and performance hardening

Phase 23 keeps Byte lightweight by reducing work at the source rather than adding a cosmetic “eco mode.”

### Adaptive telemetry

The single authoritative telemetry worker remains the only hardware sampler. Its cadence now follows semantic load:

- NEEDS_ATTENTION / STRESSED: 1.5 s
- BUSY: 2.5 s
- CALM: 5 s
- FULLSCREEN_REDUCED: 8 s
- monitoring disabled: 30 s dormant wait, immediately interruptible

LOCKED, DISPLAY_SLEEP, and SYSTEM_SLEEP block telemetry on the shared LifecycleCoordinator condition variable. Resume reconstructs both the WindowsTelemetrySource and DiagnosticEngine before producing a fresh snapshot.

### Input and rendering

Global input capture is gated before hook callbacks allocate timestamps or send channel messages. FULLSCREEN_REDUCED, LOCKED, DISPLAY_SLEEP, SYSTEM_SLEEP, and SHUTTING_DOWN disable that path and place the interpreter on a blocking control receive.

The companion no longer polls get_snapshot every two seconds. Telemetry publishes byte://snapshot-updated from the authoritative worker. The companion subscribes while active, unsubscribes from the shared animation scheduler whenever desktop awareness hides it, and refreshes once on resume.

The main application keeps its cache-only refresh loop only while its window has focus. The Quick Panel already refreshes only while open.

### Expensive process work

Process attribution remains lazy. A raw transient high CPU/memory observation no longer starts an all-process scan. Attribution begins only after a CPU or memory condition has become a sustained issue, and repeated attribution scans are rate-limited to 10 seconds. The Apps surface remains explicit/on-demand.

### Sleep sentinel

The desktop-awareness worker cannot disappear completely because it owns the display-power message window and must notice recovery. It is therefore the single low-frequency sentinel: 500 ms while active, 1 s while fullscreen-reduced, and 2 s while locked/display-sleeping/system-sleeping. Its wait is lifecycle-cancellable so shutdown does not wait for the timeout.

See [PERFORMANCE.md](PERFORMANCE.md).


## Phase 24 accessibility and failure isolation

Phase 24 separates **core startup requirements** from **optional Windows integrations**.

The following failures no longer abort application setup:

- desktop-awareness worker startup
- capture-exclusion application
- telemetry-worker startup
- global input-hook startup

Window creation/layout and the local configuration store remain core requirements. If telemetry cannot start, the cached snapshot stays unavailable and product surfaces say that data is unavailable instead of fabricating readings or remaining indefinitely in a "checking" state.

Capture exclusion remains best-effort. A failure after the preference has been persisted does not roll back unrelated app settings.

### UI accessibility boundary

`App.svelte` centralizes application-level accessibility preferences. Text scale is explicitly surface-aware: main and Quick Panel surfaces scale, while the companion surface remains at 1× so authored pixel-art geometry does not change.

The main application provides a skip link and programmatic main-region focus on section changes. Selection groups expose `aria-pressed`, current navigation exposes `aria-current`, progress indicators expose progressbar values, errors use alert semantics, and save state uses polite live regions.

Global CSS defines focus-visible treatment plus Windows forced-colors behavior. Byte's own High contrast preference remains separate from OS forced-colors and can coexist with it.

### Display recovery

Move Mode clears its shell flag before placement persistence/layout recovery. If a monitor disappears while the user is dragging, a placement error cannot leave the interaction mode latched on.

Placement saving and Quick Panel adjacency use the same monitor fallback chain already used by the main companion layout.

See [ACCESSIBILITY_RESILIENCE.md](ACCESSIBILITY_RESILIENCE.md).


## Phase 25 security and privacy hardening

Phase 25 treats each WebView as a separate authority boundary instead of granting one shared frontend capability.

Tauri application commands are declared through the build-time AppManifest, producing ACL permissions for every registered command. The production configuration explicitly enables only three capabilities:

- main-window
- quick-panel-window
- companion-window

Because explicit capability identifiers are configured, the legacy broad capability file is not loaded.

Frontend event permissions are listen/unlisten only; webviews are not granted generic event emission.

No webview receives generic filesystem, shell, process, window creation, menu, tray, image, resource, or notification-plugin authority.

The CSP is local-only, Tauri's CSP rewriting remains enabled, the asset protocol remains disabled, and Object.prototype is frozen before frontend code.

Native preference mutations pass semantic validation in addition to serde deserialization. ConfigStore repeats that validation before persistence so internal callers cannot accidentally bypass the same invariants.

Persisted JSON is size-bounded before UTF-8 conversion/deserialization. CPU/memory process attribution is no longer written into Activity event details, and native notifications do not expose process names or exact readings.

CI pins its GitHub Actions to commit SHAs and runs npm + RustSec dependency audits in addition to the existing type/test/build/fmt/clippy/check gates.

See [SECURITY_PRIVACY.md](SECURITY_PRIVACY.md).


## Phase 26 packaging and release engineering

Phase 26 adds a distribution boundary around the existing production application without changing its runtime architecture.

The canonical Windows executable is now named `Byte.exe` through Tauri's `mainBinaryName` configuration while the Rust package remains `byte-desktop`.

Production bundling targets NSIS with current-user installation. Downgrades are explicitly disabled. The installer uses Byte's production icon, a silent WebView2 download bootstrapper when required, and an NSIS post-uninstall hook that removes Byte's separate HKCU Run startup entry.

Packaging certification is independent of ordinary compilation CI. The Windows packaging workflow creates the actual NSIS installer, installs it silently, exercises reinstall, verifies the installed executable version, uninstalls it, and verifies that both installer registration and startup registration are gone.

Tagged release builds additionally compare the tag to all three project version declarations, reject non-increasing release versions, optionally import a Windows signing certificate, certify against the previous standardized installer when available, generate a portable ZIP plus release manifest and SHA-256 checksums, and publish those exact files to GitHub Releases only after certification passes.

See [PACKAGING_RELEASE.md](PACKAGING_RELEASE.md).
