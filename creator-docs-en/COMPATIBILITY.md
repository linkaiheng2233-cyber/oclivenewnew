# oclive-pack-editor vs oclivenewnew — version compatibility

How **`ui.json` inside a role pack** relates to the **desktop host**, so you do not hit “the editor exports fields the host ignores” or “the host supports slots the editor cannot edit yet”.

[中文](../creator-docs/COMPATIBILITY.md)

**Version format**: both apps use **SemVer** `MAJOR.MINOR.PATCH` in each repo root **`package.json` `version`**.

**Snapshot when this page was written**:

- **oclivenewnew** (host): `0.2.x` (see root `package.json` / `CHANGELOG.md`)
- **oclive-pack-editor** (authoring tool): `0.2.x` (see that repo’s `package.json`)

> Rows marked **0.3.x / 0.4.x** are **planned**; after release, update this table from each `CHANGELOG.md` and `ui.json.schema.json`.

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

## Cross-app compatibility (host / editor / launcher / pack)

| Component | Version source | Relationship to host | Notes |
|-----------|----------------|----------------------|--------|
| **oclivenewnew** (host) | Root `package.json` / `src-tauri/Cargo.toml` | — | Snapshot **0.2.x** |
| **oclive-pack-editor** | That repo’s `package.json` | Produces `roles/{id}/`; **`ui.json`** vs host: matrix above | `HOST_RUNTIME_VERSION` should track host `version` (editor README) |
| **oclive-launcher** | That repo’s `package.json` | Injects **`OCLIVE_ROLES_DIR`**, optional model / zip install; **does not replace** host contracts | [Launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md) |
| **Role pack** | `manifest.json` (`schema_version`, `min_runtime_version`) | Older hosts may refuse or downgrade | [PACK_VERSIONING.md](../creator-docs/role-pack/PACK_VERSIONING.md), `RoleStorage::load_role` |

For the Chinese-maintained superset (same facts), see [COMPATIBILITY.md](../creator-docs/COMPATIBILITY.md).

---

## How to read the version

| Product | Where |
|---------|--------|
| **Host** | In‑app **About** (if present); installer name; repo **`package.json`** / **`CHANGELOG.md`** |
| **Editor** | Editor **About**; that repo’s **`package.json`** |

---

## Related docs

- [role-pack/ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json)
- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [CHANGELOG.md](../CHANGELOG.md)
