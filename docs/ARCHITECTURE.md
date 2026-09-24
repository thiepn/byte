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
