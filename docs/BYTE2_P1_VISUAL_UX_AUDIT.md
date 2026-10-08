# Byte 2.0 — P1 Visual & UX Audit

**Date:** 2026-10-08  
**Base revision:** P0 `2111f74086ef5f14782f2337714b9e11d4372614`  
**Phase:** P1 (design baseline and evidence tooling, not a redesign)

## Evidence boundary

This audit distinguishes **source-verifiable structure** from **observed visual quality**. The source review covers actual manifests, UI markup/styles, animation policy and rendering paths. No actual interactive Windows UI screenshot or animation footage was accessible to the auditor during this phase. Therefore **cuteness, artistic polish, accessibility compliance, motion feel and measured performance are NOT scored**. Any visual rating prior to inspection would be fabricated.

The GitHub P1 Visual Evidence workflow produces actual frame contact sheets from source sprite atlases. These are inspection assets, **not** desktop screenshots. The manually operated `scripts/capture-p1-screenshots.ps1` captures real desktop screens when run on a physical Windows machine. Do not treat generated references, simulated screen layouts, old marketing images or manifest presence as proof of visual quality.

## 1. Source-inventory baseline

| Surface | Verified inventory | Limitation |
| --- | --- | --- |
| Characters | Byte / Mochi / Pip / Kiwi, each 26 distinct indexed sprite frames, 26 clips, 29 semantic behaviors, 9 palettes; native art 48 px on a 64×64 animation frame | The existence of frames does not prove visual quality, cuteness or interesting motion |
| Atlas / previews | Four 512×256 atlas PNGs and four preview PNGs | Perceptual review requires actual generated contact sheets or opened binary assets |
| Animation | Shared 12 FPS scheduler, lifecycle suspension, semantic priority, reduced-motion fallbacks | Motion timing cannot be judged from a clip manifest alone |
| Habitats | Meadow 81 primitives / 18 layers; Desk 78 / 17; Bedroom 65 / 18; Space 71 / 17; Aquarium 62 / 16; Rooftop 76 / 18; four time-of-day palettes each | Every background is drawn from geometric primitives; no image-backed primitive layers |
| Cosmetics | SVG cosmetics attached using per-frame anchors | Need screenshot verification across four bodies, all display modes, and extreme poses |
| Full app | Five primary destinations: Overview, Activity, Apps, Customize, Settings | Screen appearance and actual navigation ergonomics remain unobserved |
| Studio | Seven sections, four bundled looks, production canvas preview, queued persistence, undo/redo | Actual preview fidelity, clipping, card usability and rapid-choice feel unverified |
| Onboarding | Four stages with character thumbnails and simple habitat-color selectors | Does not provide a visibly immersive first character interaction in source UI |
| Quick Panel | Compact status / issue / resource metrics / controls composition | Actual density and reachability need screenshot and keyboard tests |

**Repeatable checks:** `node scripts/audit-visual-assets.mjs` and the CI artifact `byte-p1-source-art-<SHA>`.

## 2. Code-grounded design findings

| ID | Severity / priority | Source evidence | Finding | Action / phase |
| --- | --- | --- | --- | --- |
| V01 | High / P3–P4 | `public/assets/characters/*/manifest.json` | Four complete but deliberately compact sprite vocabularies; most animation clips alternate only two frames, and only three named clips have three steps. Cuteness and silhouette quality unknown until contact sheets reviewed. | Produce character concept revisions, visual review at 1×, 2×, Mini; approve a distinct pose/style board per mascot |
| V02 | High / P5 | `src/companion/animation/scheduler.ts` | Shared 12 FPS pacing is deliberately lightweight. More detailed animation cannot simply raise the global frame budget without real performance data. | Improve pose quality, state anticipation/settling, authored personality-specific loops, and benchmark before cadence changes |
| V03 | High / P7 | `public/assets/habitats/*/manifest.json` and `HabitatCanvasRenderer.ts` | Six scenes have curated colors and depth planes but use 62–81 simple primitives each. No independent detailed scene illustration assets exist. This constrains the next art-production workflow. | Replace flat-looking geometry selectively with original layered pixel scenery while preserving manifest positioning and local palettes |
| V04 | High / P10 | `src/features/quick-panel/QuickPanel.svelte` | The panel opens with a generic "B" icon; status/issue text and system metrics consume the main hierarchy before companion controls. It does not visibly place the character at the heart of the interaction. | Lead with a tiny live mascot/status and one primary action; collapse secondary metrics; keep essential Move/Click-through reachable |
| V05 | High / P11 | `src/app/MainWindow.svelte` | Default Overview is dominated by a diagnostic status hero and resource metric cards. Companion is represented by a smaller preview and sidebar control. This conflicts with the stated "cute first" product priority. | Make the live character/habitat the primary Home experience, with PC assistance progressively disclosed. Avoid a generic SaaS metric-card layout |
| V06 | Medium / P11 | `src/app/MainWindow.svelte` | Five sections are clear but UI implementation is concentrated in ~999 lines, mixing view state, diagnostics, persistence and visual markup. | Break out per-route views and reusable status/presentation models before major redesign |
| V07 | High / P12 | `src/app/customize/CustomizationStudio.svelte` | Studio has a functioning scene preview, presets and controls but is ~1,757 lines and has seven categories. Source cannot demonstrate fast browsing or pleasant outfit editing. | Reorganize as immersive dressing room; measure tasks and avoid long settings-form flows; split components |
| V08 | Medium / P13 | `public/assets/cosmetics/*.svg` + anchors | Existing SVG cosmetics are small, lightweight, and reusable, but whether they visually fit pixel characters across poses is unverified. | Verify cosmetics in-frame, in front/back layers, for all characters; redraw misaligned or inconsistent items |
| V09 | High / P11 | `src/app/settings/Onboarding.svelte` | First onboarding screen is copy/cards without live character. Habitat selection uses colored chips, not actual world images. | Introduce Byte visually in step one and a real animated character+habitat preview during choice, retaining privacy settings |
| V10 | Medium / P10–P12 | UI markup/styles | Some explanatory text is authored around 9–11 CSS px, with constrained popover and seven studio sections. This is a **readability risk**, not a proven accessibility failure. | Inspect at 100%, 125%, 150%, 200% scaling, 1366×768 and high contrast; protect focus, keyboard and reduced-motion support |
| V11 | Retain / P3–P18 | `docs/PRODUCT_CONTRACT.md`, `docs/PERFORMANCE.md` | Local-only privacy, no accounts, no telemetry upload, semantic alerts and single-source system sampling are distinctive product strengths. | Preserve strictly throughout visual changes |
| V12 | Retain / P9–P18 | P0 and current native window integration | Multiple display modes, fullscreen/lock suppression, click-through and multi-monitor code exist with green automated CI; actual desktop polish remains unverified. | Preserve and physically qualify; avoid feature-driven regression |

## 3. Retain / redesign / replace decisions

### Retain in architecture
- Tauri 2 + Svelte + Rust with one hardware sampling pipeline.
- Declarative character manifests, semantic behavior mapper, atlas-loader/caching, cosmetic anchors.
- Shared frame pacer and explicit reduced-motion/lifecycle suspension.
- Habitat manifest layer ordering, palette times of day, decoration slots.
- Privacy-first system explanations, on-demand apps diagnostics and opt-in local history.
- Current Windows modes, local persistence/rollback, and release certification gates.

### Redesign at the product level
- Byte/Mochi/Pip/Kiwi facial expressions, proportions, neutral pose appeal, motion signatures and character individuality **after inspecting contact sheets**.
- Habitat composition, scene depth, native pixel styling and foreground details.
- Default screen from system dashboard to character-first interactive Home.
- Quick Panel hierarchy; Studio category navigation and immersion; first-run introduction.

### Replace only if verified by visual evidence
- Specific atlas frames / accessories that appear jagged, repetitive, detached or incoherent.
- Specific scenes that remain flat after direct inspection.
- Any interaction which proves inaccessible, clipped or distracting on real displays.

Do not globally replace sprites or UI structures based solely on proxy inventory measurements.

## 4. Visual QA acceptance matrix

### Screen path and user-flow capture
Capture real Windows 11 screens with `scripts/capture-p1-screenshots.ps1` in this order:

1. Onboarding: actual first-run screen and companion choice.
2. Four characters: idle, one direct interaction and one typing response **per character** (stills + short recordings).
3. Six habitats: same mascot, consistent scale, day and night where possible.
4. Display modes: Habitat, Perch, Mini, Edge, Tray; show relation to desktop/taskbar.
5. Quick Panel: healthy, issue, keyboard focus / scaling where possible.
6. Main app: Overview, Activity, Apps (scan and empty state), Customize, Settings.
7. Studio: outfit changes, decorations, presets, live preview and rapid changes.
8. Accessibility: reduced motion, high contrast and Windows text scaling.
9. Windows handling: secondary display, mixed DPI, hot-unplug, fullscreen and click-through.

The capture script contains 28 named still-image steps; it intentionally does not fabricate warnings or programmatically manufacture activity events. For timing, motion, user perception, and actual usability, record a short video with the built-in Windows capture tool and manually annotate the interaction.

### Human scoring rubric (1–5, record with each screenshot/video)

| Dimension | 1 | 3 | 5 |
| --- | --- | --- | --- |
| Character cuteness | Unappealing/unclear silhouette | Pleasant but generic | Instantly lovable, distinctive at actual size |
| Expression/readability | Emotion unintelligible | Understandable at normal scale | Clear at Mini, Perch and Habitat sizes |
| Movement/behavior | Repetitive/snapping | Functionally legible | Expressive, cohesive, purposeful and unobtrusive |
| Pixel-art consistency | Conflicting visual styles | Mostly consistent | Unified outline, value, lighting, scale |
| Habitat art/depth | Flat, empty or noisy | Acceptable composition | Rich but restrained, coherent depth and staging |
| Cosmetics fit | Floats/clips | Mostly fits | Anchored and polished across extremes |
| UI visual identity | Generic monitoring dashboard | Recognizable Byte styling | Companion-first, original, cohesive |
| UX ease and clarity | Frequent confusion | Workflow can be completed | Main action evident, low-friction navigation |
| Accessibility | Important controls unreachable | Some scaling issues | Screen reader/keyboard/scaling QA passes |
| Desktop comfort | Intrusive, hard to hide | Usually convenient | Persistent but respectful, easy to adjust |

Use **N/A** when evidence is unavailable, not an estimated score. Review at 100% native UI scale and actual Mini size, not only enlarged exports.

## 5. P2–P13 handoff and priority

1. **P2 design lock:** Evaluate actual contact sheets with 1× previews and real screenshots. Select silhouette, eye/cheek, proportions, outline and palette rules. Require full 4-character concept board.
2. **P3 Byte + Mochi:** First original sprite redesign against benchmark poses (idle, blink, typing, happy, sleep). Keep renderer contract.
3. **P4 Pip + Kiwi:** Build on approved P3 approach, different silhouettes/motion.
4. **P5 motion:** Increase expressive quality, not raw animation quantity; benchmark CPU/GPU.
5. **P7 scenes:** Move from purely primitive scenery toward original layered pixel artwork.
6. **P10 panel:** Compact mascot-first interaction; clear native controls and diagnostics.
7. **P11 app:** Companion-first Home and reworked overview.
8. **P12 studio:** Preview-centered modular dressing room, performance and undo correctness.
9. **P13 cosmetics:** Character-specific visual fit and clipping QA.

## 6. P1 exit statement

- **Complete:** source structure audit, concrete priority findings, source-contact-sheet automation, screenshot-capture harness, scoring matrix and forward design constraints.
- **Requires real visual evidence:** user-facing aesthetic scores, observed screenshots, animation videos, final redesign-versus-retain decisions for individual frames and UI details.
- **No misleading claim:** P1 has not visually certified the application on a Windows desktop.
- **No public installer:** this phase only adds design-audit tooling and documentation.

## Commands

```powershell
node scripts/audit-visual-assets.mjs
pwsh -NoProfile -File scripts/build-p1-contact-sheets.ps1
pwsh -NoProfile -File scripts/capture-p1-screenshots.ps1 -List
pwsh -NoProfile -File scripts/capture-p1-screenshots.ps1 -Step all -MonitorIndex 0
```

The first two run without opening a real desktop app; the screenshot step is manual, local and opt-in. **Review screenshot files for private information before sharing**, and restore capture-exclusion settings afterward.
