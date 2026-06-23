# oclive-pack-editor vs oclivenewnew — version compatibility

How **`ui.json` inside a role pack** relates to the **desktop host**, so you do not hit “the editor exports fields the host ignores” or “the host supports slots the editor cannot edit yet”.

[中文](../creator-docs/COMPATIBILITY.md)

**Version format**: both apps use **SemVer** `MAJOR.MINOR.PATCH` in each repo root **`package.json` `version`**.

**Snapshot when this page was written (align with release review)**:

- **oclivenewnew** (Tauri host): **`0.2.0`** (root `package.json` `version` must match `distros/desktop-tauri/Cargo.toml` `version`)
- **oclive_kernel_runtime** (shared contracts crate): **`0.2.0`** (`kernel/crates/oclive_kernel_runtime/Cargo.toml`; DTO / `API_VERSION` live in that crate)
- **oclive-cli** (scaffold CLI): **`0.1.0`** (`kernel/crates/oclive-cli/Cargo.toml`; **independent SemVer**, not forced to match the desktop host; `--kernel-source` path deps align contracts with this repo)
- **oclive-pack-editor** (sister repo): **`0.2.x`** (that repo’s `package.json`; pair **`ui.json`** with host **0.2.x**)

> Rows marked **0.3.x / 0.4.x** are **planned**; after each release, refresh the snapshot lines and matrix from **`CHANGELOG.md` / `CHANGELOG.en.md`** and **`ui.json.schema.json`**.

---

## Compatibility matrix

| Editor version | Minimum host version | New or hard‑dependent `ui.json` capability | Notes |
|----------------|----------------------|---------------------------------------------|--------|
| **0.2.x** | **0.2.0** | `shell`, `slots` (`chat_toolbar`, `settings_panel`, `role_detail`, …), basic `theme` / `layout` (per schema) | Current mainline; aligned with [role-pack/ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json) |
| **0.3.x** (planned) | **0.3.0** (planned) | If the schema grows **theme/layout** subfields, follow release notes | Older hosts may **silently ignore** unknown JSON keys where models use optional serde fields |
| **0.4.x** (planned) | **0.4.0** (planned) | When **`sidebar`**, **`chat.header`**, etc. are fully authored in the editor, the host **directory bootstrap** must already expose those slots (see [DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)) | Slot names must match host `pluginStore` constants |
| **Dev builds** | **Same dev branch** | Schema + host `UiConfig` on the same branch | For developers only |

---

## Upgrade / downgrade behaviour

1. **Host older than the editor target**  
   - Fields the host **does not model**: usually **ignored** if Rust/TS uses optional serde fields; if a release flips to **reject unknown keys**, follow that release’s `CHANGELOG`.  
   - Slots the host **does not implement yet**: UI may **hide** or **no‑op** until the host is upgraded.

2. **Editor older than the host**  
   - New slots / theme keys may not be editable; you can still **hand‑edit `ui.json`** using [ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json).

3. **Pack `settings.json` / `plugin_backends`**  
   - Tied to **`min_runtime_version`** and `load_role` validation — see [PACK_VERSIONING.md](../creator-docs/role-pack/PACK_VERSIONING.md), [CHANGELOG.md](../CHANGELOG.md).

---

## Cross-app compatibility (host / runtime / CLI / editor / launcher / pack / DB)

| Component | Version source | Relationship to host | Notes |
|-----------|----------------|----------------------|--------|
| **oclivenewnew** (host) | Root `package.json` / `distros/desktop-tauri/Cargo.toml` | — | Snapshot **0.2.0** |
| **oclive_kernel_runtime** | `kernel/crates/oclive_kernel_runtime/Cargo.toml` | Path dep for GUI + headless HTTP; `SendMessageResponse.api_version` (`API_VERSION` **u32**, currently **1**), `RUNTIME_API_VERSION` (**0.2.0** string) | OOCP / black-box expectations: see [`OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md) |
| **oclive-cli** | `kernel/crates/oclive-cli/Cargo.toml` | Generates `kernel_server` / `library` skeletons; **does not ship** desktop `AppState` / SQLite policy | Contract alignment: [`OCLIVE_CLI_GUIDE.md`](../creator-docs/cli/OCLIVE_CLI_GUIDE.md), template `CONFIG_REFERENCE.md` |
| **oclive-pack-editor** | That repo’s `package.json` | Produces `distros/chat-pro/roles/{id}/`; **`ui.json`** vs host: matrix above | `HOST_RUNTIME_VERSION` should track host `version` (editor README) |
| **oclive-launcher** | That repo’s `package.json` | Injects **`OCLIVE_ROLES_DIR`**, optional model / zip install; **does not replace** host contracts | [Launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md) |
| **Role pack** | `manifest.json` (`schema_version`, `min_runtime_version`) | Older hosts may refuse or downgrade | [`PACK_VERSIONING.md`](../creator-docs/role-pack/PACK_VERSIONING.md), `RoleStorage::load_role` |
| **Host SQLite** | `distros/desktop-tauri/migrations/*.sql` | Migrations ship **with the host** only; **do not** assume you can open a DB written by a newer host with an older binary (unless `CHANGELOG` explicitly supports rollback) | Breaking DB steps must be called out in **bilingual CHANGELOG** + this page |

For breaking changes: update **`CHANGELOG.md` / `CHANGELOG.en.md`**, the planned matrix rows above, **`oclive_validation`** (if keys changed), and sister-repo README minimums.

### Release review (maintainer self-check)

1. Verify the three SemVer snapshots: root **`package.json`**, **`distros/desktop-tauri/Cargo.toml`**, **`oclive_kernel_runtime`** (often bumped together).  
2. Open [`PRODUCT_RELEASE_CHECKLIST.md`](../handoff/PRODUCT_RELEASE_CHECKLIST.md) **“对外说明”**: if contracts or sister-repo expectations changed, update this page.  
3. **HTTP / OOCP**: if `API_VERSION` or `RUNTIME_API_VERSION` changes, update the test suite and docs ([`OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md)).

For the Chinese-maintained superset (same facts), see [COMPATIBILITY.md](../creator-docs/COMPATIBILITY.md).

---

## How to read the version

| Product | Where |
|---------|--------|
| **Host** | In‑app **About** (if present); installer name; repo **`package.json`** / **`CHANGELOG.md`** |
| **Editor** | Editor **About**; that repo’s **`package.json`** |

---

## Related docs

- [`A5_CLOSURE_SUMMARY.md`](../handoff/A5_CLOSURE_SUMMARY.md)
- [role-pack/ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json)
- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [CHANGELOG.md](../CHANGELOG.md)
