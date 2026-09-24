# Byte Lightweight Collection & Unlocks — Phase 19

Phase 19 adds a small optional collection layer to Byte.

The goal is to create occasional pleasant discoveries from normal computer use, not to turn Byte into a progression game.

## Non-negotiable boundary

All core customization from Phases 13 and 18 remains immediately available.

The collection does **not** gate:

- Byte, Mochi, Pip, or Kiwi
- the six production habitats
- the original twelve cosmetics
- the original twelve habitat decorations
- the original eight palettes per character
- personalities
- display modes
- companion size
- core animation behavior

Phase 19 adds only eight optional extras.

## Unlock collection

| Extra | Type | Natural condition |
| --- | --- | --- |
| Night Cap | headwear | 7 days since Byte first joined the user |
| Memory Frame | habitat decoration | 30 days since first use |
| Pixel Headphones | headwear | 2,500 anonymous typing events |
| Aurora | universal palette | 10,000 anonymous typing events |
| Charging Orb | habitat decoration | 5 real charging starts |
| Signal Kite | habitat decoration | a few ordinary, time-separated network moments |
| Rare Idle I | animation replay | naturally encounter `rare_a` |
| Rare Idle II | animation replay | naturally encounter `rare_b` |

Aurora is authored in all four production character manifests, bringing each character to nine total palettes while leaving its existing default unchanged.

## No artificial grind

Phase 19 intentionally has no:

- XP
- levels
- currency
- shop
- premium items
- battle pass
- daily quests
- login streaks
- seasons
- loot boxes
- resource-wasting achievements
- rewards for keeping CPU busy
- rewards for generating unnecessary network traffic
- mandatory uptime

There is no gameplay power attached to collection state.

## Time milestones

CollectionStore records one local `first_seen_epoch_ms`.

Night Cap unlocks after 7 elapsed days. Memory Frame unlocks after 30 elapsed days.

These are elapsed-time milestones, not consecutive-login requirements. Missing a day does not reset anything.

## Typing milestones

Byte already observes anonymous keyboard activity for companion typing reactions.

Phase 19 increments only a numeric counter from that same anonymous signal. It does not know which keys were pressed or what was typed.

Typing progress unlocks:

- Pixel Headphones at 2,500 events
- Aurora at 10,000 events

The count is checkpointed every 250 events and on clean shutdown so Byte does not write the collection file on every keypress.

## Charging milestone

Charging Orb unlocks after five genuine battery → charging transitions.

Launching Byte while the computer is already charging establishes a baseline and does not count as a charging session.

## Network discovery

Signal Kite uses aggregate network throughput already collected by the normal telemetry engine.

A network moment can count only when:

- aggregate throughput is at least 0.5 Mbps
- at least 60 seconds passed since the previous counted moment
- the finite Signal Kite threshold has not already been reached

The Studio does not reveal a numeric network target. The intent is for this extra to appear during ordinary connected use, not to invite streaming/download grinding.

No new network request is made for progression.

## Rare idle discoveries

The production companion already has rare authored idle animations.

When `rare_a` or `rare_b` occurs naturally, the companion records that local discovery once. The corresponding animation then becomes manually replayable in the Studio preview as Rare I or Rare II.

The rare animation is allowed to finish naturally; discovery does not immediately replace it with a celebration animation.

## Persistence

Collection state lives in:

`collection.json`

It contains only local progression counters, discovery IDs, and permanent unlock IDs.

It is not synced, uploaded, or shared.

Unlocks are permanent for that collection file once earned.

## Studio integration

Phase 19 adds a seventh Studio category: **Collection**.

The Collection page shows:

- the eight finite extras
- locked / unlocked state
- plain-language unlock condition
- finite progress for time, typing, and charging milestones
- non-numeric discovery wording for network and rare-idle extras

Unlockable cosmetics, decorations, and Aurora also appear in their normal editing sections. Locked items stay visible but disabled.

When an ordinary typing/charging/network milestone unlock occurs, Byte may perform one small happy reaction. Rare-idle discoveries do not interrupt the animation that caused the discovery.

## Backend enforcement

The Rust preference path rejects attempts to equip known locked Phase 19 extras.

This protects the saved preference state from stale UI state or manually constructed IPC calls while leaving every core Phase 18 item ungated.

## Privacy

Phase 19 adds no new sensitive observation.

It records no:

- typed text
- key identity
- key codes
- app usage tied to typing
- cursor coordinates
- browsing history
- network destinations
- process history
- cloud identity

## Phase boundary

Phase 19 remains intentionally small.

Onboarding, editable application settings, startup integration, notification settings, accessibility controls, and update controls remain **Phase 20 — Onboarding & Settings**.
