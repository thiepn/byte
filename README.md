# Byte

Byte is a Windows-first, privacy-first desktop companion that makes system health understandable through a cute, customizable character instead of a technical monitoring dashboard.

## Current foundation

Phases 1–5 are implemented.

Byte now has a canonical Tauri 2 + Svelte 5 architecture plus one production Windows telemetry pipeline.

- Rust owns authoritative state, versioned preferences, lifecycle, and system telemetry.
- Svelte owns presentation and companion rendering.
- CPU, RAM, storage, network, battery, and best-effort thermal data are collected locally.
- UI snapshot reads are cache-only; opening a panel never triggers a second hardware poll.
- Volatile telemetry is smoothed without startup bias.
- Battery, thermal, and storage are optional/cached where appropriate.
- The telemetry worker is lifecycle-aware and joined on explicit shutdown.
- Global-input types represent anonymous activity only.
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

Next: Phase 6 — Human-Friendly System Intelligence.
