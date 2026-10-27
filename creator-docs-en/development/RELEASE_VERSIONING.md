# Release versioning strategy (independent releases)

[中文](../../creator-docs/development/RELEASE_VERSIONING.md)

**Conclusion:** **Desktop app, CLI, and kernel semver crates keep independent SemVer**; first public release does not force a single global number.

**SSOT scope**: independent versioning and candidate mapping rules. Current numbers, source manifests and verification status live only in [PROJECT_CURRENT_STATUS](../getting-started/PROJECT_CURRENT_STATUS.md).

| Artifact | Release cadence | Notes |
|----------|-----------------|-------|
| **Desktop Tauri** | User-visible features and installers | Unpublished changes remain in [CHANGELOG](../../CHANGELOG.en.md) `[Unreleased]` |
| **`oclive-cli`** | Scaffolding and toolchain | Breaking changes: [DEPRECATED_COMMANDS](../../kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md) |
| **`oclive_kernel_runtime`** | Runtime and HTTP/`--api` contract | See [COMPATIBILITY](../COMPATIBILITY.md) |
| **`oclive_validation`** | Role-pack / blueprint validation | Aligned with pack-editor wasm validation |

## Why versions are not bumped together

- Desktop releases bundle frontend and installers; CLI can update alone via `cargo install`.
- Kernel semver crates are referenced by multiple hosts (Tauri, `kernel_server`, future launchers); **contract changes** should ship independently of UI releases.
- [COMPATIBILITY.md](../COMPATIBILITY.md) expresses cross-artifact compatibility via **`min_runtime_version`** / `API_VERSION`, not one global version.

## Current release candidate

1. See the [status snapshot](../getting-started/PROJECT_CURRENT_STATUS.md) for the candidate number, baseline and publication status. A local rollback tag is not a public release. Freeze the target commit and pass applicable local gates plus its remote CI before release.
2. Windows MSI uses the numeric version in Tauri configuration, mapped to desktop SemVer in the status snapshot. CLI/kernel versions do not bump with the desktop candidate; dropping a prerelease suffix must not decrease the MSI numeric version.
3. Breaking role packs: must link [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md).

## Ongoing

- When **`oclive_kernel_runtime` major** bumps, re-check `min_runtime_version` and the OOCP suite.
- Monolith / embedded deliverable versions follow the **CLI + template** that produced the binary, recorded in `bench_history.json` (local, not committed).
