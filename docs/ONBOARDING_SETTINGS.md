# Byte Onboarding & Settings — Phase 20

Phase 20 completes Byte's first-run and editable application-settings experience.

## First run

A fresh Byte configuration starts with `onboarding_completed = false`. The main window opens automatically and presents four short steps intended to take about 30 seconds.

### 1 — Meet Byte

Explains the product in one sentence: Byte is a desktop companion first and a system-health explainer second.

### 2 — Privacy

Explains what Byte can use locally:

- CPU
- memory
- storage
- battery
- aggregate network activity
- best-effort temperature
- anonymous input activity

And what Byte does not collect:

- typed text
- key identity
- browsing history
- cursor coordinates/history
- cloud analytics
- uploaded process names

System monitoring can be disabled here.

### 3 — Character and habitat

The user chooses one of the four production characters and one of the six production habitats. Character changes begin from that character's authored default palette.

The full Customization Studio remains available later.

### 4 — Desktop presence

The user chooses Habitat, Perch, Mini, or Edge and may enable:

- Start Byte with Windows
- Hide during fullscreen

Finishing writes the existing CompanionPreferences and AppPreferences; there is no separate onboarding profile.

## Migration

ByteConfig schema version is now **7**.

Existing schema versions 1–5 gain Phase 20 defaults and are migrated with onboarding marked complete. This prevents an upgrade from interrupting an already configured user with first-run setup.

## Windows startup

Launch-at-startup is now real rather than a stored preference.

Byte writes only the current user's Windows Run key. Disabling the setting removes Byte's own value.

The app also reapplies the stored startup preference during launch so an executable move/update can refresh the registered path.

## Fullscreen behavior

Byte now detects foreground fullscreen windows on Windows.

When **Hide during fullscreen** is enabled:

- entering a real fullscreen foreground window hides the companion
- the Quick Panel is dismissed
- leaving fullscreen restores the companion
- Tray mode stays Tray mode
- ordinary maximized windows that leave the taskbar/work area visible are not treated as fullscreen
- Byte's own windows are ignored

The check is lightweight and runs roughly every 750 ms.

## System monitoring

**System monitoring** controls the authoritative Rust telemetry worker.

When off, Byte does not sample CPU/RAM/storage/network/battery/thermal data for system-health explanations. The cached snapshot becomes unavailable instead of pretending old data is current.

Companion customization and non-system features remain usable.

## Activity history

Activity-history collection remains separately controllable.

Settings also exposes **Clear Activity history**, which clears:

- persisted meaningful events
- current in-memory session trends

It does not clear:

- customization
- collection unlocks
- app settings

## Notifications

**Windows notifications** controls native notifications for sustained issues.

Phase 20 notification behavior is intentionally conservative:

- only `NEEDS_ATTENTION`
- one notification per active issue
- duplicate suppression while the same issue remains active
- headline only
- diagnostics still remain available inside Byte

Phase 21 now deepens this into Smart Notifications with per-category routing, cooldowns, duplicate suppression, Quiet mode, snooze, Windows permission status, and cause/action text. See [SMART_NOTIFICATIONS.md](SMART_NOTIFICATIONS.md).

## Sound

Sound remains a master preference. System-health meaning never depends on sound.

## Accessibility

### Reduce motion

Forces reduced companion/UI motion even if Windows itself does not request reduced motion. If either Byte or the OS requests reduced motion, the companion renderer uses reduced-motion animation behavior.

### High contrast

Strengthens surface/text/border contrast across Byte without changing semantic status meaning.

### Text scale

The application supports:

- 100%
- 110%
- 125%

The main UI scales while authored pixel-art companion proportions remain controlled by the companion size setting.

Keyboard focus rings, native buttons, semantic labels, and screen-reader-friendly text remain first-class. Byte does not use flashing effects for status.

## Updates

Settings shows the installed Byte version and exposes one fixed trusted **Open releases** destination:

`https://github.com/thiepn/byte/releases`

There is no generic URL/shell field and no background update daemon.

Signed release/update installation infrastructure remains part of the later release/update-hardening roadmap.

## Re-running onboarding

Settings can reopen onboarding by setting `onboarding_completed = false`.

This does not delete customization or collection progress.

## Acceptance

Phase 20 is complete when an unfamiliar user can:

- understand what Byte is
- understand the privacy boundary
- choose a character/habitat/display mode
- finish setup without external instructions
- change startup/fullscreen/monitoring/notification/accessibility settings later
- clear Activity history
- reopen onboarding
- reach customization and releases from Settings

Next: **Phase 22 — Fullscreen, Gaming & Presentation Awareness**.
