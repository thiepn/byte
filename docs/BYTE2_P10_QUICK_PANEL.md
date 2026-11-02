# Byte P10 — Companion-first Quick Panel

This draft changes only the existing Quick Panel presentation. It uses current cached state, existing backend actions, and the shared production character animation pipeline. It does not change product telemetry, release version, account state, window position or persistence.

- The current selected character is rendered with the production 64-pixel atlas, palette and selected cosmetics. On asset failure the static production preview remains available.
- The shared 12 FPS `animationScheduler` runs only while the Quick Panel mascot is mounted and motion is enabled. It unsubscribes on close/unmount and respects Windows/Byte reduced-motion preferences. No new sampling, polling, frame loop or IPC permissions.
- Live mascot uses the same status-to-semantic-behavior map as the desktop companion, including critical priority. It does not display an invented wellbeing score.
- One dominant primary action uses the existing safe allowlist; any unavailable recommended action falls back to View details. No arbitrary command or process termination.
- Detailed metrics and issue attribution are still available via native keyboard-operable disclosure controls; display mode, Move Byte and click-through remain directly visible, including when monitoring is unavailable.
- Escape closes the panel, the existing focus/blur and two-second cache-only refresh lifecycle remain untouched.

## Verification
Run original TypeScript/Svelte check, frontend tests, Rust tests and Windows Packaging. Review the Quick Panel at 100–200% text scale, high contrast, keyboard-only access, reduced motion, monitoring-off state, critical alerts, selected cosmetics, all four mascots, and actual Windows tray/window lifecycle on a representative device before claiming complete visual acceptance. Do not merge or publish on automated evidence alone.
