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

The production character layer uses authored pixel sprite atlases for Byte, Mochi, Pip, and Kiwi. See [CHARACTERS.md](CHARACTERS.md).

## Production habitats

Phase 12 promotes all six scenes to production: denser authored compositions, local-time palettes, time-specific layers, richer ellipse/polygon geometry, habitat-specific telemetry metaphors, grounded character staging, fixed decoration slots, and bounded ambience.

The scenes remain cozy pixel-art worlds rather than technical dashboards. System health is communicated through environmental metaphors, not numeric overlays.

See [HABITAT_PRODUCTION.md](HABITAT_PRODUCTION.md).


## Customization Studio

Phase 18 treats customization as a playful visual surface rather than conventional settings.

The production layout uses a large sticky preview beside a category editor. Choice cards emphasize imagery, color, habitat tone, and selected state. Dense form rows and generic dropdowns are avoided.

The preview reuses Byte's actual animation, habitat, cosmetic, decoration, palette, personality, and particle systems so the Studio is representative of the desktop companion rather than a separate mock renderer.


## Accessibility and resilient presentation

Phase 24 makes accessibility part of the visual contract rather than a separate skin.

- Every keyboard-operable control has a visible focus state.
- Selected state is exposed structurally and never relies only on fill color.
- Windows forced-colors may replace authored surface/status colors; semantic text remains authoritative.
- Byte High contrast removes decorative shadows and strengthens status colors/borders.
- Main and Quick Panel UI support 100%, 110%, and 125% text scale with wrapping/responsive fallbacks.
- Companion pixel art remains at authored proportions when application text is enlarged.
- Reduced-motion preferences remove decorative transition timing without hiding information.
- Dense diagnostic data scrolls rather than clipping at constrained effective widths.

See [ACCESSIBILITY_RESILIENCE.md](ACCESSIBILITY_RESILIENCE.md).
