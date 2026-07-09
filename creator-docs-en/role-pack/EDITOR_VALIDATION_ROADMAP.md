# Role pack editor: validation roadmap

[中文](../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md)

The pack editor (separate repo, e.g. sibling `oclive-pack-editor`) **does not share a test process** with the runtime; contract and schema authority remain in this repo.

## Short term (current)

- **Authority:** runtime **`load_role`**: top-level JSON key allowlist (`oclive_validation::json_keys`), merged **`validate_disk_manifest`**, **`validate_min_runtime_version`** (vs `CARGO_PKG_VERSION`) — see `kernel/crates/oclive_kernel_host/src/infrastructure/storage.rs`.
- **Editor:** before export, run **top-level key checks** on `manifest.json` / `settings.json` (same allowlist as Rust — `oclive-pack-editor/src/lib/jsonKeys.ts`); if wasm is built (`npm run wasm:build`), **`validateManifestWasm`** matches **`validate_disk_manifest` + `validate_min_runtime_version`**; else TypeScript light checks + **`validateMinRuntimeVersion`** (`HOST_RUNTIME_VERSION` must align with oclivenewnew `Cargo.toml`).
- **Acceptance:** export pack → set **`OCLIVE_ROLES_DIR`** to roles root → load and chat in oclive.

## Medium term (optional)

- Extract **`role_manifest_validate`** into a **standalone Rust semver crate** (separate repo or sub-crate here):
  - Runtime **git dependency**; or
  - **CLI** (`oclive-validate-pack path/to/role`) invoked by editor subprocess/CI.
- **Keep two repos separate:** editor UI repo does not embed runtime source; crate boundary is published semver crate / git tag.

## Documentation relationship

- Pack fields and version semantics: `distros/chat-pro/roles/README_MANIFEST.md`, [PACK_VERSIONING.md](./PACK_VERSIONING.md).
- Editor README links to these paths to avoid duplicate drift.
