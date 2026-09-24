# Byte

Byte is a Windows-first, privacy-first desktop companion that makes system health understandable through a cute, customizable character instead of a technical monitoring dashboard.

## Current foundation

Phases 1–25 are implemented.

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
- Byte, Mochi, Pip, and Kiwi ship as distinct production sprite families with complete behavior coverage and nine palettes each, including the optional Aurora collection colorway.
- Windows global keyboard/click/scroll activity drives privacy-safe typing, click, fast-typing, and idle reactions.
- Meadow, Cozy Desk, Bedroom, Space, Aquarium, and Rooftop are production habitats with authored depth, local-time variants, semantic system reactions, fixed decoration slots, and bounded ambience.
- Phase 13 adds persistent live customization: character, palette, habitat, 12 anchor-based character cosmetics, 12 fixed-slot habitat decorations, display mode, and size controls.
- Phase 14 adds Chill, Curious, and Energetic personality profiles with deterministic idle pacing, personality-aware wind-down, safe interaction follow-ups, charging celebration, and personality-scaled ambient habitat life.
- Personality never weakens diagnostics or captures cursor coordinates; direct warning states keep their existing priority and privacy boundaries.
- Phase 15 promotes the Quick Panel into the compact operational surface: live cache-only status, primary issue explanation, resource cards, safe allowlisted Windows actions, display-mode controls, Move Mode, click-through, and full-app access.
- Phase 16 promotes the main application Overview and Activity surfaces: live resource summaries with bounded session trends, persisted meaningful issue/power events, timeline filtering, and a richer investigation path without becoming a raw telemetry dashboard.
- Phase 17 adds explicit on-demand Apps diagnostics: locally aggregated process groups, CPU/memory shares, conservative standout confidence, point-in-time sorting, diagnostic context, and Task Manager handoff with no End task control.
- Phase 18 turns Customize into a full live Customization Studio with a large animated habitat preview, reaction testing, visual category browsing, four curated looks, optimistic queued persistence, undo/redo, reset controls, and the complete existing character/cosmetic/decor/personality/display toolset.
- Phase 19 adds a finite optional collection layer: eight local extras unlocked through elapsed time, natural anonymous typing, ordinary charging/network moments, and naturally encountered rare idles. Core customization remains immediately available.
- Phase 20 adds the production first-run and Settings layer: a short four-step onboarding flow, real current-user Windows startup registration, fullscreen companion auto-hide/restore, system-monitoring controls, sustained NEEDS_ATTENTION notifications, accessibility overrides, local history clearing, privacy explanations, and release-channel access.
- Phase 21 replaces the simple Phase 20 alert gate with Smart Notifications: all-active-issue routing, critical-category filters, a 10-minute confident runaway-process rule, persisted per-category cooldowns, duplicate suppression, Quiet mode, finite snooze controls, Windows permission awareness, and concise cause/next-step notification text.
- Phase 22 hardens Byte around fullscreen games, movies, presentations, lock/display-off state, and screen capture: centralized desktop-awareness suppression, 1.5s restore grace, display-sleep telemetry/render suspension, fresh post-sleep diagnostics, restore-safe windowing guards, Windows presentation/fullscreen signals, capture exclusion, and optional local foreground-app exclusions.
- Phase 23 makes that lifecycle power-aware end to end: telemetry uses adaptive 1.5–8 second sampling, lock/display/system sleep block heavy monitoring, fullscreen-reduced mode pauses input/render work and slows telemetry, hidden main-window polling stops, companion telemetry becomes event-driven, process attribution scans are sustained-issue-only, and the desktop-awareness loop becomes the single low-frequency sleep sentinel.
- Phase 24 hardens accessibility and failure recovery: keyboard navigation and focus transfer, screen-reader selected/progress/error semantics, forced-colors support, 100–125% text-scale resilience without scaling companion pixel art, explicit unavailable-monitoring states, missing-asset preservation, display hot-unplug recovery, and non-fatal optional Windows integrations.
- Phase 25 hardens security and privacy: per-window Tauri command ACLs, listen-only frontend event permissions, strict local CSP plus prototype freezing, backend allowlists for preference/customization payloads, bounded persisted-state reads, generic privacy-safe native notifications, non-persistent process attribution in Activity history, exact security-sensitive Tauri pins, SHA-pinned CI actions, dependency audits, and Dependabot coverage.
- Activity persists only meaningful events locally; trend points are session-only and globally bounded.
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
- [Quick Panel](docs/QUICK_PANEL.md)
- [Main app: Overview & Activity](docs/FULL_APP.md)
- [Apps & Diagnostics](docs/APPS_DIAGNOSTICS.md)
- [Customization Studio](docs/CUSTOMIZATION_STUDIO.md)
- [Lightweight collection & unlocks](docs/COLLECTION.md)
- [Onboarding & Settings](docs/ONBOARDING_SETTINGS.md)
- [Smart Notifications](docs/SMART_NOTIFICATIONS.md)
- [Desktop awareness](docs/DESKTOP_AWARENESS.md)
- [Power & performance](docs/PERFORMANCE.md)
- [Accessibility & resilience](docs/ACCESSIBILITY_RESILIENCE.md)
- [Security & privacy](docs/SECURITY_PRIVACY.md)

Next: Phase 26 — Packaging, Installer & Release Engineering.
