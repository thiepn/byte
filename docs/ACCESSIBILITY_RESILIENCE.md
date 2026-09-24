# Byte Accessibility, Robustness & Edge-Case Contract

Phase 24 makes Byte usable when ordinary assumptions fail: keyboard-only use, screen readers, Windows forced-colors, larger text, missing optional integrations, unavailable telemetry, missing assets, monitor changes, and partial Windows API failures.

## Accessibility baseline

Byte's application surfaces use native HTML controls wherever possible.

### Keyboard

- The main application exposes a **Skip to main content** link.
- Sidebar navigation reports the active page with `aria-current="page"`.
- Moving between main sections transfers programmatic focus to the main region so keyboard and screen-reader users do not have to traverse the sidebar again.
- Quick Panel supports **Escape** to close and receives focus when opened.
- Toggle-like visual button groups expose `aria-pressed`.
- Onboarding moves focus to the new step heading after Back/Continue.
- Focus indicators apply to buttons, links, inputs, selects, textareas, and programmatically focusable elements.

### Screen readers

- Error messages use alert semantics.
- Save/progress states use polite live regions where repeated interruption would be inappropriate.
- Onboarding and collection progress expose native progressbar semantics and numeric values.
- Decorative pixel previews remain hidden from the accessibility tree when the adjacent text already names the item.
- Diagnostic meaning is always present in text. Color, particles, animation, and sound are supplemental.

### Contrast

Byte supports three complementary paths:

1. normal light/dark theme
2. Byte's explicit High contrast preference
3. Windows/browser **forced-colors** mode

Forced-colors uses system colors, preserves visible borders/focus, and gives selected/active controls a non-color-only outline.

### Motion

Reduced motion combines:

- the operating-system `prefers-reduced-motion` preference
- Byte's explicit Reduce motion preference

CSS transitions/animations collapse to effectively instant behavior, while the companion animation engine uses its authored reduced-motion path.

### Text scale

Byte supports 100%, 110%, and 125% interface scaling.

- Main application and Quick Panel scale.
- Companion pixel art does **not** scale with the text preference.
- Page headings/rows wrap rather than assuming a single line.
- Settings rows become stacked at narrow effective widths.
- Apps diagnostics may horizontally scroll instead of clipping data.
- Onboarding progressively collapses to one-column layouts.

## Graceful degradation

Optional platform integrations must never become a reason Byte cannot launch.

| Integration | Failure behavior |
| --- | --- |
| Desktop awareness worker | Byte continues; fullscreen/lock awareness may be unavailable |
| Capture exclusion | Byte continues; exclusion remains best-effort |
| Telemetry worker startup | UI remains available with an explicit unavailable state |
| Global input hooks | Passive typing/click reactions are disabled; core companion remains usable |
| Notification permission/API | In-app diagnostics remain authoritative |
| Optional cosmetic asset | Missing attachment is omitted |
| Selected character asset in Studio | Current look is preserved and an error is shown |

Byte never replaces unavailable telemetry with fabricated numbers.

## Display and window edge cases

Monitor-relative placements already survive normal DPI/display changes. Phase 24 additionally hardens hot-unplug behavior:

- Move Mode clears before placement/layout recovery work.
- A monitor disappearing during a drag cannot leave Byte stuck in Move Mode.
- placement saving and Quick Panel positioning use the same current → primary → available-monitor fallback path.
- saved coordinates remain clamped to the selected monitor work area.

If Windows reports no usable monitor at all, the operation returns an explicit window error instead of inventing coordinates.

## Persistence and races

Existing persistence guarantees remain unchanged:

- config writes are atomic temporary-file replacements
- corrupt config is quarantined and defaults are restored
- mutex poison recovery uses the owned inner value
- Studio preference writes remain serialized
- telemetry remains single-source/cached

Phase 24 does not add a second recovery database or cloud backup path.

## Verification

CI must pass:

- Svelte/TypeScript check
- frontend tests
- frontend production build
- Rust format
- Rust tests
- Clippy with warnings denied
- Rust check

Phase 24 adds deterministic tests for accessibility scale normalization and the invariant that text scaling never changes companion pixel-art scale.

Manual release certification should additionally cover:

- Tab/Shift+Tab across onboarding, main navigation, Settings, Apps, Studio, and Quick Panel
- Escape-close from Quick Panel
- Windows High Contrast / forced-colors
- 125% Byte text scale at the minimum main-window size
- Windows reduced motion + Byte Reduce motion
- monitoring disabled
- denied notification permission
- missing/corrupt optional asset
- unplugging/rearranging monitors during Move Mode
- lock/unlock and display sleep/wake
- launch with optional Windows integration failure injected where practical
