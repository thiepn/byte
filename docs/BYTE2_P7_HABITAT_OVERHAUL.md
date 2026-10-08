# Byte 2.0 — P7 Six Habitat Art Overhaul

**Status:** Production-manifest scenery committed; automated art gallery and regression gates. Physical Windows appearance review remains outstanding.

## Goal

Make the desktop friend feel as though it lives inside six different little worlds rather than six geometric telemetry backgrounds. The environment must frame the actual character without hiding its face, take on morning/day/evening/night colors, and communicate health changes quietly.

P7 expands existing production **256×256 editable vector-pixel manifest art**; it does not load photographic images, add SVG image-fetch plumbing, replace the shared renderer, add another RAF/timer, or alter the seven reaction semantics. Art uses authored integer-positioned palette rectangles, small polygons/ellipses and carefully placed light details. It remains time-palette driven and local.

## Actual shipped scene enhancements

| World | P7 authored visual additions | Extra primitives | Preserved |
|---|---|---:|---|
| **Meadow** | Pixel birch canopy and tree trunks, flower clusters, dew, meadow grass, mushrooms and close reeds | 185 | Rolling hills, breezes, fireflies, flowers, meadow health signals |
| **Cozy Desk** | Tiny book-spine shelving, workspace side props, desk tiles, pencil-note forms, lamp/window light | 109 | Monitor, coffee/desk elements, workload, books/storage warnings |
| **Bedroom** | Framed art, reading shelf, patchwork bed fabric, plants/rug trim, soft time-of-day lighting | 103 | Bed, curtains, clock, radiator, sleep/warmth signals |
| **Space** | Constellation points, side bulkheads, control light strips, deck tiles and relay lights | 115 | Planets, station floor, energy, signal and cargo reactions |
| **Aquarium** | Coral gardens, individual tiny fish, stationary bubbles, water beams, shells and scattered pebbles | 126 | Tank, sand, sea plants, low-energy/charging bubbles |
| **Rooftop** | Dense small city facade windows, raised planter boxes, railing highlights, masonry and evening glow | 194 | Layered skyline, antenna and solar/thermal reactions |

**Total:** 832 additional original scenery primitives, across **54 new authored layers** (eight scene-specific layers plus one character-contact shadow layer per scene). Every scene has an explicit P7 layer for **each of four local-time states** (MORNING, DAY, EVENING, NIGHT). Existing canvas, focal ground point, decoration slots and state layers remain unchanged. The six visuals intentionally have different compositional rhythms; the center foreground is reserved for the mascot.

## Character protection and depth

- Every habitat gets a soft **BACK-plane** contact shadow directly below its original `characterAnchor`, at a low 0.32 opacity.
- New scenery uses existing `BACK` and `FRONT` planes. Close reeds/tiles/shells/brick borders use FRONT and avoid the character's central face and upper body.
- The character continues to be rendered at z-index 2, between back-plane and front-plane canvases. Source-art gallery preview uses this same order.
- Back-plane additions live between order 7 and 34; FRONT trim is order 76/77 so key foreground reactions and props remain legible.
- Six user decoration slots, all seven health reaction layers and particles, and minimal Perch support remain present. Mini/Edge/Tray have **no full habitat**.

## Runtime and performance

- No new image decoding, loaders, network requests, saved-data schema migration or dependencies.
- Same `HabitatCanvasRenderer` data-driven rendering and 256×256 canvases with image smoothing disabled.
- No ambient timer, extra animation loop, or animated particle process. Existing system uses the shared ~12 FPS scheduler.
- The total number of authored manifest primitives is bounded below 320 per world; global particle budget remains 15, and reduced-motion suspension policy is unchanged.
- No changes to click-through/visibility, native Window placement, input monitoring or privacy.

## Production evidence & automated regression

- `src/companion/habitats/p7-worlds.test.ts`: validates all six distinct high-detail scene compositions, at least 100 added authored primitives per world, at least 50 source rectangles, four time-specific art layers per scene, quiet back-plane contact shadows at the original anchor, front-plane face clearance, preserved reaction/decoration/Perch contracts and a bounded total primitive count.
- `scripts/build-p7-habitat-gallery.mjs`: creates a fully **offline** gallery HTML containing real committed habitat manifests and exact production Byte/Mochi/Pip/Kiwi PNG previews. It uses the same Canvas2D shape vocabulary and layer order as the app. It provides all 6 × 4 = **24 source scenes**, mascot switching, simulation of each PC-health reaction and an option to hide the sprite.
- The existing `Byte P1 Visual Evidence` GitHub Actions workflow now runs `node --check` on this generator, builds the gallery and uploads it alongside source-contact-sheet evidence.
- The existing visual asset, registry, renderer, habitat, identity, frontend, Rust and Windows packaging tests continue to gate merge.

## Visual review and acceptance

1. Open the PR's **Byte P1 Visual Evidence** artifact, file `byte2-p7-habitat-gallery.html`, offline in Edge/Chrome. Compare all 24 scenes. Toggle four different mascots, hide the sprite to inspect contact shadows, and preview the seven system reactions.
2. In a packaged Windows Byte build, change habitat under Studio, inspect **HABITAT**, **PERCH**, **MINI**, **EDGE**, **TRAY**, and 100–200% monitor scaling. Measure whether the character occupies the correct comfortable portion of the scene and sits on the shadow without floating.
3. Verify nighttime light remains legible; warm scenes should still be cute in the morning, and each world must be distinguishable at original size with the character overlay.
4. Check cosmetics and all six user decoration slots; confirm new scenery doesn't obscure hats, tail, hands or selected props.
5. Assess runtime responsiveness, CPU impact, reduced motion, fullscreen/tray suppression and screenshot privacy behavior on a **physical Windows device**. The GitHub Windows runner is a build/smoke-certification environment, not this physical signoff.
6. Record actual screenshots and human rating against P2's Cozy Pixel Life rubric before approval for a signed public release. Source art and tests cannot certify subjective aesthetic quality.

## Later work

P8 owns selective ambient effects, restrained lighting, atmosphere and optional parallax; P9 native desktop UX; P10 Quick Panel, P11 main app, P12 Studio, P13 cosmetics. P7 does **not** add those systems or issue a public release.
