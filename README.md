# Byte

Byte is a Windows-first, privacy-first desktop companion that makes system health understandable through a cute, customizable character instead of a technical monitoring dashboard.

## Current status

Byte is in the v1.0 architecture foundation stage. The repository intentionally starts from a clean Tauri 2 + Svelte 5 baseline.

### Product principles

- Cute first: the companion should be worth keeping on the desktop even when the PC is healthy.
- Helpful second: important system conditions are explained in plain language.
- Lightweight: Byte must not become the resource problem it monitors.
- Private: no accounts, ads, analytics, cloud sync, or key-content logging.
- Calm: normal heavy workloads are "busy", not automatically treated as unhealthy.

## Development

Prerequisites:
- Node.js 20+
- Rust stable
- Windows WebView2 / Tauri build prerequisites

```bash
npm install
npm run check
npm run tauri:dev
```

The authoritative v1 contracts live in `docs/`.
