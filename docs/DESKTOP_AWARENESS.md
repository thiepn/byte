# Byte Desktop Awareness — Phase 22

Phase 22 ensures Byte does not unexpectedly appear over a fullscreen game, movie, presentation, lock screen, sleeping display, or user-selected application.

The feature is implemented as one centralized desktop-awareness state rather than scattered fullscreen checks.

## Goals

Byte should:

- disappear before it becomes distracting
- never fight a fullscreen foreground application
- restore only when Byte itself hid a previously visible companion
- remain in Tray when the user chose Tray
- respect explicit user hiding
- keep Quick Panel and Move Mode from bypassing suppression
- avoid appearing in supported Windows screen captures by default
- avoid storing foreground paths or window titles

## Awareness signals

### Foreground geometry

Byte compares the foreground window's outer rectangle with the containing monitor rectangle.

An exact/full-monitor match within a small tolerance is treated as fullscreen.

A normal maximized application that leaves the taskbar/work-area boundary visible is not treated as fullscreen from geometry alone.

### Windows notification state

Byte also queries Windows user-notification state.

This gives native indications for:

- fullscreen Direct3D use
- presentation mode
- busy / interruption-sensitive state
- user not present / locked-style state

The signal complements geometry rather than replacing it.

### Display power

A message-only Windows window registers for `GUID_CONSOLE_DISPLAY_STATE`.

When Windows reports the console display as off, Byte immediately suppresses its visible companion and Quick Panel.

Display dim/on clears that reason when no higher-priority awareness condition remains.

Phase 22 uses this only for visibility. Worker suspension belongs to Phase 23.

### Foreground app exclusions

Users can optionally add executable names for applications where Byte should remain locally hidden whenever that app is foreground, even if it is windowed.

Examples might include a specific recording, kiosk, creative, or game tool chosen by the user.

The backend permits at most 32 entries.

Input rules:

- executable name only
- trailing `.exe` is optional
- case-insensitive normalization
- no path separators
- no drive-colon/path input
- duplicates removed

Byte transiently queries a process image path only to extract the executable stem. The path itself is discarded and never crosses IPC.

## Suppression priority

Visibility reasons are resolved in this order:

1. display off
2. user not present / locked
3. explicit foreground-app exclusion
4. presentation
5. fullscreen

The reason is exposed in Settings through a read-only `DesktopAwarenessSnapshot`.

## Centralized shell guard

Suppression is enforced inside the windowing layer, not only inside the watcher.

During suppression:

- `show_companion` is a no-op
- `show_quick_panel` is a no-op
- Move Mode cannot begin
- companion layout updates keep the native window hidden
- the Quick Panel is dismissed when suppression starts
- Smart Notifications are suppressed

This prevents tray actions, customization, mode changes, and incidental UI events from re-showing Byte over a game or presentation.

## Restore semantics

When suppression begins, AppState records whether the companion was actually visible.

On exit:

- if Byte hid a visible companion, it restores it through normal mode/layout logic
- if Byte was already hidden, it stays hidden
- Tray mode remains Tray
- saved placement and DPI-aware layout are reused

A transition from one suppression reason directly into another does not cause a visible flash between them.

## Byte foreground race

Opening Byte's main window or Settings can temporarily make a Byte-owned window foreground while a fullscreen/presentation app remains behind it.

Phase 22 detects this case and preserves the existing external suppression state until a non-Byte foreground observation can establish that the external condition truly ended.

This prevents opening Settings from causing the companion to reappear over the game/presentation behind the main window.

## Screen capture

**Keep Byte out of screen capture** is enabled by default.

Byte applies Windows `WDA_EXCLUDEFROMCAPTURE` display affinity to:

- companion
- Quick Panel
- main application window

This avoids brittle heuristics such as assuming that the presence of Teams, Zoom, Discord, OBS, or another recorder means screen sharing is active.

The option is best-effort and depends on supported Windows capture paths. It is a convenience/privacy feature, not DRM and not an absolute security boundary.

Disabling it restores ordinary capture behavior.

## Startup behavior

The companion window starts hidden at the Tauri configuration level.

Byte performs an initial desktop-awareness observation before normal companion initialization is allowed to reveal the window.

This avoids a brief startup flash over a game or presentation when Byte launches with Windows.

## Settings

The Windows & desktop Settings section now contains:

- Hide for fullscreen games & video
- Hide during presentation mode
- Keep Byte out of screen capture
- live Desktop awareness status
- foreground-app exclusion editor
- always-on lock & display-off protection explanation

The awareness status can show:

- Byte is available on the desktop
- Fullscreen app
- Presentation mode
- Session locked
- Display asleep
- Excluded foreground app

## Notifications

Smart Notification OS delivery pauses during any active visibility suppression.

Diagnostic tracking continues. A suppressed alert is not recorded as sent.

If the issue remains eligible after the game/presentation/lock/display-off condition ends, normal Smart Notification evaluation can notify then.

## Lifecycle boundary

Phase 22 uses existing lifecycle states to describe fullscreen/presentation/lock context where appropriate.

It deliberately does **not** implement the full power-efficiency policy.

In particular, display-off currently means:

- hide Byte
- keep the shell from reappearing
- report Display asleep in awareness state

It does not yet mean:

- stop telemetry worker
- stop input worker
- suspend the renderer
- change sampling cadence
- consolidate all workers under a shared sleep coordinator

Those belong to **Phase 23 — Power & Performance Hardening**.

## Acceptance

Phase 22 is complete when:

- Byte stays hidden across fullscreen games/video and configured presentation state
- lock/not-present and display-off cannot leave the companion visible
- no tray/UI/layout operation bypasses suppression
- restore occurs only when Byte should restore
- switching suppression reasons causes no reveal flash
- opening Byte's own UI does not clear an external suppression condition
- user-selected foreground apps work by normalized name only
- screen-capture exclusion is applied consistently to Byte windows
- Smart Notifications do not interrupt suppressed contexts
- all behavior remains local

Next: **Phase 23 — Power & Performance Hardening**.
