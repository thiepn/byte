# Byte v1.0 Visual Contract

Art direction: modern cozy pixel art for the companion world, paired with a clean modern Windows interface.

## Production companion

- Character native art target: 48x48 px.
- Animation canvas: up to 64x64 px.
- Nearest-neighbor rendering.
- About 8-14 deliberate colors per character.
- Soft top-left lighting.
- Shared semantic animation vocabulary.
- Cosmetics attach through semantic anchors.

Characters: Byte, Mochi, Pip, Kiwi.
Habitats: Meadow, Cozy Desk, Bedroom, Space, Aquarium, Rooftop.
Personalities: Chill, Curious, Energetic.

## Application UI

- Segoe UI Variable/system sans.
- 4px spacing base.
- 8/10/14/18px radius hierarchy.
- Calm semantic status colors.
- No permanent cyberpunk HUD, reticles, tiny monospace telemetry, giant gauges, or excessive neon.

The production character layer now uses authored pixel sprite atlases for Byte, Mochi, Pip, and Kiwi. The old CSS/programmer-art placeholder has been removed. See [CHARACTERS.md](CHARACTERS.md).


## Habitat runtime foundation

Phase 11 implements the visual scene architecture for Meadow, Cozy Desk, Bedroom, Space, Aquarium, and Rooftop. Each uses a 256×256 logical pixel canvas, four local-time palettes, fixed decoration slots, semantic system reactions, sparse bounded ambience, and explicit back/front planes.

The six Phase 11 scenes are runtime foundations. Phase 12 is responsible for final habitat art polish and production certification.
