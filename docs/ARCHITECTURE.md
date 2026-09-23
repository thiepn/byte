# Byte Architecture — Phase 4 Baseline

## Boundaries

Byte has four layers:

1. Platform core: Windows integration, lifecycle, anonymous input, telemetry.
2. Application core: state, diagnostics, activity, configuration.
3. Companion engine: character/habitat/cosmetic behavior and rendering.
4. Product UI: Quick Panel and main application features.

The frontend never independently polls hardware. The backend never selects sprite frames.

## State and telemetry

Rust owns the authoritative system snapshot and persisted configuration. Frontend IPC reads cached state. `TelemetrySource` is the sampling boundary; `MockTelemetrySource` provides deterministic development data. Phase 5 replaces the mock with the single production producer.

## Lifecycle

`LifecycleCoordinator` owns ACTIVE, FULLSCREEN_REDUCED, LOCKED, DISPLAY_SLEEP, SYSTEM_SLEEP, and SHUTTING_DOWN. Future periodic work must share this owner rather than creating independent forever-loops.

## Input privacy

The Windows boundary exposes keyboard activity, left click, right click, and scroll only. Its public event type cannot represent key identity or text.

## Windows

- `companion`: transparent desktop surface.
- `quick-panel`: compact status surface.
- `main`: normal application window.

Closing `main` hides it so tray/companion operation continues.

## Persistence

`ByteConfig` is versioned. Invalid config is quarantined and Byte recovers to defaults. Writes use a same-directory temporary file before persistence.

## IPC

Phase 4 permits only snapshot/preferences reads, companion preference updates, known Byte window show/hide actions, and explicit quit. There is no arbitrary shell command, generic filesystem bridge, cleaner, RAM trimmer, or process-kill IPC.

## Assets

Characters and habitats are manifest-driven. Final atlases arrive later; Phase 4 includes placeholder manifests and a CSS dev companion.

## Validation

Windows CI runs frontend type checks/tests/build plus Rust format/tests/clippy/check.
