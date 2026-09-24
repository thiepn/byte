# Byte Native Windowing & Shell

Phase 7 owns desktop placement and native-window behavior so character and habitat code never needs to reason about monitors, taskbars, or DPI.

## Windows

Byte has three long-lived Tauri windows:

- companion: transparent, undecorated, always-on-top, skipped from the taskbar.
- quick-panel: compact transient information surface positioned beside Byte where possible.
- main: conventional application window; closing it hides it instead of quitting Byte.

## Display modes

The same companion window is reused for all modes.

- Habitat: 240 × 240 logical px at Medium.
- Perch: 170 × 126 logical px at Medium.
- Mini: 116 × 116 logical px at Medium.
- Edge: 132 × 176 logical px at Medium.
- Tray: companion window hidden; telemetry continues.

Small is 80% of the mode size. Large is 120%.

## DPI

Mode sizes are defined in logical pixels and converted to integer physical pixels using the selected monitor's scale factor before being applied.

This keeps the desktop footprint consistent across 100%, 125%, 150%, 175%, and 200% Windows scaling while avoiding fractional physical dimensions.

## Taskbar-aware Perch

Tauri monitor work areas are used rather than full monitor bounds. Windows excludes taskbars from this work area.

Perch defaults to the work area's bottom edge, so Byte visually sits against the taskbar without covering it regardless of whether the taskbar is on the bottom, left, right, or another monitor.

## Placement persistence

Habitat, Perch, Mini, and Edge each store an independent normalized position:

- monitor name
- X fraction inside usable work area
- Y fraction inside usable work area

Coordinates are not stored as raw pixels.

Habitat and Mini restore both axes. Perch preserves only the coordinate along the detected taskbar edge and always snaps the other axis back against that taskbar. Edge preserves vertical placement while always snapping horizontally to the selected left/right edge.

When resolution or DPI changes, Byte reconstructs a safe physical position from the new work area. If the saved monitor no longer exists, Byte falls back to the current monitor, then the primary monitor, then the first available monitor.

Every restored position is clamped to the work area. A stale saved coordinate therefore cannot permanently strand Byte off-screen.

Tray mode has no placement.

## Move Mode

Move Byte is explicit.

Entering Move Mode:

1. disables click-through if active,
2. shows/focuses the companion,
3. displays a small move overlay,
4. allows one native window drag.

When the drag ends, Byte stores the normalized position, reapplies the active display-mode geometry, and exits Move Mode automatically. The Done button does the same. Perch therefore snaps back to the taskbar and Edge snaps back to its selected screen edge after movement.

Normal character clicks do not move the window.

## Click-through

The shell supports native cursor pass-through with Tauri's ignore-cursor-events API.

Click-through is runtime state rather than a hidden permanent configuration. It is always recoverable from the system tray, which remains interactive even when the companion ignores mouse input.

Entering Move Mode automatically disables click-through.

## Quick Panel positioning

When the companion is visible, the Quick Panel attempts to open on Byte's right side. If there is not enough usable space, it opens to the left. Vertical placement is clamped to the current monitor's work area.

When Byte is hidden or in Tray mode, the panel opens near the bottom-right of the primary work area.

## Tray

The tray menu provides:

- Open Byte
- Show / Hide Companion
- Move Byte
- Toggle Click Through
- Display Mode
  - Habitat
  - Perch
  - Mini
  - Edge
  - Tray Only
- Quit Byte

Left-clicking the tray icon opens the Quick Panel.

Explicit Quit stops and joins the telemetry worker before exiting.

## Configuration migration

Phase 7 moves the local config schema from v1 to v2.

Existing v1 config is migrated in place and preserves the user's current companion settings. New placement and Edge fields receive defaults. Unknown future schema versions are rejected and quarantined rather than silently interpreted.

## Deferred work

Fullscreen/presentation auto-hide and Windows lock/display-power event wiring belong to Phase 22. Phase 7 establishes the lifecycle and windowing primitives those features will use.
