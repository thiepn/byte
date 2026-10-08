# Byte 2.0 — P6 Direct Character Interactions

P6 adds direct, opt-in interaction with the **actual rendered pixel sprite**. It reuses the P3/P4 production art and P5 one-shot animation clips. No new sprites, animations, hooks, telemetry, permissions or persistent settings are needed.

## What users can do

| Gesture | How it works | Result |
|---|---|---|
| Tap | Single primary-button click directly on opaque character pixels | Character-specific tap acknowledgement |
| Wave | Double-tap the character within 360 ms | Distinct greeting / wave reaction |
| Pet | Stroke the character at least 26 screen pixels while keeping the pointer inside the artwork | Character-specific affectionate/curious reaction |
| Cuddle | Press and hold a character for at least 650 ms | A calmer character-specific response |
| Status | Click **any background area** or transparent part of the character canvas | Existing Quick Panel opens |
| Keyboard | Focus Byte, then press **P** to pet, **W** to wave, **H** to cuddle | Accessible equivalent to direct gesture |
| Keyboard status | Focus Byte, then press **Enter** or **Space** | Existing Quick Panel opens |
| Move Mode | Use the existing tray Move Byte command, then drag the companion normally | All direct character interactions are disabled during movement |
| Click-through | Use the existing system-tray toggle | The native window ignores cursor input entirely; interactions do not bypass it |

The transparent pixel-hit test reads a **single alpha pixel** from the already-rendered character canvas, including the player's current cosmetics. It is only performed on the initial press and subsequent **active** gesture movements, never on continuous passive cursor movement. Clicking empty pixel areas still opens status. The local pointer position is never written to disk, sent over IPC, stored in preferences or forwarded to a backend.

### Species responses

Only existing one-shot semantic behaviors are used; all four production manifest mappings are checked by tests.

| Character | Tap | Wave | Pet | Cuddle |
|---|---|---|---|---|
| Byte | Tiny click/tap | Authored rare A robot greeting | Happy | Curious glance |
| Mochi | Blink | Authored rare B kitten greeting | Happy | Authored rare A comfort |
| Pip | Surprised | Happy wobble | Authored rare A squish | Authored rare B settling |
| Kiwi | Curious peck | Happy wing-up | Authored rare B gesture | Authored rare A settling |

A simple first tap responds immediately; a second tap within 360 ms becomes a wave. A stroke or a long press fires **at most once**. The controller rejects an unexpected second pointer, ignores tiny accidental jitter, aborts when the pointer exits the sprite, and resets when the character changes, moves, hides or suspends.

Long presses are recognized from **the existing 12 FPS animation scheduler**, not from a new timer. Tap and double-tap are resolved on pointer release; long-press feedback begins during the hold. No background gesture processor or animation loop is created.

## Safety and comfort

- Direct gestures are issued as `source: "interaction"` requests to the existing `CharacterAnimator`; critical and diagnostic states cannot be overridden.
- All mapped clips are one-shots, so no interaction leaves the companion trapped in a continuous action.
- `QUIET` interaction level still permits deliberately initiated reactions but suppresses the extra environmental effect. `prefers-reduced-motion` and Byte's Reduce Motion preference continue to control static animation fallbacks and disabling particle effects.
- The old Quick Panel interaction remains reachable by clicking the background, using the keyboard, or via the system tray. Move Mode's native drag behavior is unchanged.
- No native click-through bypass, mouse hooks, global cursor coordinates, or persistent interaction history.

## Implementation

- `src/companion/interaction/direct-interaction.ts`: pure deterministic gesture recognizer, sprite-specific action map, and alpha hit test.
- `src/companion/interaction/direct-interaction.test.ts`: direct gesture classification, double-tap timing, touch/second-pointer cancellation, stroke threshold, hold timing, alpha hit testing and compatibility with all four actual character manifests.
- `src/companion/renderer/CompanionScene.svelte`: routes local pointer and keyboard events, preserves status background and Move Mode, cancels on lifecycle/pref changes, advances hold on the shared scheduler and respects QUIET/reduced motion.

## Acceptance criteria

**Automated**: complete existing frontend, Rust, Windows packaging and sprite-evidence CI; tests for tap, wave, pet, hold, cancellation, transparent pixel corners, four separate gesture maps, and finite clips.

**Manual Windows qualification (not yet performed):**

1. Run the packaged Windows app, switch Byte/Mochi/Pip/Kiwi and all display modes (Habitat, Perch, Mini, Edge); verify accurate hit targets on 100–200% scaling and characters with accessories.
2. Tap, stroke, double-tap and hold each mascot; verify one reaction per gesture, distinguishable emotional responses, and absence of quick-panel popups on the opaque sprite.
3. Click empty scene areas and transparent atlas corners; verify Quick Panel access.
4. Use keyboard focus + P/W/H/Enter/Space. Verify reduced-motion fallback frames, QUIET (no extra particles), full screen suppression, Tray, hidden companion and mouse click-through.
5. Enter Move Mode and drag; confirm direct gestures never steal native window movement or leave the window stuck.

Actual Windows screenshots/videos and user visual approval remain required before a public signed Byte 2.0 release. P7 adds redesigned habitats; P8 adds subtle environmental visual effects.
