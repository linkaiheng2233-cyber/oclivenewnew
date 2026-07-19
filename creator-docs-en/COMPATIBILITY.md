# Pack editor (oclive-pack-editor) ↔ host (oclivenewnew) compatibility

[中文](../creator-docs/COMPATIBILITY.md)

This page explains how **`ui.json` in role packs** relates to the **desktop host**, so authors do not ship fields the host ignores—or miss fields the host already supports.

**Version format**: both repos use **SemVer** `MAJOR.MINOR.PATCH` from root **`package.json`** `version`.

**Snapshot (aligned with release review)**:

- **oclivenewnew** (desktop / Tauri host): **`0.5.0`** (root `package.json` and `distros/desktop-tauri/Cargo.toml` must match)
- **oclive_kernel_runtime** (shared contract crate): **`0.2.0`** (`kernel/crates/oclive_kernel_runtime/Cargo.toml`; DTO / `API_VERSION` live in that crate)
- **oclive-cli** (scaffold CLI): **`0.1.0`** (`kernel/crates/oclive-cli/Cargo.toml`; **independent semver**, not required to match the desktop host; when scaffolding with `init --kernel-source`, path deps align contracts). **Default build** depends on `oclive_kernel_runtime` + `oclive_validation` only (`cargo tree -p oclive-cli --no-default-features` has **no** `libsqlite3-sys` / `axum`). **`doctor config-resolve`** defaults to runtime pure resolution; **`--via-host`** (feature **`diagnostics-host`**) optionally runs in-memory `AppState` deep diagnosis.
- **oclive-pack-editor** (sister repo): **`0.5.0`** (`ui.json` parity with host **0.5.x**)
- **oclive-vscode** (VS Code extension, sister repo): **`0.4.1`** (independent semver; spawn/attach contract needs host **≥0.4.0**, **0.5.0** recommended)

---

## Compatibility matrix

| Editor version | Minimum host | New or required `ui.json` capability | Notes |
|----------------|--------------|--------------------------------------|-------|
| **0.2.x** | **0.2.0** | `shell`, `slots` (`chat_toolbar`, `settings_panel`, `role_detail`, etc.), base `theme` / `layout` (see schema) | historical baseline |
| **0.3.x** | **0.3.0** | extended theme/layout fields per release notes | lower hosts usually **ignore unknown fields** |
| **0.4.x** | **0.4.0** | full **`sidebar`**, **`chat.header`**, etc. need host **directory bootstrap** for those slots ([DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)) | slot names must match host `pluginStore` constants |
| **0.5.x** | **0.5.0** | portrait catalog / `visual_presentation` export aligned with host `display_metrics` and voice side-channel `ui.json` slot seeds | see [CHANGELOG.en.md](../../CHANGELOG.en.md) `[0.5.0]` |
| **dev** | **same dev** | schema and host `UiConfig` on the same branch | local pairing only |

---

## Upgrade and downgrade behavior

1. **Host older than editor target**  
   - Unknown **`ui.json`** keys: usually **silently ignored** when models use **`serde` defaults + optional fields**; if a release rejects unknown keys, see that version’s `CHANGELOG`.  
   - Declared but unimplemented slots: may **not render** or **do nothing** until the host is upgraded.

2. **Editor older than host**  
   - New host slots / theme keys may be uneditable in the old editor; **edit `ui.json` manually** against [ui.json.schema.json](role-pack/ui.json.schema.json).

3. **Pack `settings.json` and `plugin_backends`**  
   - Governed by **`min_runtime_version`** and host `load_role`; see [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md) and [CHANGELOG.en.md](../../CHANGELOG.en.md).

---

## One-page external compatibility (host / editor / launcher / packs / kernel / CLI)

| Component | Version source | Relation to host | Notes |
|-----------|----------------|------------------|-------|
| **oclivenewnew (host)** | root `package.json` / `distros/desktop-tauri/Cargo.toml` | — | snapshot **0.5.0** |
| **oclive_kernel_runtime** | `kernel/crates/oclive_kernel_runtime/Cargo.toml` | path dep for host and headless HTTP; `SendMessageResponse.api_version` (`API_VERSION` **u32**, currently **1**), `RUNTIME_API_VERSION` (string **0.2.0**) | OOCP / black-box scripts: `creator-docs/testing/OOCP_TEST_SUITE.md` |
| **oclive-cli** | `kernel/crates/oclive-cli/Cargo.toml` | scaffolds `kernel_server` / `library`; default features **no** desktop `AppState` / SQLite; optional `diagnostics-host` for `--via-host` | [OCLIVE_CLI_GUIDE.md](cli/OCLIVE_CLI_GUIDE.md) |
| **oclive-pack-editor** | sister `package.json` | writes `distros/chat-pro/roles/{id}/`; **`ui.json`** matrix above | `HOST_RUNTIME_VERSION` must match host `version` |
| **oclive-vscode** | sister `package.json` | spawn/attach **`kernel_server --api`**; `distro.oclive.toml` mirrors `examples/distro-profiles/vscode.oclive.toml` | **0.4.1** today; host **0.5.0** recommended |
| **oclive-launcher** | sister `package.json` | sets **`OCLIVE_ROLES_DIR`**, optional model name, zip install; **does not** replace host contract | [launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md) |
| **role packs** | `manifest.json` (`schema_version`, `min_runtime_version`) | older hosts may refuse load or degrade | [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md) |
| **host SQLite** | `kernel/crates/oclive_kernel_host/migrations/*.sql` | ships only with **host** releases; do not downgrade DB after a forward migration unless `CHANGELOG` says so | breaking migrations need bilingual `CHANGELOG` + this table |

On breaking changes: update **`CHANGELOG.md` / `CHANGELOG.en.md`**, this matrix, **`oclive_validation`** (if touched keys), and sister-repo README minimum versions.

### Release review (maintainers)

1. Verify snapshot semver: root **`package.json`**, **`distros/desktop-tauri/Cargo.toml`**, **`oclive_kernel_runtime`**.  
2. Follow [CONTRIBUTING](../CONTRIBUTING.en.md) and [release versioning](development/RELEASE_VERSIONING.md) for external notes when contracts or sister dependencies change.
3. **HTTP / OOCP**: if `API_VERSION` or `RUNTIME_API_VERSION` changes, sync tests and docs (`creator-docs/testing/OOCP_TEST_SUITE.md`).

Headless HTTP authentication is part of the host launch contract: `--api` requires `OCLIVE_API_TOKEN` by default, and callers send `x-oclive-api-token` on every route except `/health`. Never use `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` with production or persistent data.

---

## How to read versions

| Product | Where |
|---------|--------|
| **Host** | in-app About (if present); install name; repo **`package.json`** / **`CHANGELOG.md`** |
| **Editor** | editor About; repo **`package.json`** |

---

## Remote LLM env (pointer)

Env matrix for Remote LLM (`OCLIVE_LLM_BACKEND`, `OCLIVE_REMOTE_LLM_*`, `OCLIVE_LLM_CLOUD_API_STYLE`, OpenAI aliases) lives in the Chinese SSOT [REMOTE_PLUGIN_PROTOCOL.md §2.0](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md); EN protocol page links there.

## Related

- [A5_CLOSURE_SUMMARY.md](../../handoff/A5_CLOSURE_SUMMARY.md)
- [ui.json.schema.json](role-pack/ui.json.schema.json)
- [DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §2.0 — Remote LLM env matrix
- [CHANGELOG.en.md](../../CHANGELOG.en.md)
