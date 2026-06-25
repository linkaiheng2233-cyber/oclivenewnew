# Changelog (English)

> **Chinese mirror**: [CHANGELOG.md](CHANGELOG.md) — keep user-facing entries in sync between both files.

## [Unreleased]

### Added

- **Chat Pro stream cancel**: new sends abort the in-flight SSE via `AbortController` and remove dangling `streaming` bubbles.
- **Chat Pro stream toggle**: Settings → General → Advanced “Streaming replies” (`localStorage` `oclive.chat.streamEnabled`, default on).
- **Monorepo layout (kernel / distros)**: Rust crates under `kernel/`; desktop distros under `distros/{shared,chat-pro,theater,desktop-tauri}`; RFC: [`handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md`](handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md).
- **Theater Track A engineering hygiene (round 16)**: [`handoff/theater/MODE2_UNFREEZE.md`](handoff/theater/MODE2_UNFREEZE.md) Mode 2 unfreeze checklist; `theater-prompt-drift` wired into `dimension5-acceptance.mjs` and `test:theater:smoke`; self-contained `prompts/` in minimal director example; `data/plugins.json` entry for `com.oclive.theater_director_official`.
- **`CODE_OF_CONDUCT.md`** (Contributor Covenant).
- **`human-docs-en/`** minimal set (L0–L3 + 08/09/10 English summaries).
- **`human-docs/08_PR_GATE_MATRIX.md`**, **`09_GLOSSARY.md`**, **`10_SETUP_WINDOWS.md`**.
- **`handoff/GOOD_FIRST_ISSUES.md`** curated issue table.
- **`npm run check:ci-local`**; `package.json` `engines.node >=20`, **`.nvmrc`**.
- Frontend: `src/api/plugin/*`, `useMainShell*`, `useChatStorageSettings`, `chatStoreSend`.

### Changed

- **Repository layout**: root `crates/`, `src-tauri/`, and `src/` moved to `kernel/crates/`, `distros/desktop-tauri/`, and `distros/{shared,chat-pro,theater}/`; root `npm run tauri:dev` / `tauri:dev:theater` unchanged.
- **Theater doc SSOT sweep**: `theater_director` unified as **shipped (2026-06)** (DISTRO_DEFAULT_PLUGINS · ARCHITECTURE · NAMING · ROADMAP §7 · IA); [`TECHNICAL_DEBT_INVENTORY.md`](handoff/TECHNICAL_DEBT_INVENTORY.md) round 16; acceptance chain → [`PLAYTEST_MATRIX.md`](handoff/theater/PLAYTEST_MATRIX.md).
- **Hybrid chat mirror**: `rebuild_mirror_best_effort` / `delete_mirror_best_effort` (K-ROBUST-01).
- **`canonical_llm_sync` / `plugin_state` / MCP·Ollama downgrade**: `tracing::warn!` (K-ROBUST-02/03).
- **Kernel snapshot & storage capability degraded UI** (`kernel.ts`, `useKernelStatus`, `ChatStorageSettingsPanel`).
- **`process_message` readability closure**: `preflight_turn` / `PostLlmCtx` / `PreLlmOutput` grouping; `events.rs` / `blueprint_v2_slot_registry.rs` module extraction; `SettingsView` tab subcomponents; `role_runtime` submodules; `blueprint_v2` tests moved to `tests/`.
- **handoff layout**: `THEATER_*` → `handoff/theater/`, `VSCODE_*` → `handoff/vscode/`; added `handoff/launcher/`, `handoff/pack-editor/`, `handoff/studio/` distro doc indexes; fixed broken links after theater greenfield reset.
- **README / CONTRIBUTING / SECURITY** community infra updates; PR template links to PR gate matrix; optional `scripts/setup-dev.ps1`.
- **Five-dimension review closure (Batch 1–3)**: architecture overview co-presence chain aligned with Stable code; VS Code / cross-host docs policy-first; `user_identities` validation matches `load_role`; `reply_post_processor` requires non-empty `plugin_id` when `enabled` + `directory`; `ProcessMessageError` stage preserved on outward `AppError` messages; chat turn `role_runtime` preflight merge, single CASE `memory decay` UPDATE, `SessionCache` skips repeat interaction_mode seed; workspace `default-members` excludes `fuzz`; `chatStore` load / `addMessage` micro-optimizations.
- **`chat_storage` pack validation**: `oclive pack validate` now checks `config.json` → `chat_storage` (backend / location / positive integers / replay threshold 0.1–1.0), parity with `reply_post_processor`; `CHANGELOG.md` parity synced.
- **Prompt guardrails elevation & footer dedup**: `KERNEL_DIALOGUE_GUARDRAILS` always includes state continuation, vent-first, and length-by-input (pack `reply_quality_anchor` cannot override); removed standalone `【回复结构】` block; tone block no longer exposes `warmup_level` / normalized impact-factor jargon; official mumu/shimeng/枫侵月 anchors slimmed to persona-only deltas.
- **License change**: host relicensed from AGPL-3.0 + plugin exception to **Apache-2.0** (root `LICENSE` + `NOTICE`); enables closed-source commercial distros and embedded downstreams to combine the kernel freely; `LICENSE_POLICY.md` updated.
- **Official distro · Daily chat / Story mode split**: `distro.oclive.toml` adds `[interaction]`; new `desktop-chat` profile; `desktop` / `vscode` default `pure_chat`; first-run seed order is distro → role pack → `pure_chat`.
- **Pure-chat UI slimming & settings tiers**: Daily chat hides scene/time/plugin sidebar and numeric favor; Settings split into Essentials / More options; Story-mode hint after N turns.
- **User-identity surprise unlock**: identity picker hidden on first screen; after 5 turns or keyword match, show “You could also be…” identity sheet.
- **Interaction mode default & dedup**: first run always Daily chat; user choice persists in `role_runtime`; mode switch unified in `InteractionModeBar` above chat input; removed duplicate locale/appearance/plugin tiles from top-bar More.
- **Pure-chat hides plugins**: Daily chat hides plugin slots, market, Ctrl+Shift+F, and Settings Plugins tab; Story mode restores full plugin UI.
- **Product narrative alignment**: README / AGENTS / positioning docs unified as "AI role assembly platform"; frozen items (dual_core, blueprint v3, expert_routing) phrased as "mechanism pre-wired, off by default"; chat storage clarifies hybrid as the production path.
- **Profile scheduling UX**: desktop status bar, Settings → Kernel & Connection, and the VS Code status bar share unified profile-adaptation wording (attach / mismatch / pin / replace / degraded).
- **Distro profile parse SSOT**: `distro.oclive.toml` parsed once via `oclive_kernel_runtime::distro_oclive_file` (K-PROFILE-01).
- **Host domain re-exports**: runtime engine modules deprecated at `oclive_kernel_host::domain`; `check-host-reexport-imports.mjs` ratchet (D-OPUS-05).
- **`resolve_*` naming adjudication (D-NAME-01)**: 35 non-policy functions renamed to `load_*` / `find_*` / `pick_*` / `build_*` / `merge_*` / `compute_*` / `invoke_*`; 22 cross-host / per-turn policy anchors kept; verb table in `NAMING_CONVENTIONS.md` §4.4.

### Added

- **Theater release packaging**: `npm run tauri:build:theater` · `OCLIVE_TAURI_SHELL=theater` · roles subset (`theater-breakfast-a/b`) via [`scripts/filter-theater-roles.mjs`](scripts/filter-theater-roles.mjs) into `src-tauri/resources/roles/`.
- **Theater 15s engineering proxy**: [`scripts/theater-stranger-proxy.mjs`](scripts/theater-stranger-proxy.mjs) · aggregated in `npm run test:theater:smoke` (CI `frontend` job).
- **Theater vision & roadmap SSOT**: [`handoff/theater/DEVELOPMENT_ROADMAP.md`](handoff/theater/DEVELOPMENT_ROADMAP.md) (Mode 1 greenfield; legacy `THEATER_*` docs removed).
- **Three-distro kernel smoke (Pro / Flash)**: `npm run test:distro:smoke` aggregates profile mirror · distro kernel e2e · Tauri bundled-first; `e2e-distro-kernel` adds a **theater** scenario; CI **`cross-host-e2e`** adds `e2e-tauri-bundled-kernel` and VS Code profile diff. Closure SSOT: [`handoff/THREE_DISTRO_KERNEL_CLOSURE.md`](handoff/THREE_DISTRO_KERNEL_CLOSURE.md).
- **Chat Pro bundled-first spawn (K-SCHED-05/01)**: Tauri `bundle-kernel-for-tauri.mjs` · `pick_best_for_spawn` bundled → shared → dev; `binary_upgrade` replace off by default.
- **VS Code Flash profile mirror**: `examples/distro-profiles/vscode.oclive.toml` ↔ sibling `distro.oclive.toml` · `npm run test:distro-profile-mirror`.
- **Contract extension envelope (V-CONTRACT Phase 0)**: `SlotExtension { schema_id, data }`; optional `extension` on `EmotionResult` / `ComplexEmotionOutput`; `PromptInput.extra_sections` injects generic blocks before the quality anchor; evolution rules in `EXTENSION_POINTS.md`.
- **Hot-path stage tracing (K-PERF-02)**: `oclive_turn` target logs per-`ChatStage` `elapsed_ms`; sample table in `creator-docs/getting-started/PERFORMANCE.md` §6.
- **CHANGELOG CI gate (K-DOC-02)**: `scripts/check-changelog-parity.mjs` wired into `dimension5-acceptance.mjs`.
- **AI Theater v0 (`theater` distro)**: `examples/distro-profiles/theater.oclive.toml`; `TheaterShell` first screen (hides six-slot/blueprint UI); breakfast scene + dual contrast role packs + pre-generated `skeleton.json`; 3 poke chips + local Ollama beat patch (graceful fallback).
- **Product freeze**: no kernel expansion until Theater v0 stranger validation — see [`handoff/theater/DEVELOPMENT_ROADMAP.md`](handoff/theater/DEVELOPMENT_ROADMAP.md) §4.8.
- **Creator golden path**: `creator-docs/getting-started/CREATOR_GOLDEN_PATH.md` (separate from kernel docs).

### Performance

- **Batched memory-decay writes (K-PERF-01/06)**: `DbManager::persist_memory_decay_batch` changed from one independent `UPDATE` per memory to a single batched transaction; after rank, one call per turn merges decay write-back and `accessed_at` touch. See `long_term_memory.rs` and `turn_pipeline/pre.rs`.
- **Lazy shell loading (K-PERF-09)**: `App.vue` now imports `FluentShell` / `ToolShell` via `defineAsyncComponent`, loading only the shell selected by `resolveOcliveShell()`; the non-rendered shell no longer ships in the first-screen main chunk.
- **Hot-path DB merge (K-PERF-03~06)**: one `EffectiveSessionConfig` per turn; single `get_role_runtime_snapshot` read; shared `TurnPrefetch` / skip agent DB when `agent=none`; one memory-decay transaction. Baseline: `handoff/OPUS_48_PERF_BASELINE.md`.
- **Long-lived memory/SQLite (K-PERF-07/08/12)**: `SessionCache` six-map cap+TTL; `personality_vector` composite index migration `033`; `hybrid_store` drops redundant `get_chat_session`; `role_cache` LRU(32); LLM startup probe runs in background.
- **In-shell lazy panels + poll backoff (K-PERF-10/11)**: non-first-screen panels in `FluentShell`/`ToolShell` via `defineAsyncComponent`; `useKernelStatus` backs off to 60s when the tab is hidden.
- **RoleRuntimeSnapshot downstream reuse (K-PERF-20)**: `relation_snapshot` / `post` / `pre` Profile paths share one snapshot; at most one `get_current_emotion` per turn (except post-write refresh).
- **Ollama model settings batch read (K-PERF-21)**: `resolve_effective_ollama_model` uses a single `get_app_settings([provider, remote_model])` round trip.
- **Chat session list + upsert (K-PERF-22)**: session list snippet via window-function JOIN; `upsert_chat_session` `RETURNING` removes post-write re-read.
- **Long-term memory & operation-log indexes (K-PERF-23)**: migration `034_perf_indexes.sql` (`idx_ltm_role_content` / `idx_operation_logs_role`).
- **Fewer post-phase Role clones (K-PERF-24)**: `TurnContext.role_arc` reused for profile evolution spawn.
- **`pre_llm` Wave 1 parallelism (K-PERF-14)**: `turn_pipeline/pre.rs` uses `tokio::try_join!` for five read-only paths (context / emotion / model / narrative hint / memories); `oclive_turn` logs `pre_llm_wave1` aggregate; see `PERFORMANCE.md` §6.

---

## [0.3.0] - 2026-06-07

**Desktop host `0.3.0`** · **VS Code extension `0.3.0`** · **`SendMessageResponse.schema` 14**

### Breaking

- **`SendMessageResponse.schema`** bumped to **14**: optional `raw_reply` when `include_raw_reply: true` and post-processor changed the LLM text.
- **`high_risk_grants.json`**: only canonical permission keys (`mcp:http`, `mcp:stdio`, `process:spawn`, `network:*`); legacy aliases no longer read — migrate files and re-grant.

### Changed

- **Experimental dual-core runtime**: `oclivenewnew-tauri` Cargo feature **`dual_core`** (default off); build with `cargo build -p oclivenewnew-tauri --features dual_core`.
- **Dual-core status wording**: runtime dual-core docs now use **Opt-in Beta (default off)**; Stable remains the default delivery path.

### Added

- **User Identity & Reply Post-Processor Phase 2 (closure)**: HostProfile merge; remote/directory backends; HTTP `/user_identity/*`; desktop & VS Code identity switch; `RoleInfo` / `GET /role_info` read-only post-processor fields; debug panel status line. See [handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md](handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md).
- **Docs**: ROLE_PACK_SPEC §1.1 / §9.7, architecture overview orthogonal units, USER_MANUAL §3.4–3.5, RFC Phase 2 acceptance checked.
- **Forgetting curve & relation evolution (`config.json`)**: Ebbinghaus long-term memory decay; mention reinforcement; immersive favorability estrangement; virtual time ratio; memory-shaped personality nudges. See [ROLE_PACK_SPEC §9](creator-docs/role-pack/ROLE_PACK_SPEC.md).
- **Dual-core quality hardening**: optional **S14** OOCP scenario; CI dual-core build + `--include-dual-core`; `dual_core_happy_path.rs` integration test.

#### Chat Storage (phase 3)

- **Pluggable backends**: `hybrid`, `file`, `sqlite`; env or `config.json` → `chat_storage.backend`.
- **Memory replay**: `replay_memory_extraction` / `get_replay_progress` with configurable similarity threshold.
- **File backend**: search, replay, `list_sessions_by_role`.
- **Capability detection & UI**: `get_chat_storage_capabilities`; gated storage panel.
- **Config**: optional `chat_storage.backend`, `replay_similarity_threshold`.
- **Developers**: extended `ConversationStore` trait. See [handoff/CHAT_STORAGE_ARCHITECTURE.md](handoff/CHAT_STORAGE_ARCHITECTURE.md).

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

- [COMPATIBILITY.md](creator-docs-en/COMPATIBILITY.md), [PRODUCT_RELEASE_CHECKLIST.md](handoff/archive/PRODUCT_RELEASE_CHECKLIST.md), bilingual **creator-docs-en** mirror and blueprint v2 doc closure.

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
