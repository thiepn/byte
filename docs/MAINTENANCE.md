# Byte Post-Release Updates & Maintenance

Phase 28 defines how Byte evolves after its first certified public release without turning updates into a background network service.

## Release channels

Byte has two explicit channels:

- **Stable** — normal releases such as `v1.2.3`; Settings opens GitHub's latest stable release page.
- **Beta** — preview releases such as `v1.3.0-beta.1`; Settings opens the full Releases page so the user can choose a preview manually.

Changing channels only changes the fixed GitHub destination used by **Check for updates**. Byte does not poll, download, stage, or install updates in the background.

## Release preparation

All user-visible changes belong under `## [Unreleased]` in `CHANGELOG.md`.

Prepare a stable release with:

```bash
npm run release:prepare -- --version 1.2.3 --channel stable
```

Prepare a beta with:

```bash
npm run release:prepare -- --version 1.3.0-beta.1 --channel beta
```

The command:

1. validates channel/version syntax
2. refuses non-increasing versions
3. requires non-empty Unreleased notes
4. moves those notes into a dated version section
5. updates `package.json`
6. updates `package-lock.json`
7. updates `src-tauri/Cargo.toml`
8. updates `src-tauri/tauri.conf.json`

The resulting changes still require a normal PR and full certification.

## Release notes discipline

Tagged releases do not rely on automatically generated notes as the primary user-facing record.

`scripts/extract-release-notes.mjs` extracts the exact version section from `CHANGELOG.md`. The release workflow uses that text as the GitHub Release notes.

Stable and Beta releases therefore share one auditable source of release notes.

## Version policy

Byte uses SemVer-style versions:

- Stable: `MAJOR.MINOR.PATCH`
- Beta: `MAJOR.MINOR.PATCH-beta.N`

Major/minor releases may add or change product behavior.

Patch releases are for compatible fixes, security corrections, packaging fixes, or emergency recovery.

## Hotfix policy

Byte uses **patch-forward recovery**.

Because the Windows installer intentionally blocks downgrades, emergency rollback does not mean installing an older binary over a newer one.

Instead:

1. revert or repair the problematic code in a PR
2. prepare a higher patch version on the same major/minor line
3. add changelog notes
4. merge the fully certified PR to `main`
5. run **Byte Hotfix** with the base published tag and new patch tag
6. the workflow certifies source/dependencies and pushes the tag
7. the normal tagged-release workflow rebuilds and certifies the public artifacts

This keeps installer ordering, migrations, provenance, and user-data ownership consistent.

## Configuration compatibility

Byte's current configuration schema and migration floor are explicit Rust constants and mirrored in `maintenance/release-policy.json`.

Phase 28 policy:

- current schema: `9`
- minimum migratable schema: `1`
- configuration downgrades are not supported
- unknown future schema versions are rejected/quarantined rather than guessed
- schema migrations must be deterministic and covered by tests
- a schema version may not be dropped from the migration window silently; changing the floor requires an explicit compatibility decision and release note

## Deprecation policy

Persistent fields, IPC commands, settings, and serialized enum values should not be removed in the same release in which they are deprecated unless required for a security issue.

Normal removal sequence:

1. stop creating new usage
2. retain read/migration compatibility
3. document replacement
4. wait at least one stable release
5. remove only with migration/tests and release notes

Security-critical removals may be accelerated but must still preserve safe failure behavior.

## Dependency maintenance

Dependabot runs weekly for:

- npm
- Cargo
- GitHub Actions

Routine minor/patch changes are grouped to reduce PR noise.

Security-sensitive dependency changes still pass normal CI and packaging certification before merge.

## Scheduled maintenance

`.github/workflows/maintenance.yml` runs weekly and can be dispatched manually.

It has two responsibilities:

### Security audit

- maintenance-policy verification
- `npm audit --audit-level=high`
- RustSec `cargo audit`

### Runtime/toolchain compatibility

Byte builds and launches on both `windows-2022` and `windows-2025` GitHub runners.

The job performs:

- release/maintenance metadata validation
- frontend type checking
- Rust tests
- release-mode Tauri build without installer bundling
- a bounded Byte startup/WebView2 smoke test

These server runners verify Windows toolchain/WebView2 regression behavior. They do **not** replace manual certification on a currently supported Windows 11 desktop release.

## Windows support policy

Primary user support targets Windows 11 versions currently supported by Microsoft.

The installer can bootstrap WebView2 when needed. A healthy WebView2 runtime remains a Byte runtime requirement.

Older or out-of-support Windows versions may continue to work but are not part of the primary certification promise.

## Release health

Byte intentionally has no analytics/crash-reporting backend. Release health is therefore assessed through:

- certified CI/package results
- GitHub issues and user reports
- manual Windows verification
- dependency/security advisories
- scheduled compatibility jobs

See [RELEASE_HEALTH_CHECKLIST.md](RELEASE_HEALTH_CHECKLIST.md).
