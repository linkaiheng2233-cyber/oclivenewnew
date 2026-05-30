# Changelog (English)

> **Chinese mirror**: [CHANGELOG.md](CHANGELOG.md) — keep user-facing entries in sync between both files.

## [Unreleased]

### Changed

- **Dual-core status wording**: runtime dual-core docs now use **Opt-in Beta (default off)**; Stable remains the default delivery path.

### Added

- **Forgetting curve & relation evolution (`config.json`)**: Ebbinghaus long-term memory decay; mention reinforcement (`mention_count` + `reinforcement_factor`); immersive-mode favorability estrangement and relation-stage downgrade; virtual time ratio (`time.speed`) and first-immersion anchor aligned to `life_schedule`; reinforced memories nudge personality / mutable profile “memory shaping”. See [ROLE_PACK_SPEC §9](creator-docs/role-pack/ROLE_PACK_SPEC.md).
- **Dual-core quality hardening**: added optional **S14** OOCP scenario (experimental happy path with valid DAG); CI `oocp-test-suite` now builds with `--features dual_core` and runs `run.mjs --include-dual-core` (covers S13 fallback + S14 happy path); added integration test `src-tauri/tests/dual_core_happy_path.rs` to validate `DualPipelineRunner::run_experimental` success path.

#### Chat Storage (phase 3)

- **Pluggable backends**: `hybrid` (default, SQLite + JSON mirror), `file` (JSON only), `sqlite` (DB only); select via `OCLIVE_CHAT_STORAGE_BACKEND` or role pack `config.json` → `chat_storage.backend`; `oclive-cli init` interactive step added.
- **Memory replay**: `replay_memory_extraction` / `get_replay_progress` — merge re-extract AI memories from chat history (dedupe by keyword similarity; configurable `replay_similarity_threshold`, default 0.6); storage settings UI supports role / scene / session scopes with progress polling.
- **File backend**: `search_messages` (JSON directory scan); `replay_memory_extraction` (chat from files, memories to SQLite); `list_sessions_by_role` for role-scoped replay.
- **Capability detection & UI**: `get_chat_storage_capabilities` exposes `supports_search` / `supports_replay` / `supports_cleanup` / `backend_kind`; storage panel gates actions and shows backend label (i18n).
- **Config**: optional `chat_storage.backend`, `chat_storage.replay_similarity_threshold` (backward compatible).
- **Developers**: `ConversationStore` gains `list_sessions_by_role`, `supports_*`; `replay.rs` role scope uses trait instead of direct DB. See [handoff/CHAT_STORAGE_ARCHITECTURE.md](handoff/CHAT_STORAGE_ARCHITECTURE.md).

---

## [0.2.0] - 2026-05-22

**Desktop host `0.2.0`** · **`oclive-cli` `0.1.0`** · **`oclive_kernel_runtime` `0.2.0`** (independent SemVer; see [RELEASE_VERSIONING.md](creator-docs/development/RELEASE_VERSIONING.md)).

### Breaking

- **Role pack v2:** new packs use **`pipeline.ocblueprint`** (`schema_version: 2`) as the sole config hub; **`oclive pack validate` defaults to v2**. Migration: [V1_TO_V2_MIGRATION.md](creator-docs/role-pack/V1_TO_V2_MIGRATION.md).
- **CLI:** removed top-level `publish`, `plugin search/update`, `registry login` (see [DEPRECATED_COMMANDS.md](crates/oclive-cli/DEPRECATED_COMMANDS.md)).

### Added

- **Blueprint v2 & architecture graph:** `slot_registry`, session `set_session_slot_override`, persist **`save_role_slot_registry`**; golden packs such as **`roles/mumu`** migrated.
- **Dual-core:** `runtime_config.dual_core` + `pipeline.experimental` steps with silent fallback to stable `co_present` (off by default).
- **`oclive-cli` toolchain** (22 top-level subcommands): `init` (incl. **`--monolith`**), `build`, `bench` (`--matrix` / `--cold-start` / `--soak` / `--save`), `dev`, `pack`, `doctor`, `test --oocp`, `explain`, etc.; see [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md).
- **Monolith weld mode:** `init --monolith` → `build` → dual-binary **`bench`**; [RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md).
- **HTTP `--api`:** `GET /health`, `POST /chat`; CI **OOCP black-box S0–S11** + process-restart smoke test.
- **Agent / MCP**, directory-plugin high-risk grants, plugin HTML **`OclivePluginBridge`**, market index install.
- **Startup health** `startup_health`; **`oclive explain`** for all `AppError` codes; **`oclive doctor`** blueprint checks.
- **Orchestration:** `TurnContext`; `AppStateBuilder` + policy registry split; rolling file logs (`OCLIVE_LOG_DIR` / `--api`).

#### Role-pack chat storage location

- **`config.json` → `chat_storage.location`**: new `"role_pack"` / `"global"` (default `"global"`, backward compatible). With `"role_pack"`, chat logs live under the role pack `chats/` subdir; falls back to the global path with a warn if the role pack directory is not writable.
- **Init scaffold:** `oclive-cli init` adds a “Chat storage location” step (follow role pack / global).
- **Storage management panel:** shows the current location badge when a role is selected (follows role pack / global).
- **Export format change:** `export_role_chats` output changed from ZIP+base64 to combined JSON (`application/json`); content unchanged; `zip` dependency removed.

### Changed

- **Main orchestration:** Tauri and HTTP both use **`process_message`**; entry blueprint is **not** the first-turn DSL scheduler.
- **Pack format:** `pack validate` defaults to v2 (`--profile legacy` for old packs); manifest/settings top-level key allowlist tightened.
- **Tauri:** `generate_handler!` grouped by domain with comments; dropped `reqwest` `blocking` and direct `@tauri-apps/api/fs` (custom commands); plugin bridge script as frontend IIFE asset.
- **Architecture graph v2:** removed hand-drag connection composable (edges derived from `slot_registry`).
- **Frontend:** i18n domain split, modular `tauri-api`, Vite vendor chunks; `TopBarMorePanel` extracted from `App.vue`.

### Fixed

- **Errors:** unified **`AppError` / `KernelErrorBody` JSON** + frontend **`apiErrors`** (invoke and HTTP same shape).
- **SQLite:** WAL + pool (`sqlite_pool.rs`); Release profile tuning (`opt-level=3`, `codegen-units=1`).
- **Concurrency:** in-memory **`Cache`** read-first + capacity cap; role cold-load **`DashMap` inflight** (no `Arc::strong_count`).
- Plugin subscription races, custom events wrongly filtered by `bridge.events`, visible warnings when Remote URLs unset, etc.

### Performance

- Release binary sampling ~**12 MiB PE / 7.6 MiB .text** (see [PERFORMANCE.md](creator-docs-en/getting-started/PERFORMANCE.md)).
- Directory-plugin IPC in-flight dedupe (catalog / bootstrap / plugin_state); `pluginStore` refresh and slot memo optimizations.

### Engineering

- Workspace **`cargo clippy -D warnings`** aligned with CI; shared **`oclive_validation`**; **11** `invoke` hot-path integration tests.
- **`npm run check:release`** release gate; Playwright **`vite preview`** smoke (Ubuntu CI).

### Documentation

- [COMPATIBILITY.md](creator-docs-en/COMPATIBILITY.md), [PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md), bilingual **creator-docs-en** mirror and blueprint v2 doc closure.

---

## [0.2.0] — 2026-04-02

(Earlier items in the 0.2.x cycle; summarized in the **0.2.0** release above.)

### Added

- Large pack import progress: backend `import_progress` events + frontend progress modal.
- Pre-import preview (`manifest.json` peek) and conflict dialog when role ID exists.
- Import **`.zip`** (same as `.ocpak`) and **extracted folders**; see `roles/README_MANIFEST.md`.
- Scene welcome after `switch_scene`; relation tier upgrade via `relation_state` on `send_message`.

### Changed

- Virtual scroll always on when messages exist; export filename default `{role_name}_{version}.ocpak`.

### API

- `send_message` adds `relation_state`; `emotion` remains user-input analysis.

---

## [0.1.0]

- Initial public baseline (first tagged version in repo).
