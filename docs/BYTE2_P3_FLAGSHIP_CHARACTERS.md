# Byte 2.0 — P3: Byte and Mochi Production Sprite Redesign

**Branch:** `feature/p3-byte-mochi-sprite-redesign-20261008`  
**Design source of truth:** [P2 Cozy Pixel Life](BYTE2_P2_VISUAL_IDENTITY.md)  
**Baseline:** [P1 Source and Sprite Audit](BYTE2_P1_VISUAL_UX_AUDIT.md)

## Delivered

P3 replaces **only Byte and Mochi** source PNGs, default semantic recolor colors, and per-frame accessory anchors. Pip and Kiwi remain untouched for P4. Existing character IDs, user preferences, cosmetic IDs, saved palette IDs, 26 named frames, 29 semantic behavior mappings, state machine, GPU/canvas renderer, and local monitoring remain unchanged.

| Character | P1 distinct art / 26 | P3 distinct art / 26 | New default appearance |
| --- | ---: | ---: | --- |
| Byte | 21 | 26 | A softer, compact round-visor robot with raised antennae, clearer eyes, cheek accents and visible arm/foot movements |
| Mochi | 20 | 26 | A cream marshmallow kitten with substantial pointed ears, bright eyes, blushing cheeks, expressive tail and independently moving paws |

### Production artifacts

- `public/assets/characters/byte/atlas.png` — lossless indexed PNG, 512×256, 26 frames in existing 8×4 grid, 64×64 cells.
- `public/assets/characters/byte/preview.png` — 128×128 nearest-neighbor thumbnail from actual P3 idle frame.
- `public/assets/characters/mochi/atlas.png` — same production contract, original 26-pose artwork.
- `public/assets/characters/mochi/preview.png` — actual Mochi P3 idle artwork, not a concept rendering.
- Both `manifest.json` files — corrected exact source color slots, default palettes, and pose-dependent semantic anchors.

Sprites use **seven unique, semantically mapped source colors plus full transparency** (outline, body shadow, body primary, highlight, accent, eye, blush). This matters because Byte's `recolorAtlas` replaces colors by their source RGB values; two identical source colors cannot maintain separate palette semantics. The P3 eye/outline colors therefore must remain distinct in both characters.

### Artistic decisions

Byte stays recognizably a small blue robot, but gets an enlarged dark visor, bright pupil shapes, rounded body edges, amber antennae and more legible expression changes. It is curious and gentle rather than mechanically stiff. Distinct happy, startled, angry, sleepy, charging, concern, typing and rare-idle poses are authored pixel by pixel.

Mochi is an upright cream kitten with visibly separate ears, blush, two front paws and a curling tail. Different breathing/idle, blink, look, loaf/sleep, left/right paw typing, paw raise, happy, surprised, stressed, warming, and two rare idle poses are real separate bitmaps.

All source raster coordinates are whole integers at **64×64**. No image smoothing, in-between interpolation, additional rendering engine, network image fetching or per-sprite animation loop was introduced.

### P3 automated quality gates

New `src/companion/animation/p3-character-art.test.ts` tests:

1. Decode the *actual* transparent indexed 4-bit PNG with PNG filters and 64×64 source tile locations; count bitmap-distinct poses.
2. Assert **at least 24/26** distinct bitmaps. Current output is **26/26 for both**.
3. Assert the common problem pairs that P1 found identical are now visibly different: idle A/B, sleep A/B, busy A/B, happy/surprised, click/idle.
4. Assert all **seven semantic source colors** are unique, each appears in the atlas, and the chosen default palette matches them exactly.
5. Decode the preview and assert nearest-neighbor 128×128 output is precisely a 2× enlargement of the P3 idle frame, down to every pixel.
6. Check all semantic accessory anchors remain on-canvas, that paw/arm anchors respond to typing, and happy lifts the arm positions.

The existing manifest and renderer tests remain applicable, and normal `npm run test` / `npm run check` / Windows packaging must remain green.

## Animation runtime compatibility

The existing `CharacterCanvasRenderer` loads image atlases from the **same URLs**, draws the same `frame.index` values, and recolors them with the manifest's semantic source-to-palette map. No new renderer adaptation is necessary. The shared 12 FPS scheduler, `prefers-reduced-motion`, semantic priority and normal activity sampling have not changed.

All seven accessory anchor names remain: `head`, `face`, `body`, `back`, `left_hand`, `right_hand`, `ground`. Anchor locations now track the new head/face/paw/body positions and are adjusted for locomotion, typing, excited movement and sleep.

## Review procedure

1. Open the new atlas PNGs directly from the PR with pixelated/nearest-neighbor rendering, zoom at 1×, 2× and 4× without smoothing.
2. Review the P1 Visual Evidence workflow's contact sheets; inspect Mini, Perch and Habitat render sizes (24/32/48/96 pixels).
3. In the Studio, select Byte and Mochi in all saved palettes and swap cosmetics. Verify headwear, glasses, body and hand props in idle, sleep, typing, happy, surprised and rare states, including left/right and raised paws.
4. Observe both in the **actual packaged Windows app** for 5+ minutes idle, interact, type, pet, adjust display mode, hide and reopen, toggle reduced motion, use mouse click-through and change character/palette. Capture screenshots and short videos.
5. Use the P2 review rubric for silhouette, cuteness, expressions, outfit fit and actual movement at native size. Automated success does not certify aesthetic appeal or physical Windows behavior.

## Non-goals / remaining work

- P3 does not redesign Pip/Kiwi, habitats, application navigation, Quick Panel or Studio. Those remain P4/P7/P10–P13.
- This phase does not change PC telemetry, introduce AI, accounts or remote assets.
- Contact-sheet imagery is **source art evidence**, not a substitute for live Windows proof.
- User-level **subjective approval of the new character style**, accessory overlay screenshots, packaged Windows observation and release signoff remain manual gates; no new public release or release tag.
- Sprite pixel editing source is the shipped indexed PNG atlas itself. Use lossless indexed PNG tools (e.g. Aseprite), retain exact seven palette-slot RGB values and frame cells, and regenerate the matching preview with nearest-neighbor scaling. The new tests protect this contract.
