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

## Alerts

Transient spikes do not notify. Sustained meaningful conditions progress from observation to visual reaction to in-app explanation and only then to native notification when justified. Cooldowns and duplicate suppression are mandatory.

## Windows

Closing the full application hides it instead of quitting Byte. Byte hides by default during fullscreen usage and later lifecycle phases suspend unnecessary work during lock/sleep.
