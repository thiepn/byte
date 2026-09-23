# Byte

Byte is a Windows-first, privacy-first desktop companion that makes system health understandable through a cute, customizable character instead of a technical monitoring dashboard.

## Phase 4 architecture baseline

The repository now starts from a canonical Tauri 2 + Svelte 5 architecture. Phase 4 intentionally uses a development placeholder companion and mock snapshot so production telemetry, artwork, habitats, and UX can be added without prototype debt.

- Rust owns authoritative state, versioned preferences, lifecycle ownership, and the telemetry-source boundary.
- Svelte owns presentation and companion rendering.
- Windows: `companion`, `quick-panel`, and `main`.
- Browser development falls back to deterministic local mock data.
- Global-input types represent anonymous activity only.
- Cleaners, RAM trimming, generic process killing, analytics, and arbitrary command execution are absent.

## Development

Prerequisites: Node.js 22+, Rust stable, Windows 11 recommended, and normal Tauri/WebView2 Windows prerequisites.

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

Next: Phase 5 replaces the mock snapshot with Telemetry Engine V2.
