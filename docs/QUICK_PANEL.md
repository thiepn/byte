# Byte Quick Panel — Phase 15

Phase 15 turns the compact panel beside Byte into the primary explanation-and-action surface.

The panel follows Byte's progressive-disclosure contract:

1. glance at Byte for broad state
2. click Byte for the compact explanation
3. open the full app only when deeper investigation is useful

## Live cache-only status

The panel reads the authoritative cached SystemSnapshot. It never starts a second hardware sampler.

While visible, it refreshes the cached snapshot, preferences, and shell state every two seconds. Refreshing stops when the panel closes or loses focus.

The header exposes snapshot freshness so stale cached data is not presented as if it were current.

## Status summary

The top card presents one of the existing product states:

- CALM
- BUSY
- STRESSED
- NEEDS_ATTENTION

BUSY retains neutral informational styling rather than warning styling.

## Primary issue

When a diagnostic issue exists, the panel shows:

- headline
- plain-language explanation
- severity
- likely contributor, only when the diagnostic engine supplied one
- culprit confidence
- secondary issue count
- one narrow recommended action

The panel never invents a culprit or recommendation.

## Safe Windows actions

Phase 15 implements the existing recommended-action enum using a fixed native allowlist:

- OPEN_TASK_MANAGER → Task Manager
- OPEN_STORAGE_SETTINGS → `ms-settings:storagesense`
- OPEN_BATTERY_SETTINGS → `ms-settings:batterysaver-settings`
- VIEW_DETAILS → Byte's full application

There is no generic command, URI, shell, or executable IPC endpoint. The frontend can only request one of the serialized RecommendedActionKind values.

## Resource overview

The compact panel shows CPU, memory, storage, and battery as small semantic cards. It also shows aggregate network throughput and best-effort temperature when available.

This is deliberately not a sensor dashboard. It does not expose:

- per-core CPU
- voltages
- fan controls
- SMART data
- benchmark information
- raw process tables

## Companion controls

The panel offers fast access to:

- Habitat
- Perch
- Mini
- Edge
- Tray mode
- Move Mode
- click-through toggle
- full Byte application

These use the existing persisted shell services instead of duplicating window state in Svelte.

## Window behavior

The Quick Panel remains an always-on-top compact surface positioned beside the companion when possible and inside the monitor work area otherwise.

Its production logical size is 340×500. It closes on explicit close, focus loss, entering Move Mode, entering Tray mode, or launching a recommended destination.

## Accessibility

- all controls are real buttons
- focus-visible styling comes from the global design system
- state is communicated by text as well as color
- resource bars are supplementary rather than the only state indicator
- reduced-motion users receive no extra animation requirement

Next: Phase 16 — Full Application Surfaces.
