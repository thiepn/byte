# Byte 2.0 — P5 Character Animation & Personality

**Phase:** P5 | **Source:** P2 Cozy Pixel Life, P3/P4 real sprite atlases  
**Scope:** 4 existing manifests, shared animation-runtime bug fixes, species mannerisms, safety/quality tests. No new sprites, network calls, global input collection, animated CSS timers or Windows hooks.

## Production changes

P3/P4 gave each mascot 26 truly distinct bitmap poses, but before P5 **all four manifests reused almost identical timing curves**. P5 reauthors the clips to convey separate character temperaments, using the existing frame names, 12 FPS shared scheduler and one semantic state machine.

| Character | Animation personality | Typical resting and reaction timing |
| --- | --- | --- |
| Byte | Deliberate, curious little mechanical companion | Measured antenna/body "breath", compact typing taps, short anticipation before greeting |
| Mochi | Sleepy, affectionate kitten | Long soft idle holds, slow loaf/sleep, ear/paw movements before response, gentler settling |
| Pip | Bouncy asymmetric jelly | Fast wobble, double squash/stretch in joyful reactions, a visible jelly follow-through |
| Kiwi | Curious energetic bird | Short head/wing shifts, discrete hop–peck–recover, wing-up celebration and quick settle |

Clip timing uses stepped 90ms-or-longer source poses (visually meaningful at the existing approximately 83ms render cadence); **no higher FPS, tweening, animation library or extra work loop** is introduced. Every named clip has an explicit reduced-motion fallback frame. For one-shots, brief fallback completion still returns to the active base state.

### Concrete transition fixes

1. **Sleep → wake:** the old manifests mapped `sleep>wake` to the same `wake` clip as the target. This caused two sequential wake animations. P5 removes the redundant transition: the semantic `wake` clip already includes anticipation, awakening and settle.
2. **Attention warning entrance:** three prior transitions pointed at the looping `needs_attention` clip. A looping "entrance" never reaches the pending target. P5 adds a dedicated finite, two-step `attention_enter` clip and retains the persistent `needs_attention` loop as the final critical/diagnostic state.
3. **Validation:** a manifest with a looping transition now fails `validateCharacterManifest`; duration values must be finite and positive.
4. **Elapsed-time continuity:** `CharacterAnimator.advance` now carries remaining clamped delta into the pending target when a one-shot entrance finishes between scheduler ticks. This removes timing pauses without changing the 125ms delta clamp or adding a scheduler.

### Character reactions and real personality settings

The existing **CHILL / CURIOUS / ENERGETIC** and **QUIET / NORMAL / PLAYFUL** preferences remain the source of truth. They still set the frequency and intensity of incidental events, not diagnostics.

P5 introduces four low-priority character-specific mannerism profiles:

- **Byte:** curious visor glance, deliberate typing follow-up and small mechanical alert after a quick drag.
- **Mochi:** blink/soft eye reactions, sleepy side glances and gentle paws after typing.
- **Pip:** unexpected surprise and happy squishy recovery, longer springy post-drag.
- **Kiwi:** directional head glances, eager post-typing peck and curious wing movement.

The profiles live in `src/companion/personality/profiles.ts` and select **only behaviors already defined in each character manifest**. No motion hook, global cursor tracking, keystroke identity, analytics, accounts, or extra persistence was added. Extra idle signature weight is small and manifest-supported. A newly created director receives the actual character ID when preferences change, preventing stale character identity after a switch.

### Runtime priority and safety

All incidental reactions still use the same semantic priority ladder:

`critical > diagnostic > interaction > system > input > personality > idle`.

A critical warning entrance is interruptible **by higher-priority events**, and the ongoing warning loop remains explicitly non-interruptible by equal/lower-priority events. Recovery from diagnostic/critical to the now-safe base state remains forceful and responsive. Personality reactions may be rejected if the warning has priority.

The shared scheduler remains ~12 FPS with a **125ms maximum delta**, stops when hidden and resumes without fast-forward. No additional recurring timers or RAF loops have been introduced. Existing neutral/quiet and reduced-motion settings continue to suppress excess activity.

## Acceptance and verification

**Automated** (new/expanded tests):
- Four unique per-species idle and fast-typing timing profiles.
- Every production manifest has a finite warning entrance, distinct clips and reduced-motion fallback poses.
- No doubled wake transition; wake uses exactly one semantic clip.
- The alert entrance completes and reaches its persistent critical target; incidental gestures cannot override critical.
- Recovery restores the normal base state.
- Reduced-motion completion exits a one-shot wake without repeated animation.
- An entrance that ends between scheduler ticks carries leftover time into the next frame.
- A manifest validator rejects non-finite durations and looping entrances.
- Character-specific reactions differ while CHILL remains subtle.
- All existing sprite bitmap, cosmetic-anchor, runtime/scheduler, privacy and Windows packaging tests remain applicable.

**Visual/human acceptance still needed:** record actual packaged Windows footage and evaluate Byte visor/antenna motion; Mochi ears/tail/paws; Pip stretch; Kiwi independent wings/peck at 24, 32 and 48px. Check all four on desktop and in Mini/Perch, both reduced-motion and normal, at various display refresh rates; verify pointer entry and direct pet/click do not distract in Quiet mode. Use a before/after video and P2's cuteness/comfort rubric. A green CI is not a substitute for human visual approval.

## Non-goals

P5 does not add extra pixel-art frames, physics, interpolation, background character AI, new tabs/UI, or new care/feeding chores. More expressive direct *pet* interactions are P6. Rich pixel habitats are P7–P8. Later UX/navigation redesigns are P10–P13. No public release or signed installer publication occurs in P5.

## Source files

- `public/assets/characters/{byte,mochi,pip,kiwi}/manifest.json`
- `src/companion/animation/manifest.ts`
- `src/companion/animation/state-machine.ts`
- `src/companion/personality/profiles.ts`
- `src/companion/renderer/CompanionScene.svelte`
- `src/companion/animation/p5-choreography.test.ts`
- `src/companion/animation/manifest.test.ts`
- `src/companion/animation/state-machine.test.ts`

Do not claim this phase as physically Windows-certified or aesthetically approved until source-independent screen recordings and physical user tests are reviewed.
