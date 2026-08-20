# Release versioning strategy (independent releases)

[中文](../../creator-docs/development/RELEASE_VERSIONING.md)

**Conclusion:** **Desktop app, CLI, and kernel semver crates keep independent SemVer**; first public release does not force a single global number.

| Artifact | Current version (`main`, 2026-08-20) | Release cadence | Notes |
|----------|-----------------------------------------------|-----------------|-------|
| **Desktop Tauri** (`package.json` / `distros/desktop-tauri`) | **0.5.0** | User-visible features and installers | Changes in [CHANGELOG.md](../../CHANGELOG.md) `[0.5.0]` |
| **`oclive-cli`** | **0.1.0** | Scaffolding and toolchain | CLI breaking changes: [DEPRECATED_COMMANDS.md](../../kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md) |
| **`oclive_kernel_runtime`** | **0.2.0** (semver crate) | HTTP / `--api` contract | See [COMPATIBILITY.md](../COMPATIBILITY.md) |
| **`oclive_validation`** | **0.1.0** | Role pack / blueprint validation | Aligned with pack-editor wasm |

## Why versions are not bumped together

- Desktop releases bundle frontend and installers; CLI can update alone via `cargo install`.
- Kernel semver crates are referenced by multiple hosts (Tauri, `kernel_server`, future launchers); **contract changes** should ship independently of UI releases.
- [COMPATIBILITY.md](../COMPATIBILITY.md) expresses cross-artifact compatibility via **`min_runtime_version`** / `API_VERSION`, not one global version.

## First public release recommendations

1. [CHANGELOG.md](../../CHANGELOG.md) **`[0.5.0] - 2026-08-20`** is prepared; tag **`oclivenewnew-v0.5.0`** on release day. This internal debut supersedes the empty-asset placeholder release from 2026-07-10.
2. Tag: **`oclivenewnew-v0.5.0`** (desktop); optional **`oclive-cli-v0.1.0`**.
3. Breaking role packs: must link [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md).

## Ongoing

- When **`oclive_kernel_runtime` major** bumps, re-check `min_runtime_version` and the OOCP suite.
- Monolith / embedded deliverable versions follow the **CLI + template** that produced the binary, recorded in `bench_history.json` (local, not committed).
