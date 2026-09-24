# Byte Habitat Production — Phase 12

Phase 12 promotes all six Byte habitats from runtime foundations to production scenes.

## Production bar

Every production habitat now has:

- a distinct authored composition rather than a shared programmer-art template
- at least 14 habitat layers and a substantially denser primitive count
- back, midground, ground, and foreground depth
- explicit local-time variants through time-gated scene layers
- four complete semantic palettes: morning, day, evening, and night
- a grounded character-safe focal area
- a minimal Perch treatment that does not duplicate the full room
- all seven semantic system reactions represented visually
- bounded ambient particles using the existing shared 12 FPS scheduler
- six fixed decoration slots reserved for Phase 13 customization

## Production primitive vocabulary

Phase 12 adds two data-driven primitives to the existing rectangle, circle, and line vocabulary:

- ELLIPSE for clouds, rugs, rocks, glows, fish, and soft silhouettes
- POLYGON for hills, furniture silhouettes, spacecraft/platform shapes, fins, and solar panels

Layers may also specify a time array and only render for matching MORNING, DAY, EVENING, or NIGHT state.

The renderer remains manifest-driven. No habitat contains a dedicated animation loop or Svelte-specific scene code.

## Habitat identities

### Meadow

Rolling hills, tree line, wildflowers, foreground grass, sun/moon variants, pollen/fireflies, breeze activity, brambles for memory pressure, stone stacks for storage pressure, heat haze, and a charging bloom.

### Cozy Desk

Window light, shelves and books, monitor, desk furniture, lamp, plant, mug, dust, workload bars, memory-book stacks, storage drawers, lamp heat, and cable charging feedback.

### Bedroom

Window and curtains, bed, rug, nightstand, lamp, night stars, alarm activity, clutter pressure, under-bed storage, radiator heat, and soft charging light.

### Space

Nebula layers, planets, star field, station platform, console, thruster activity, asteroid pressure, cargo storage, reactor heat, energy charging ring, and signal particles.

### Aquarium

Water depth, tank frame, rocks, plants, fish silhouettes, sand, bubbles, fish-school workload motion, kelp pressure, storage chest, heater heat, and charging bubbles.

### Rooftop

Time-shifting sky, city skyline and lights, rooftop equipment, antenna, city activity streaks, pressure antenna bars, storage crates, roof heat haze, solar charging, and signal particles.

## Performance constraints

Phase 12 preserves the Phase 11 runtime budget:

- one shared animation scheduler
- 12 FPS scene cadence
- global maximum of 15 habitat particles
- no ambience under reduced motion
- no full habitat in Mini or Edge
- minimal Perch rendering
- no network/weather dependency for scene state

## Certification

Automated tests require every shipped habitat to be marked production, retain all six decoration slots, represent every semantic reaction, include time-specific art, and meet a minimum visual-density threshold.

Next: Phase 13 customization, cosmetics, props, and fixed-slot decorations on top of these production worlds.
