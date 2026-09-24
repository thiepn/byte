# Byte Habitat Rendering Engine

Phase 11 establishes the reusable world-rendering engine underneath all six Byte habitats.

Final production habitat art is Phase 12. Phase 11 focuses on the runtime contract, layering, grounding, time-of-day, sparse ambience, and semantic system reactions.

## Scene composition

The companion scene now has three visual planes:

1. habitat back canvas
2. character canvas
3. habitat front canvas

This lets scenery appear behind Byte while foreground plants, furniture, bubbles, boxes, and effects can appear in front.

All three are driven by Byte's existing shared animation scheduler. Habitats do not create their own animation loop.

## Habitat manifest

Every habitat defines:

- 256×256 logical canvas
- character grounding anchor
- morning/day/evening/night palettes
- ordered back/front layers
- supported display modes per layer
- fixed decoration slots
- sparse particle profiles
- semantic reaction profiles
- foundation/production status

Scene geometry is data rather than hard-coded Svelte markup.

## Palette slots

Habitat primitives reference semantic color slots rather than literal colors.

Examples:

- sky
- far
- ground
- ground2
- accent
- warm
- shadow
- light
- signal
- low

Each time-of-day palette supplies the actual colors.

The same geometry can therefore move from morning to day to evening to night without duplicating layers.

## Time of day

Byte uses local system time only.

- Morning: 05:00–09:59
- Day: 10:00–16:59
- Evening: 17:00–20:59
- Night: 21:00–04:59

No weather or location API is required.

## Character grounding

Every habitat defines one character anchor.

Every character animation frame already provides a ground anchor.

The scene aligns the current frame's ground anchor to the habitat anchor. This means sleeping, typing, bouncing, or shifting poses remain planted on the same surface instead of visually floating as sprite geometry changes.

Perch mode uses the same mechanism against a dedicated perch line.

Mini and Edge remain character-focused and do not render full habitat scenery.

## Layering

Each layer defines:

- BACK or FRONT plane
- order
- supported display modes
- optional opacity
- optional semantic reaction
- one or more pixel-style primitives

A reaction layer is drawn proportionally to its current semantic intensity.

This makes system telemetry influence a habitat without the habitat needing to know raw CPU percentages or diagnostic thresholds.

## System reaction contract

Habitat reactions are:

- BUSY
- MEMORY_PRESSURE
- THERMAL
- LOW_BATTERY
- CHARGING
- NETWORK

The mapping from SystemSnapshot to these semantic reactions is centralized.

Examples:

- BUSY is driven by legitimate workload intensity.
- MEMORY_PRESSURE comes from a real memory diagnostic issue.
- THERMAL comes from a real thermal issue.
- LOW_BATTERY and CHARGING are mutually meaningful battery states.
- NETWORK uses aggregate throughput as playful activity, never as a warning.

Phase 12 can give each environment a different visual metaphor without changing telemetry or diagnostics.

## Particles

Habitat ambience uses a deterministic bounded particle engine.

Rules:

- global maximum: 15 particles
- each profile has its own smaller maximum
- no particle updates outside the shared scheduler
- no particles in reduced-motion mode
- no habitat particles in Mini or Edge
- Perch remains visually minimal
- network/charging particle counts scale with semantic intensity

Example foundation ambience:

- Meadow: fireflies
- Cozy Desk: dust motes
- Bedroom: window stars
- Space: stars
- Aquarium: bubbles
- Rooftop: city glints

Particles are seeded by habitat + session day so testing is reproducible.

## Decoration slots

All six habitats expose six fixed semantic slots:

- large_background
- wall_or_sky
- surface_left
- surface_right
- small_prop
- ambient

Each slot has:

- x/y anchor
- plane
- order

Phase 13 will attach actual user-selected decorations to these slots.

There is deliberately no freeform furniture editor.

## Display modes

Habitat:

- full back/rear/ground/foreground/effects scene
- character aligned to habitat anchor
- ambience and semantic reactions

Perch:

- only the small perch platform and relevant lightweight status effects
- character remains grounded

Mini:

- character only

Edge:

- character only

Tray:

- no companion rendering

## Six foundation habitats

Phase 11 ships valid runtime foundations for:

- Meadow
- Cozy Desk
- Bedroom
- Space
- Aquarium
- Rooftop

They already have distinct geometry and four distinct time palettes, but they are marked foundation rather than production. Phase 12 replaces/refines their visual content to the final art-quality bar without changing the renderer architecture.

## Reduced motion

Reduced motion keeps static environmental meaning but disables moving particles.

Reaction layers still communicate busy/heat/memory/battery conditions without requiring motion.

## Validation

Automated tests cover:

- all six habitat manifests
- palette consistency
- required reactions
- decoration-slot count
- back/front layer presence
- local daypart mapping
- network/battery/system reaction mapping
- deterministic particles
- particle cap
- reduced-motion particle suppression

The final visual-quality judgment remains part of Phase 12 and the later visual audit.
