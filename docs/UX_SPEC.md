# Byte v1.0 UX Contract

Byte uses progressive disclosure:

1. Look at Byte: understand the broad condition.
2. Click Byte: understand what matters and the likely next action.
3. Open Details: investigate only when desired.

Primary states are CALM, BUSY, STRESSED, and NEEDS_ATTENTION. BUSY is legitimate work and must never be presented as damage by default.

## Primary surfaces

Desktop Companion, Quick Panel, Full Application, Tray Menu, Native Notifications, and First-run Onboarding.

The full app has exactly five main destinations: Overview, Activity, Apps, Customize, Settings.

## Interaction rules

Byte may react to anonymous keyboard activity, clicks, scrolling, local pointer entry, petting, dragging, charging, network activity, workload, and rare idle events. Global cursor coordinates are not collected. Chill, Curious, and Energetic change low-priority pacing and expression while diagnostic meaning remains stable. There are no feeding, cleaning, death, streak, happiness-maintenance, or recurring chore systems.

## Quick Panel

Clicking Byte opens a compact 340×500 explanation-and-action surface. It shows the current product state, at most one primary issue, a compact resource overview, one safe recommended action when available, and essential companion/window controls.

The Quick Panel is cache-only and does not trigger hardware sampling. It closes on focus loss and keeps detailed investigation in the full application rather than growing into a Task Manager replacement.

## Full application

The main application keeps exactly five destinations: Overview, Activity, Apps, Customize, and Settings.

Phase 16 makes Overview and Activity production surfaces. Overview shows current state plus CPU, memory, storage, battery, network, and best-effort thermal context with small bounded session trends. Activity stores meaningful events rather than raw telemetry rows.

Apps uses an explicit on-demand diagnostic scan rather than a background leaderboard. Customize is a first-class Studio with a large animated preview and visual controls; every selection persists immediately without an Apply step. Settings summarizes current configuration without prematurely adding later system-integration controls.

## Alerts

Transient spikes do not notify. Sustained meaningful conditions progress from observation to visual reaction to in-app explanation and only then to native notification when justified. Cooldowns and duplicate suppression are mandatory.

## Windows

Closing the full application hides it instead of quitting Byte. Byte hides by default during fullscreen usage and later lifecycle phases suspend unnecessary work during lock/sleep.


## Lightweight collection

Phase 19 adds one optional Collection section inside the Customization Studio.

Core characters, habitats, Phase 13 cosmetics/decorations, personalities, palettes, and display controls remain immediately available. The collection contains only eight finite extras and does not gate Byte's core experience.

Unlock conditions are based on passive elapsed time or signals Byte already observes for companion reactions. There are no daily-login rewards, streaks, XP bars, currency, shop, battle pass, resource-wasting challenges, or mandatory pet maintenance.

Locked extras remain visible with a plain-language condition. Time, typing, and charging milestones may show simple finite progress. Network and rare-idle discoveries intentionally do not expose grindable numeric targets.


## First-run onboarding

Phase 20 introduces a short four-step setup shown only for new installations:

1. Meet Byte and understand the companion-first product.
2. Review the local-only privacy boundary and optionally disable system monitoring.
3. Choose character and habitat.
4. Choose desktop display mode, startup behavior, and fullscreen behavior.

Completing onboarding persists the ordinary companion/app preferences. Existing schema-v5 installations migrate with onboarding marked complete so upgrades do not unexpectedly reopen first-run setup.

## Settings

Settings is now editable rather than a read-only summary.

It contains:

- Windows startup
- fullscreen auto-hide
- system monitoring
- local Activity history
- Windows notifications
- sound master switch
- reduced motion
- high contrast
- 100 / 110 / 125% application text scale
- current application version and fixed GitHub Releases destination
- Activity-history clearing
- re-run onboarding
- handoff to the Customization Studio

There is still no threshold editor, optimizer control panel, generic shell launcher, or advanced sensor configuration.


## Smart Notifications

Phase 21 keeps notifications intentionally rare.

Eligible OS notifications are limited to:

- critical memory pressure
- serious sustained thermal pressure
- critically low storage
- critically low battery while not charging
- a confidently attributed process that remains under critical CPU pressure for about 10 minutes

Normal CPU load, normal memory use, ordinary network activity, collection progress, and routine system changes do not produce health notifications.

Smart Notifications evaluate all active diagnostic issues rather than only the primary issue shown in compact UI.

Users can disable individual categories, enable notification Quiet mode, or snooze for one hour, four hours, or until 08:00 the next local day. Snooze/Quiet mode affects OS notifications only; in-app diagnostic state remains current.


## Fullscreen, gaming & presentation awareness

Phase 22 centralizes whether Byte is allowed to appear on the desktop.

Byte suppresses the companion and Quick Panel when appropriate for:

- fullscreen applications and games
- Windows presentation/busy state when configured
- the locked/not-present desktop
- console display-off state
- user-configured foreground application exclusions

The companion remembers whether it was visible before a suppression interval and restores only when appropriate. Tray mode and an already-user-hidden companion are not overridden.

Showing Byte, opening the Quick Panel, entering Move Mode, or reapplying window layout cannot bypass an active suppression state.

Screen sharing is handled with Windows capture exclusion rather than a hard-coded list of conferencing/recording apps. The option is enabled by default and applies to Byte's trusted windows. It is a privacy/convenience measure for supported Windows capture paths, not a DRM or absolute content-security mechanism.

Custom foreground exclusions store normalized executable names only. Byte does not store executable paths or window titles for this feature.

Display-off currently guarantees visibility suppression. Phase 23 owns deeper worker/rendering/telemetry suspension and power-performance coordination.
