# Byte Character Animation Runtime

Phase 8 establishes the reusable animation engine used by every companion. Final character art is intentionally not part of this phase.

## Runtime pipeline

Character manifest
→ validated asset registry
→ CharacterAnimator
→ shared AnimationScheduler
→ RenderFrame
→ CharacterCanvasRenderer

The renderer never decides what the character means. It draws the frame selected by the semantic state machine.

## Character manifest

Every character ships a versioned manifest containing:

- native art size
- animation canvas size
- sprite atlas metadata
- named frames
- per-frame cosmetic anchors
- animation clips
- semantic behavior → clip mapping
- optional transition clips
- reduced-motion fallback frames
- idle behavior profile

Character-specific frame numbers never appear in system-monitoring code.

## Semantic behaviors

The runtime supports the Phase 3 behavior vocabulary, including:

idle, blink, sleep, wake, movement, typing, clicks, petting, happy, surprised, curious, annoyed, busy, stressed, heat, memory pressure, low battery, charging, network activity, and needs-attention.

Characters are free to map multiple behaviors to the same clip when their production art does not need distinct animation.

## Priority model

Animation requests use semantic sources:

1. critical
2. diagnostic
3. interaction
4. system
5. input
6. personality
7. idle

Higher-priority behavior can interrupt lower-priority behavior. Lower-priority activity cannot stomp an active warning or direct interaction. Clips can additionally opt out of equal/lower interruption.

## Base versus transient behavior

The animator maintains one persistent base behavior derived from current system state.

Examples:

- CALM → idle
- BUSY → busy
- STRESSED + memory issue → memory_pressure
- STRESSED + thermal issue → hot
- NEEDS_ATTENTION → needs_attention

Transient interactions such as blink, click, pet, or future typing taps temporarily interrupt that base and return to it automatically when their one-shot clip ends.

## Transitions

The manifest may define transitions such as:

- idle → sleep
- sleep → wake
- idle/busy/stressed → needs_attention

The state machine plays the transition clip first, then enters the requested target behavior.

## One shared scheduler

The entire companion scene uses one requestAnimationFrame scheduler capped at approximately 12 animation updates per second.

Individual characters, cosmetics, particles, or UI components must not create their own frame loops.

The scheduler:

- emits at the target cadence
- clamps delta to 125 ms
- stops while the document is hidden
- resets timing on visibility resume

This prevents hidden/background tabs from fast-forwarding the character after a long pause.

## Reduced motion

The runtime follows prefers-reduced-motion immediately.

Each clip can specify a static reduced-motion frame. Looping behaviors hold that pose. One-shot behaviors preserve completion timing with a short reduced-motion duration and then return to the base state.

Core meaning therefore survives without bouncing, shaking, or rapid animation.

## Idle scheduling

Idle reactions are data-driven and deterministic for the seed supplied by the host.

The manifest defines:

- minimum idle delay
- maximum idle delay
- weighted idle behaviors

The current development Byte can blink, look curious, or perform a small happy reaction. Production characters will receive their own weighted profiles in Phase 9/14.

## Cosmetic anchors

Every sprite frame provides the shared anchor vocabulary:

- head
- face
- body
- back
- left_hand
- right_hand
- ground

Anchors may also carry rotation, flip, and layer metadata.

The canvas renderer already supports generic cosmetic attachments before or after the base sprite. Phase 13 will populate that system with actual hats, glasses, scarves, props, and character-specific compatibility data.

## Pixel rendering

The production renderer:

- renders the native animation canvas
- disables canvas image smoothing
- uses integer atlas frame rectangles
- lets CSS scale the final canvas using pixelated/crisp rendering

This avoids interpolated blurry sprites at normal desktop sizes.

## Development atlas

Phase 8 includes a deliberately simple eight-frame Byte development atlas. It exists only to prove the runtime end-to-end.

It is not final art and should be removed/replaced by the Phase 9 production character assets.

## Current system integration

Until a later event-stream optimization phase, the companion reads the already-cached SystemSnapshot every two seconds and only updates the animator's base semantic behavior.

This does not trigger hardware sampling. The authoritative telemetry loop remains in Rust.

## Acceptance rules

A new production character should require:

1. a valid manifest,
2. a sprite atlas,
3. semantic behavior mappings,
4. frame anchors.

It should not require changes to telemetry, diagnostics, the scheduler, or the animation state machine.
