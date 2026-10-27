# Release versioning strategy (independent releases)

[中文](../../creator-docs/development/RELEASE_VERSIONING.md)

**Conclusion:** **Desktop app, CLI, and kernel semver crates keep independent SemVer**; first public release does not force a single global number.

| Artifact | Current working-tree version (2026-10-27) | Release cadence | Notes |
|----------|-----------------------------------------------|-----------------|-------|
| **Desktop Tauri** (`package.json` / `distros/desktop-tauri`) | **0.6.0-rc.2** | User-visible features and installers | Candidate; changes remain in [CHANGELOG.en.md](../../CHANGELOG.en.md) `[Unreleased]` |
| **`oclive-cli`** | **0.1.0** | Scaffolding and toolchain | CLI breaking changes: [DEPRECATED_COMMANDS.md](../../kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md) |
| **`oclive_kernel_runtime`** | **0.2.0** (semver crate) | HTTP / `--api` contract | See [COMPATIBILITY.md](../COMPATIBILITY.md) |
| **`oclive_validation`** | **0.1.0** | Role pack / blueprint validation | Aligned with pack-editor wasm |

## Why versions are not bumped together

- Desktop releases bundle frontend and installers; CLI can update alone via `cargo install`.
- Kernel semver crates are referenced by multiple hosts (Tauri, `kernel_server`, future launchers); **contract changes** should ship independently of UI releases.
- [COMPATIBILITY.md](../COMPATIBILITY.md) expresses cross-artifact compatibility via **`min_runtime_version`** / `API_VERSION`, not one global version.

## Current release candidate

1. **`0.6.0-rc.2`** identifies the local internal-test candidate with the new role workspace, replacing the old test entry point. Local rollback baseline tags exist; no public release has been created. Freeze the target commit and pass applicable gates plus remote CI for that commit before release.
2. MSI uses numeric version **`0.6.0.2`**; other desktop identifiers remain **`0.6.0-rc.2`**. CLI and kernel versions do not bump with the desktop candidate.
3. Breaking role packs: must link [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md).

## Ongoing

- When **`oclive_kernel_runtime` major** bumps, re-check `min_runtime_version` and the OOCP suite.
- Monolith / embedded deliverable versions follow the **CLI + template** that produced the binary, recorded in `bench_history.json` (local, not committed).
