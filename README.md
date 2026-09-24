# Byte

Byte is a Windows-first, privacy-first desktop companion that makes system health understandable through a cute, customizable character instead of a technical monitoring dashboard.

## Current foundation

Phases 1–14 are implemented.

Byte now has a canonical Tauri 2 + Svelte 5 architecture plus one production Windows telemetry pipeline.

- Rust owns authoritative state, versioned preferences, lifecycle, and system telemetry.
- Svelte owns presentation and companion rendering.
- CPU, RAM, storage, network, battery, and best-effort thermal data are collected locally.
- UI snapshot reads are cache-only; opening a panel never triggers a second hardware poll.
- Volatile telemetry is smoothed without startup bias.
- Battery, thermal, and storage are optional/cached where appropriate.
- The telemetry worker is lifecycle-aware and joined on explicit shutdown.
- Human-friendly diagnostics distinguish CALM, BUSY, STRESSED, and NEEDS_ATTENTION.
- CPU/memory process attribution is lazy and only names a culprit when confidence is meaningful.
- Issues use sustained timing, hysteresis, recovery, priority, and safe recommended actions.
- Native windowing supports Habitat, Perch, Mini, Edge, and Tray modes with DPI-safe sizing.
- Companion placement is saved per mode as monitor-relative coordinates and restored safely after display changes.
- Perch uses the Windows work area so it stays taskbar-aware.
- Move Mode, native dragging, click-through, tray controls, and adjacent Quick Panel placement are implemented.
- Character animation is manifest-driven with one shared 12 FPS scheduler, semantic behavior priorities, transitions, reduced-motion fallbacks, deterministic idle scheduling, anchors, and canvas sprite rendering.
- Byte, Mochi, Pip, and Kiwi ship as distinct production sprite families with complete behavior coverage and eight palettes each.
- Windows global keyboard/click/scroll activity drives privacy-safe typing, click, fast-typing, and idle reactions.
- Meadow, Cozy Desk, Bedroom, Space, Aquarium, and Rooftop are production habitats with authored depth, local-time variants, semantic system reactions, fixed decoration slots, and bounded ambience.
- Phase 13 adds persistent live customization: character, palette, habitat, 12 anchor-based character cosmetics, 12 fixed-slot habitat decorations, display mode, and size controls.
- Phase 14 adds Chill, Curious, and Energetic personality profiles with deterministic idle pacing, personality-aware wind-down, safe interaction follow-ups, charging celebration, and personality-scaled ambient habitat life.
- Personality never weakens diagnostics or captures cursor coordinates; direct warning states keep their existing priority and privacy boundaries.
- Customization and personality are entirely local; there is no account, store, virtual currency, unlock timer, or cloud inventory.
- Cleaners, RAM trimming, generic process killing, analytics, and arbitrary command execution are absent.

## Development

Prerequisites: Node.js 24+, Rust stable, Windows 11 recommended, and normal Tauri/WebView2 Windows prerequisites.

```bash
npm install
npm run verify
npm run tauri:dev
```

Rust checks:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml
```

## Source of truth

- [Product contract](docs/PRODUCT_CONTRACT.md)
- [UX contract](docs/UX_SPEC.md)
- [Visual contract](docs/DESIGN_SYSTEM.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Telemetry engine](docs/TELEMETRY.md)
- [System intelligence](docs/DIAGNOSTICS.md)
- [Native windowing](docs/WINDOWING.md)
- [Character animation runtime](docs/ANIMATION_RUNTIME.md)
- [Production characters](docs/CHARACTERS.md)
- [Global input reactions](docs/INPUT_REACTIONS.md)
- [Habitat rendering runtime](docs/HABITAT_RUNTIME.md)
- [Production habitats](docs/HABITAT_PRODUCTION.md)
- [Customization and decoration](docs/CUSTOMIZATION.md)
- [Personality and ambient behavior](docs/PERSONALITY.md)

Next: Phase 15 — Quick Panel.
