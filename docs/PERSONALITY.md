# Byte Personality & Ambient Behavior — Phase 14

Phase 14 makes Byte feel temperamentally different without adding virtual-pet maintenance.

## Personalities

Byte ships three persistent personality profiles.

### Chill

- longest incidental idle gaps
- strongest bias toward subtle blink/look behavior
- reduced non-semantic habitat ambience
- enters sleep immediately when the native inactivity signal fires
- rapid repeated clicks can produce an annoyed reaction
- calm pointer entry reaction

### Curious

- balanced idle cadence
- increased curious/look-left/look-right weighting
- medium habitat ambience
- performs a curious reaction when inactivity begins, then sleeps after a short wind-down
- repeated click bursts produce surprise
- notices pointer entry with a curious reaction

### Energetic

- shortest idle gaps
- stronger expressive and rare-idle weighting
- full authored habitat ambience
- performs a happy reaction when inactivity begins and stays awake longest before sleeping
- repeated click bursts produce a happy reaction
- pointer entry is greeted more enthusiastically

## Interaction level

The existing Quiet / Normal / Playful setting now tunes incidental expression within the selected personality.

Quiet lengthens idle cadence and reduces non-semantic ambience. Playful shortens idle cadence and allows the full ambient particle budget. It does not change telemetry, alert thresholds, diagnostic priority, or the amount of input data observed.

## Character idle policy

Personality does not replace character-authored idle profiles.

For each active character, Byte starts with that character's original weighted idle set, including its rare_a / rare_b behaviors. The personality policy then:

- rescales the idle delay range
- adjusts existing weights
- adds personality-relevant behaviors only when the character manifest supports them

The resulting profile is installed into the existing CharacterAnimator and remains deterministic for the existing character/day seed.

## Inactivity and sleep

The native input runtime still emits only anonymous IDLE_START / IDLE_END events after its existing inactivity threshold.

Phase 14 interprets those events differently:

- Chill: sleep immediately
- Curious: curious reaction, then sleep after 15 seconds
- Energetic: happy reaction, then sleep after 35 seconds

Quiet shortens the wind-down; Playful lengthens it.

The countdown advances through the shared animation scheduler. There is no extra JavaScript timer loop.

Any resumed anonymous keyboard/click/scroll activity releases the personality sleep state and requests wake. Higher-priority diagnostic/system states can reject personality requests normally.

## Ambient reactions

Phase 14 adds bounded personality-aware details:

- fast typing stop: a subtle personality-specific follow-up
- repeated left-click burst: four anonymous click timestamps inside 900 ms, with a five-second reaction cooldown
- pointer entry: only when the pointer enters Byte's own transparent companion window
- dragging: a post-drag expression based on drag duration
- charging: a one-shot happy reaction only on a false → true charging transition

The first charging snapshot never triggers a startup celebration.

## Habitat ambience

Personality scales only non-reaction particle profiles:

- Chill: reduced ambience
- Curious: medium ambience
- Energetic: full authored ambience

NETWORK, CHARGING, and other semantic reaction particles keep their telemetry-driven intensities. Reduced-motion still disables all moving habitat particles.

The global 15-particle cap remains unchanged.

## Privacy boundary

Phase 14 does not add:

- typed content
- key identity
- scan codes
- mouse coordinates
- cursor history
- per-application behavior tracking
- behavioral analytics
- cloud state

The cursor-following idea from early brainstorming is intentionally implemented only as local pointer-entry awareness. Byte does not poll or store global pointer position.

## Priority and safety

Personality is low priority.

The existing priority order remains:

critical → diagnostic → interaction → system → input → personality → idle.

A cute or expressive personality event therefore cannot suppress a meaningful thermal, memory, battery, storage, or needs-attention state.

## Persistence

Configuration schema v5 adds:

```text
personality: CHILL | CURIOUS | ENERGETIC
```

Curious is the migration-safe default for existing installs. Config versions 1–4 migrate without losing customization, palettes, habitat selection, display mode, window placements, or other companion settings.

Next: Phase 15 — Quick Panel.
