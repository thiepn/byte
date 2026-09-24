# Byte Habitat Rendering Engine

Phase 11 established the reusable world-rendering engine underneath all six Byte habitats. Phase 12 now supplies production habitat art on top of that runtime.

## Scene composition

The companion scene has three visual planes:

1. habitat back canvas
2. character canvas
3. habitat front canvas

All three are driven by Byte's shared animation scheduler. Habitats do not create their own animation loop.

## Habitat manifest

Every habitat defines:

- 256×256 logical canvas
- character grounding anchor
- morning/day/evening/night palettes
- ordered back/front layers
- optional time-gated layers
- supported display modes per layer
- fixed decoration slots
- sparse particle profiles
- semantic reaction profiles
- foundation/production status

Scene geometry remains data rather than hard-coded Svelte markup.

## Primitive vocabulary

The runtime supports rectangles, circles, lines, ellipses, and filled polygons. Ellipses and polygons were added in Phase 12 so production scenes can express clouds, rugs, rocks, fish, hills, furniture silhouettes, spacecraft forms, and solar panels without habitat-specific rendering code.

## Time of day

Byte uses local system time only.

- Morning: 05:00–09:59
- Day: 10:00–16:59
- Evening: 17:00–20:59
- Night: 21:00–04:59

A layer can restrict itself to one or more of these periods. This is used for sun/moon swaps, skyline lights, window stars, and related scene details. No weather or location API is required.

## Character grounding

Every habitat defines one character anchor. Every character animation frame already provides a ground anchor. The scene aligns the current frame's ground anchor to the habitat anchor so sleeping, typing, bouncing, or shifting poses stay planted.

Perch uses the same mechanism against a dedicated perch line. Mini and Edge remain character-focused and do not render full habitat scenery.

## Semantic reactions

Habitat reactions are BUSY, MEMORY_PRESSURE, STORAGE, THERMAL, LOW_BATTERY, CHARGING, and NETWORK.

The mapping from SystemSnapshot to these semantic reactions is centralized. Production habitats choose their own metaphor without seeing raw hardware thresholds.

## Particles

Habitat ambience uses a deterministic bounded particle engine.

- global maximum: 15 particles
- one shared 12 FPS scheduler
- no moving particles under reduced motion
- no habitat particles in Mini or Edge
- Perch remains visually minimal
- network/charging particle counts scale with semantic intensity

## Decoration slots

All six habitats expose six fixed semantic slots: large_background, wall_or_sky, surface_left, surface_right, small_prop, and ambient.

Phase 13 attaches user-selected decorations to these slots. There is deliberately no freeform furniture editor.

## Production habitats

Meadow, Cozy Desk, Bedroom, Space, Aquarium, and Rooftop are all marked production in Phase 12. Each has a distinct composition, time-specific art, reaction metaphors, and bounded ambience while sharing the same renderer contract.

See [HABITAT_PRODUCTION.md](HABITAT_PRODUCTION.md).

## Validation

Automated tests cover manifest validity, palette consistency, production status, all reaction mappings, decoration-slot count, back/front depth, time-specific art, primitive density, local daypart mapping, deterministic particles, the particle cap, and reduced-motion suppression.

Real-device composition still belongs in later visual/release certification.
