# Byte 2.0 — P4: Pip & Kiwi Character Redesign

**Phase:** P4 — second character production art pass  
**Art direction:** [P2 — Cozy Pixel Life](BYTE2_P2_VISUAL_IDENTITY.md)  
**Earlier implementation:** [P3 — Byte & Mochi](BYTE2_P3_FLAGSHIP_CHARACTERS.md)

## Scope and result

P4 **replaces the real production pixel art** for Pip and Kiwi. The new assets use exactly the existing public paths, 512×256 four-bit indexed PNG atlas format, 64×64 pixel grid cells, 26 pose names, 26 clips, 29 semantic behaviors, and nine named per-user palettes. There is no new UI, heavy animation framework, particle engine, backend, telemetry or native window behavior.

| Mascot | P1 distinct raster poses | P4 new distinct poses | Distinctive P4 art changes |
| --- | ---: | ---: | --- |
| Pip | 21/26 | **26/26** | Asymmetric droplet/jelly silhouette, larger eyes, tiny fins, true squash-to-puddle sleep, elongation/hop reactions |
| Kiwi | 18/26 | **26/26** | Pear-shaped green bird, expressive pupils, raised asymmetric wings, tufted head, distinct peck-up/peck-down typing beak, tiny feet |

**All four characters now have 26/26 distinct production bitmap poses**, including Byte and Mochi redesigned in P3. Independent full-app/live animation and cosmetic visual approval is *still required* on a real Windows machine.

## Files delivered

- `public/assets/characters/pip/atlas.png` and `preview.png`
- `public/assets/characters/kiwi/atlas.png` and `preview.png`
- Both corresponding `manifest.json` files: exactly seven recolorable semantic source slots and new frame-specific cosmetic anchors.
- `src/companion/animation/p3-character-art.test.ts`: extended existing decoded-PNG tests from two to all four production characters, plus tests of Pip's flattened sleep geometry and Kiwi's wider raised-wing silhouette.
- `scripts/p3-character-review.html`: original source art local preview tool supports all four mascots at 24/32/48/96 px and every saved palette.
- This document and the updated production-character status in `docs/CHARACTERS.md`.

The atlas is not a decorative concept board: its frame cells are used in the production `CharacterCanvasRenderer` via the existing URLs, with the same frame indices. The preview is a nearest-neighbor 2× enlarged, exact bitmap reproduction of the atlas `idle_a` frame.

## Character-specific decisions

### Pip — gentle asymmetric jelly

- **Signature outline:** upward peak, subtle droop to one side, and two short side fins. This is distinctly different from a circle and stays recognizable at 24 px.
- **Everyday:** resting body with big bright eyes, visible cheeks; the second idle displaces its wobble/fins, while glance poses change eye location.
- **Sleep:** body visibly collapses into a low, broad puddle. The automated test requires the full visible bitmap height to shrink by **at least 15 px** relative to idle.
- **Emotion:** joyful stretch and smile; true wide-eyed surprise; annoyed eyebrows; flattened low-battery, stressed, warming, charging and rare interaction states.
- **Movement:** tiny arms/fins move separately during left/right typing and reactions. Motion remains authored frames in the existing 12 FPS scheduler, not physics or procedural animation.

### Kiwi — curious, clumsy pocket bird

- **Signature outline:** pear-shaped small bird with a top feather tuft, short feet, distinct beak and separately moving wings.
- **Everyday:** clear eye shapes and directional eye glances; a second idle changes wing posture.
- **Motion:** two hop/step poses, wings-up celebration, raised wings on surprise, dedicated left/right peck-up/peck-down typing poses.
- **Sleep:** wings fold into the side of the body, drooping face and softened contour.
- **Expression:** open surprise beak, happy closed eyes, stressed/worried brows, annoyed and tired variations.
- **Silhouette test:** the raised-wing `happy` frame must be at least **3 px wider** than `idle_a`.

## Palette and accessory compatibility

The renderer maps palette colors by **exact RGB keys**. Therefore P4 uses seven **distinct** source colors and an alpha-transparent background. The semantic slot names stay unchanged: `dark`, `shadow`, `primary`, `highlight`, `accent`, `eye`, `blush`. The `default` saved palette is updated to match the new source colors. Other user-selectable palette IDs and user preferences remain intact.

Every frame retains all seven original cosmetic anchors: `head`, `face`, `body`, `back`, `left_hand`, `right_hand`, `ground`. P4 moves relevant anchors for jelly stretch/sleep, raised bird wings, typing, walking and excited animations while keeping them within the 64×64 canvas.

### Known follow-up risk

Exact on-screen cosmetic placement with *all* accessory SVGs cannot be certified from the manifest alone. Source tests check contract and coordinates, but **real Windows screenshots** are required to approve hat/face/body/back/hand prop fit in idle, happy, typing and sleep for each new mascot.

## Quality gate

The expanded `p3-character-art.test.ts` decodes the actual transparent, indexed PNG files (not merely filenames) and verifies for all four mascots:

1. 512×256, four-bit palette PNG and 128×128 lossless preview dimensions.
2. **At least 24/26 unique bitmap crops**; P4 achieved 26/26 in both new atlases.
3. Different idle A/B, sleep A/B, busy A/B, happy/surprised and click/idle frames.
4. Seven distinct actual palette RGB colors, each present in the atlas.
5. Every preview pixel exactly matches the corresponding idle frame enlarged by 2× using nearest-neighbor.
6. All seven cosmetic anchors stay within the canvas and typing/excited poses meaningfully move arm/wing anchors.
7. Two P4-specific sprite-outline assertions (sleep squashing, wing-raise width).

The P1 source-art evidence workflow independently generates unedited 26-frame contact sheets and `frame-diversity.json`.

## Review process and release policy

1. Compare actual committed sprites at **24/32/48/96 pixels** on light and dark backgrounds.
2. Use the local inspector `scripts/p3-character-review.html`; load the repository through a local HTTP server (e.g. `python -m http.server`) and switch character, pose, palette and background.
3. In the **packaged Windows app**, inspect Pip and Kiwi in Habitat, Mini and Perch, with all palette selections and representative accessories. Capture actual screenshots or video as evidence, including reduced motion and click-through scenarios.
4. Human reviewers should score overall cuteness, expression, silhouette, motion, cosmetic fit, and scene compositing at actual size; P2 targets at least 4/5.
5. Only approve a public installer after full CI, packaging and **physical Windows-device** acceptance. GitHub Actions runner certification does not replace hands-on assessment.

### Scope boundaries

- Byte and Mochi P3 artwork unchanged.
- All six habitats still use their existing P1-era scenery; illustrated updates are P7.
- App UI, onboarding, Studio and Quick Panel redesign remain P10–P13.
- No new public release tag, app-store publishing or Windows certificate signing is performed by P4.

## Local artifact authoring

Original source frames were drawn on integer coordinates into 64×64 transparent canvases and packed into four-bit indexed PNG sheets. The reviewed generator and contact sheets are included in the P4 art package alongside production PNGs. The actual committed atlas/preview pairs are the runtime source of truth. Avoid smoothing or changing the indexed color values during any future art edit; the pixel tests will detect incompatible changes.
