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

Smart Notifications are an explicit opt-in on a fresh install and default off. Native alerts remain suppressed until onboarding has completed even if a stored preference is enabled.

### 3 — Character and habitat

The user chooses one of the four production characters and one of the six production habitats. Character changes begin from that character's authored default palette.

The full Customization Studio remains available later.

### 4 — Desktop presence

The user chooses Habitat, Perch, Mini, or Edge and may enable:

- Start Byte with Windows
- Hide during fullscreen

Finishing writes the existing CompanionPreferences and AppPreferences; there is no separate onboarding profile.

On a fresh install, Byte does not start desktop-awareness, telemetry, or global-input background workers until this completion is persisted. Telemetry sources are also constructed lazily, so disk/network/battery/thermal enumeration does not happen behind the onboarding screen.

## Migration

ByteConfig schema version is now **9**.

Existing schema versions remain directly migratable from schema 1 through schema 8. Phase 28 adds the serde-defaulted Stable release channel while preserving onboarding and all existing preferences. This prevents an upgrade from interrupting or resetting an already configured user.

## Windows startup

Launch-at-startup is now real rather than a stored preference.

Byte writes only the current user's Windows Run key. Disabling the setting removes Byte's own value.

The app also reapplies the stored startup preference during launch so an executable move/update can refresh the registered path.

## Desktop awareness

Phase 22 expands the original fullscreen setting into a complete desktop-awareness layer.

Settings now exposes:

- Hide for fullscreen games & video
- Hide during presentation mode
- Keep Byte out of screen capture
- optional foreground-app exclusions
- a live awareness status readout

Lock/not-present and console display-off hiding are always active safety/convenience behavior.

Fullscreen recognition combines monitor geometry with Windows fullscreen Direct3D state. Presentation handling uses Windows presentation/busy state. Ordinary maximized windows are not treated as fullscreen merely because they fill the normal work area.

Byte's own foreground windows preserve any existing external suppression instead of accidentally restoring the companion over the fullscreen/presentation app behind them.

Screen capture uses Windows capture exclusion rather than maintaining a list of conferencing/recording tools.

See [DESKTOP_AWARENESS.md](DESKTOP_AWARENESS.md).

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

Settings shows the installed Byte version and stores one local release-channel choice:

- **Stable** → `https://github.com/thiepn/byte/releases/latest`
- **Beta** → `https://github.com/thiepn/byte/releases`

**Check for updates** explicitly opens the corresponding fixed allowlisted destination in the browser. There is no generic URL/shell field, background update polling, automatic download, or forced installation.

Changing channels does not contact GitHub by itself.

## Re-running onboarding

Settings can reopen onboarding by setting `onboarding_completed = false`.

This does not delete customization or collection progress. An already-running installation keeps its background worker objects, but telemetry sampling and anonymous input capture are gated off while onboarding is open. Completing the flow again reuses the existing workers idempotently instead of starting duplicates.

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

Phase 28 supersedes the old update note with the explicit Stable/Beta maintenance model.
