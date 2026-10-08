# Byte 2.0 — P2 Visual Identity: Cozy Pixel Life

**Status:** design direction locked for P3–P13, **not** production-art approval.  
**Source of truth:** `design/byte2/identity.json` (machine-readable), this document (art and UX interpretation), P1 audit (baseline evidence).  
**Previous architecture:** the Byte v1.0 documents remain authoritative for runtime implementation until later phases explicitly migrate them.  
**Commit scope:** P2 is intentionally non-disruptive. It does not overwrite any shipped character atlas, habitat manifest, user preference or Windows surface.

## Identity in one sentence

**Byte is a tiny living pixel friend who keeps you company while quietly caring for your computer.**

Four characters, six distinct worlds, expressive but restrained animation, warm authored pixel artwork, and a useful modern native-desktop shell. Byte should be *immediately lovable at 48 px*, visually distinct from a generic system monitor, and neither demanding nor distracting.

### Product hierarchy

1. **Friend first:** The character is the default reason to open the app. Its pose, expression and home communicate a state without technical text.
2. **Place second:** A small inhabited environment makes it feel like a creature with a home rather than an icon floating over a dashboard.
3. **Useful only when useful:** Show status in ordinary language, then deeper PC-health information on demand or when actionable.
4. **Respect the desktop:** Quiet animations, usable click-through/hiding/modes, truthful device status, zero mandatory accounts.
5. **Own the look:** Character silhouette and pixel-art texture are the core brand. Avoid a generalized AI/SaaS panel aesthetic.

## Visual family: Cozy Pixel Life

- **Pixel-perfect creatures**, authored on a grid. The oversized character face and active silhouette must still read when shrunk to Mini mode.
- **Soft shapes, definite pixels:** round cheeks and bodies built with stepped pixel outlines, *not* blurred edges, high-res airbrush, vector strokes, or generic emoji expressions.
- **Warm-clay, twilight-blue foundations:** pastel tones are an accent, not a rainbow wash. Warm upper-left light, cooler shadow side, and grounded feet.
- **Selective shadows:** one readable shadow mass, a small highlight, and small dark details. Avoid high-frequency noise or gradients in the sprite.
- **Four immediately distinct silhouettes:** robot with visor/antennae, kitten ears/paws/tail, asymmetric wobbling jelly, pear-shaped bird/beak/wings.
- **Charming in stillness:** a neutral pose must be attractive before motion, particles, hearts, cosmetic purchases or interface embellishments.
- **Cozy pixel environments:** six lived-in settings with controlled parallax-like planes, grounded lighting and few quiet details; no overloaded 3D dioramas.

### Prior art, not to copy

The P1 contact sheets are verified representations of current Byte artwork. They establish limitations and compatibility, **not** the final aesthetic ceiling. P2 concept imagery is exploratory, not source-ready. No third-party mascot, game character or copyrighted composition is to be traced into production assets.

## Existing technical contract — preserve

| Layer | Fixed P2 rule | What later phases may change |
| --- | --- | --- |
| Character native footprint | ~48×48 within 64×64 animation canvas | Shape details and art source resolution may evolve without breaking 64×64 output |
| Atlas | 64×64 cells, current 8×4 sheet | Add cells/atlases with migration and perf evidence in P3+, never silently break manifest |
| Rendering | Integer sprite rectangles, nearest-neighbor/pixelated, no smoothing | Cache implementation details after benchmarks |
| Colors | Seven semantic recolor slots, 9 named palettes per current character | P3 may refine actual default slot RGB without changing ID, mapping, other user palettes |
| Animation | Shared semantic runtime, 12 FPS scheduler, priority and reduced-motion mode | Reauthor frame poses, clips, behavior mappings, personality sequences |
| Cosmetics | Seven per-frame named anchors | Add authored offsets/anchor tooling but preserve old saved cosmetics |
| Habitats | 256×256 coordinate space, separate BACK/FRONT layers, time palettes, decoration slots | P7 may add image-backed layers with backwards-compatible extension |
| Data | Local-only diagnostics/preferences | No new backend, AI API, subscriptions or cloud requirement |
| Release | CI, packaged tests and physical Windows approval | No shortcuts because visual work seems low-risk |

Do not make the frontend execute a new animation or particle timer loop. Keep idle impact measurable in a packaged Windows build.

## Mascot production art briefs

### Byte — small soft robot

**Silhouette:** squat rounded pebble robot, enlarged navy visor, two expressive amber antenna tips, short feet, little segmented arms; recognizable as Byte, not a generic box.

**Emotion:** digital eyes with explicit visual vocabulary: idle curved pupils, slow half-blink, worried angled pupils, wide surprised orbits, smiling squint. A friendly face takes precedence over status icons.

**Motion personality:** slightly anticipatory and mechanical; the antennae move first, then visor/torso, then short legs settle. No constant sparks. Small helpful reassurance on a normal PC, earnest attention for a sustained issue.

**Mandatory P3 benchmarks:** calm idle, clearly different idle breath, real blink, typing left/right, happy, surprised, sleep enter/hold, irritated, charging, concern. Keep all semantic warning behaviors distinguishable where appropriate.

**Avoid:** straight static rectangle, tiny one-pixel "smiles" nobody can read, flashing lights every few seconds.

### Mochi — affectionate kitten

**Silhouette:** compact marshmallow kitten, peach ears, cheek tufts, huge front paws, short rounded torso and curling tail. The ears/tail make the outline instantly feline in Mini.

**Emotion:** large readable dark pupils; pink cheek color separated from the eye; sleepy curved closed eyelids, shy pleased smile, recognizable worried expression.

**Motion personality:** alternating paw taps, tail curling before reactions, slower loaf/stretch cycles, one tiny ear twitch. Petting changes both eyes and head/ear pose.

**Mandatory P3 benchmarks:** open-eyed idle, subtle distinct breath with tail motion, blink, left/right paw typing, paws-up joy, startled ears, sleepy loaf, groom rare idle, curious head tilt.

**Avoid:** identical click/surprised/idle frame, generic cat head with no paws, constant heart particles.

### Pip — lively squishy blob

**Silhouette:** small asymmetric jelly-drop body, subtly higher top point, droopy cheek edge and tiny soft side fins/arms. Distinct from an ordinary blue circle.

**Emotion:** expressive high-contrast eyes, flexible "w" smile and wide-mouth surprise that remain legible at 24 px.

**Motion personality:** physical squash → stretch → jelly overshoot → settle, all authored intentionally; sleep forms a visibly wider puddle, not a recolored idle.

**Mandatory P4 benchmarks:** calm asymmetric idle, offset wobble idle, blink, two distinct effort poses, deep sleep, broad joyful stretch, wide-eyed surprise, shy recoil, split-off rare idle.

**Avoid:** transparent Gaussian blur, same circle with barely changed eye pixels, excessive bouncing while user works.

### Kiwi — energetic little bird

**Silhouette:** rounded pear body, oversized face, definite small beak, visible separate winglets, tiny feet, possibly an identifying leaf-feather tuft.

**Emotion:** head tilt/pupil direction strongly changes expressions; beak actually opens/closes for pecking/typing.

**Motion personality:** quick paired hops, asymmetric independent wing flaps and awkward planted landings. Curious rather than hyperactive all the time.

**Mandatory P4 benchmarks:** attentive front idle, real head-tilt idle, blink, two peck states, surprised open beak, wings-up delight, tired drooping wings, sleep, seed rare idle.

**Avoid:** repeated neutral sprite for wake, click, surprise and stress; invisible feet; both wings moving identically in every pose.

## Pixel consistency and measurable P3/P4 acceptance

- The P1 baseline (bitmap-unique poses of 26): Byte 21, Mochi 20, Pip 21, Kiwi 18. This is why the new minimum is **90% visually distinct indexed art at each character's baseline atlas**, measured as bitmap equality, **not** a promise that each semantic state needs an entirely new atlas cell.
- For 26 indexed frames, **at least 24 must have unique raster content**. More frames are not inherently better; frames with visible expression or silhouette change are better.
- Hard compare these pairs: `idle_a != idle_b`, `sleep_a != sleep_b`, `busy_a != busy_b`, `happy != surprised`, `click != idle_a`. Some low-priority semantic mappings may reuse a clip if that is explicitly justified.
- Test characters at **24, 32, 48 and 96 rendered pixels**, on cream and graphite surfaces, with and without one cosmetic. 96 px is for diagnosing individual pixels, never the only review scale.
- At least **3 different emotional eye sets and 3 clearly different mouth/brow configurations** per mascot; asymmetric side poses must remain plausible and grounded.
- Verify the seven cosmetic anchors on *every* frame; check no hats float, paws disappear or glasses cover expressions. Keep metadata and preview thumbnail in sync.
- Poses must remain intelligible when playback is off or reduced motion is on. No animation state only communicated by speed or color.
- Eye direction and pet response should be measured with a short captured video; the contact sheets alone do not certify motion.
- Use the existing P1 checker for integrity, then extend with future-art uniqueness tests in P3/P4. Do **not** fail production CI on old sprite diversity before those phases actually ship.

## Habitat art direction

The P1 runtime currently uses geometric shapes only. P7 should add **original layered pixel artwork** without making scene updates heavy.

| Habitat | Emotional story | Distinctive focal details | Restrained ambience |
| --- | --- | --- | --- |
| Meadow | Relaxing open air | Sunlit flowers, a soft grass patch and a tree canopy | A few petals/breezes |
| Cozy Desk | Tiny home workstation | Warm lamp, stack of books, mug, plant | Delicate screen/light flicker |
| Bedroom | Safe sleepy space | Bed blanket, low shelf and square window | Gentle curtain movement |
| Space | Cozy stargazing nook | Cabin porthole, planet and constellation | Sparse drifting stars |
| Aquarium | Peaceful underwater room | Colored coral, sea plants and scattered shells | A few bubbles/light rays |
| Rooftop | Quiet evening retreat | Railings, plant pots and layered city skyline | One or two sunset glints |

**Scene rules:** 256×256 final logical dimensions; character always has a readable resting surface and contact shadow. Allocate visible positive/negative space so the character reads. Color-variation (morning, day, evening, night) should change atmosphere, not flatten the character. No particle effect may visually override diagnostic severity or overwhelm the sprite. Sprite and environment pixel sizes must be consistent.

## UI: the app, Quick Panel and Studio must belong to the same world

The UI is **not a game inventory stretched over a Windows window**, and it is not a generic analytics dashboard. It uses compact, reliable Windows controls around a lively pixel-art stage.

### Main window / Home (P11)

- On open, the **live companion scene** is the visual center, occupying a meaningful area without a giant text header.
- At most a short, one-line character status (e.g. "Mochi is resting") and a calm PC status. No permanent grid of CPU, RAM, network, battery, temperature and gauges.
- Expose Health/Activity/Studio/Settings as **real navigation destinations**, not popups pretending to be pages. The Health page owns charts and detailed diagnostics.
- Underneath the world, show **real interaction affordances**: pet, wave/visit, move/hide, and customize (exact set to be validated at P11).
- Avoid oversized typography, floating cards inside cards, fake hologram glass, gradients, generic AI feature pills, or icon-only action ambiguity.

### Quick Panel (P10)

- Small recognizable mascot or live cropped sprite + readable health status.
- If action is needed, give one strong explanation and action, with supporting confidence/context available.
- Move/hide/click-through remain reachable and explicit. The popover should *not* become a full mini dashboard with six metric cards.
- Fit in actual monitor work area, keep focus management and keyboard escape behavior, and follow high-contrast and reduced-motion preferences.

### Studio (P12)

- Stable living stage with character and habitat in view while outfit choices change.
- Visible character strip; style tabs can be real in-page views, with no modal maze.
- Choices must be pictured, not described solely by names and colored circles.
- Fast preset application, accessible keyboard choice, instant accurate preview, undo/redo and persisted selections.
- Hiding Studio should not unhide a deliberately hidden companion.

### Onboarding (P11)

First step **shows** Byte, not just a large mission-statement sentence. Selecting character and habitat previews their actual production art, *not the generated concept sheet*. Privacy and monitoring opt-ins must remain explicit and truthful.

## UI palette and type

The canonical colors are machine-readable in `design/byte2/identity.json`. Light foundation is warm mineral/cream with graphite text; dark foundation is graphite/ink with gentle cream text. Character tones—blue, cream-peach, lavender-blue, kiwi-green—belong to the sprite and scenery. **Warning/error colors remain semantic**; never recolor danger to pink just for cuteness.

The validator checks AA **4.5:1** text contrast for normal/secondary/accent text on defined surfaces. This is not a full accessibility audit. User high-contrast mode, Windows forced-colors, focus indicators and 100–200% text scaling remain required.

- System font: Segoe UI Variable (fall back to system sans); avoid decorative web-font downloads.
- Human-readable core text no smaller than 12 px, favor 13–14 px for normal body copy.
- Concise page labels, headings generally 16–24 px; no enormous landing headline.
- Radius scale 7/11/13 px, thin controlled borders and shadow. **No generic rounded SaaS cards or oversize pill parade.**
- Pixel scene retains straight source pixels even when containing shell has curved corners.
- Controls and informational readouts should use text plus shape/icon, never color only.

## Art production workflow and approval gates

1. **P2** — lock style grammar, concepts, color tokens, contrast, persona briefs and artifact/test contracts (this phase). **No production atlas edits.**
2. **P3** — create 3 distinct original silhouette/face study variations each for Byte and Mochi, compare at 24/48 px, choose one family. Author baseline clips and anchor maps; review real contact sheets and real app preview before merging. Aim for >=24/26 visibly distinct poses, no common duplicate pairs.
3. **P4** — Pip/Kiwi at equal quality; generate same before/after proof, unique pose and personality checks.
4. **P5** — motion polish using current 12 FPS scheduler and 125ms delta clamp; confirm no fast-forward after hide/sleep and perf regression.
5. **P7** — original hand-authored habitat background planes; migrate safely and ensure four time palettes.
6. **P10–P13** — implement UI/Studio/Quick Panel and cosmetics from the locked art grammar after verifying actual production screenshots.
7. **P17–P18** — long-session performance, focus/accessibility, mixed DPI, installer, physical Windows certification and release.

### Visual approval rubric

Every production character has an **art gate** (not merely passing JSON validation): human review of cuteness, silhouette, emotional clarity, palette readability, motion personality and cosmetic fit at actual size. Target >=4/5 per category with no subcategory below 3.5/5; if scored on an integer scale, no value below 4. Evidence must include source PNG, contact sheet, real composed preview and an interaction video. Subjective appeal cannot be machine-certified.

The P2 generated concept board is useful inspiration for shapes, faces and color balance, but **it is not a legitimate 64×64 pixel atlas**, has antialiased illustrative shading, and can contain decorative UI details that should not be copied literally. Each production frame must be redrawn and QA'd as true pixel art.

## How P2 is validated

```bash
npm run design:verify
npm run design:test
npm run visual:audit
```

CI checks that the theme passes contrast requirements, character/habitat IDs and semantic anchors are unchanged, production frame **names** exist, and all critical reduced-motion/performance/release rules remain enabled.

**Passing these checks does not mean P3/P4 sprites exist, that UI has been redesigned, or that the app was visually tested on a physical Windows desktop.**
