# Byte Architecture — Phase 5 Baseline

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

On Windows, WindowsTelemetrySource uses sysinfo 0.39.6 for CPU, RAM, disks, networks, and best-effort component temperature, plus GetSystemPowerStatus for battery state.

See [TELEMETRY.md](TELEMETRY.md).

## Sampling and lifecycle

CPU, memory, and network are sampled at the 1.5-second base cadence. Slower-changing or more expensive signals use cached values and slower refresh intervals.

LifecycleCoordinator owns ACTIVE, FULLSCREEN_REDUCED, LOCKED, DISPLAY_SLEEP, SYSTEM_SLEEP, and SHUTTING_DOWN. It uses a condition variable so telemetry can block during display/system sleep rather than wake periodically.

The telemetry worker is owned by AppState and joined during explicit quit.

## Human-friendly diagnostics

Phase 6 adds a stateful DiagnosticEngine after telemetry normalization and before the snapshot cache.

The pipeline is now:

Windows telemetry source → normalized/smoothed snapshot → diagnostic engine → cached SystemSnapshot → frontend.

The diagnostic engine owns sustained-condition timing, hysteresis, recovery, issue priority, process culprit attribution, confidence, and safe recommendations. It never performs destructive system actions.

Process scanning is lazy and only runs while CPU or memory is in a high candidate state or has an active issue. Diagnostic thresholds and behavior are documented in [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Input privacy

The Windows input boundary exposes keyboard activity, left click, right click, and scroll only. Its public event type cannot represent key identity or text.

## Windows

- companion: transparent desktop surface.
- quick-panel: compact status surface.
- main: normal application window.

Closing main hides it so tray/companion operation continues.

## Persistence

ByteConfig is versioned. Invalid config is quarantined and Byte recovers to defaults. Writes use a same-directory temporary file before persistence.

## IPC

Snapshot reads are cache-only. There is no frontend hardware polling, arbitrary shell command, generic filesystem bridge, cleaner, RAM trimmer, or process-kill IPC.

## Assets

Characters and habitats remain manifest-driven. Phase 5 does not change the Phase 3 visual contract.

## Validation

Windows CI runs frontend type checks/tests/build plus Rust format/tests/clippy/check. Telemetry fixtures cover healthy data, changing load, and absent optional sensors.
