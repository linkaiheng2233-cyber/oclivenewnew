# Changelog (English)

> **Chinese mirror**: [CHANGELOG.md](CHANGELOG.md) — keep user-facing entries in sync between both files.

## [Unreleased]

**Kernel / CLI / quality (current `main`; app version remains **0.2.0** per `package.json` / `src-tauri`):**

- **Kernel orchestration**: **`process_message`** is the **sole main orchestration entry** for Tauri and the HTTP API; the **entry blueprint (`pipeline.ocblueprint`) is removed from the main path**; sub-flows run in order inside the `chat_engine` module.
- **Monolith**: **`oclive-cli`** four-phase flow (**`init --monolith`** → **`build`** → dual-binary **`bench`**) and **`vendor/oclive_monolith_builtin/`** weld stubs; see [RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) and [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md).
- **CLI**: **`oclive dev`** (hot watch under `roles/`); **`oclive bench --save` / `--compare`** (`bench_history.json`); **`oclive pack validate|create|publish`** (pack validation and `.oclivepack` publish).
- **Startup health**: **`startup_health`** before the first chat turn (slots, pack files, **`DbManager::health_ping`**, optional LLM probe); skip with **`OCLIVE_SKIP_STARTUP_HEALTH`** and related env vars.
- **Errors & observability**: **`thiserror`**-based **`AppError`** with frontend-mappable copy; **`tracing`** + **`RUST_LOG`** (CLI / library **`init_tracing`** defaults to **`info`**).
- **Static analysis**: workspace **`[workspace.lints]`** and **`cargo clippy ... -D warnings`** aligned with CI and local **`check:rust:clippy`** (warnings fail the build).
- **Frontend i18n**: added **`app.documentTitle`**; **`App.vue` / `DirectoryShellApp.vue`** sync **`document.title`** and **`document.documentElement.lang`** with locale; **`index.html`** inline script uses **`oclive.appLocale`** + browser language to reduce tab-title flash before Vue mounts; **directory-shell Vue bootstrap** now calls **`app.use(i18n)`**. New Vitest **`i18n_locale_parity`** asserts **zh-CN / en-US** message key trees match.

### Added

- Plugin manifests can declare subscribed host events (`shell.bridge.events` or `ui_slots[].bridge.events`) to avoid unnecessary broadcasts.
- Settings “General”: **“Force iframe mode”** — when on, all plugin UIs render in iframes for maximum sandbox isolation.
- Dev mode: static security scan (acorn) on Vue slot source; dangerous APIs trigger a warning dialog; user chooses whether to continue.

### Changed

- Role switch: host event broadcast timing adjusted so plugin subscriptions are synced before `role:switched`.
- Directory plugin bootstrap (`get_directory_plugin_bootstrap`) result includes `subscribedHostEvents`.

### Fixed

- Concurrent slot bootstrap could leave inconsistent event subscription sets.
- Custom plugin events were wrongly filtered by `bridge.events`: only built-in host events use subscription filtering; custom events stay broadcastable.

### Performance

- Dedupe concurrent in-flight IPC for `get_directory_plugin_catalog` (global single call).
- Dedupe `get_directory_plugin_bootstrap` IPC per `role_id` to cut duplicate calls when many slots mount.
- Dedupe `get_plugin_state` IPC per `role_id`; clear in-flight keys before `save`/`reset` to reduce stale reads under concurrency.
- Dev Vue slots: reuse scanned source for `vue3-sfc-loader`, avoiding a second `read_plugin_asset_text` per `.vue`.
- Rust: `directory_plugin_bootstrap_dto` merges `subscribed_host_events` in the same pass as `ui_slots`; one `manifest.json` parse per enabled plugin dir (whole-shell URL still parsed separately).
- `pluginStore.refresh()` only replaces state when directory `catalog` / `pluginState` actually change.
- `setHostEventSubscribedEvents` short-circuits when the subscription signature is unchanged.
- `pluginsOrderedForSlot`: slot-level memo; order filter uses `Set` instead of `includes`.
- `pluginStore` precomputes sorted `catalogCandidatesBySlot` on `catalog` change for reuse.
- `pluginStore.refresh()` shares one in-flight Promise; `applyDirectoryBootstrap` writes `ui_slots` to `bootstrapUiSlots` so slots read from store and repeat `get_directory_plugin_bootstrap` calls drop.
- Embedded slots (chat toolbar / settings / role detail) share `useDirectoryPluginSlotEmbed`; `slotOrderMemo.clear()` on directory `catalog` change.

### Engineering

- **Clippy / rustfmt**: full workspace `cargo clippy -- -D warnings` clean; **`cargo fmt --all`** aligned with CI `rustfmt --check`.
- **Shared crate `crates/oclive_validation`**: single source for disk manifest validation (`validate_disk_manifest`, `parse_hhmm`, `KnowledgePackConfigDisk`, …); runtime depends on it; editor can use **wasm** (`--features wasm`, `wasm32-unknown-unknown`) via `validate_manifest_wasm`.
- **Local HTTP API**: binary supports `--api` / `--port` (or `OCLIVE_API_PORT`), default `http://127.0.0.1:8420`, `GET /health`, `POST /chat` (`role_path` + `message`, optional `session_id`) for editor try-chat and tools. `session_id` maps to internal SQLite key `{manifest_role_id}__sess__{sanitized}`; JSON includes `reply`, echoed `session_id`, and top-level **`personality_source`** (`vector`|`profile`, aligned with pack `evolution`). Empty `message` → 400; session key length capped at **256**. **`POST /chat`** uses **`spawn_blocking`** for directory probe and `load_role_from_dir` (same idea as `import_role_pack`).
- **Tauri**: `peek_role_pack` uses **`spawn_blocking`** for long zip/disk reads off the async command thread.
- **Clippy**: `process_co_present` / `process_remote_life` / `detect_movement_intent` explicitly **`allow(too_many_arguments)`** with rationale for `-D warnings` CI.
- **HTTP API tests**: `tests/http_api_chat.rs` with `tower::oneshot` for `GET /health`, `POST /chat` (empty 400; success includes `personality_source` + `reply`); shared **`api_router`** with `serve_api`.
- **Role load**: if `plugin_backends` uses `remote` but `OCLIVE_REMOTE_PLUGIN_URL` / `OCLIVE_REMOTE_LLM_URL` unset, log `oclive_plugin` warning on successful `load_role_from_dir` (still falls back to built-ins per PLUGIN_V1).
- **Pack contract**: optional **`min_runtime_version`** (semver) in `manifest.json`; **`load_role`** rejects when host is too old. Top-level key allowlist (`oclive_validation::json_keys`); unknown top-level keys in `manifest` / `settings` error; **`_`-prefixed note keys** still allowed. **`validate_min_runtime_version`**; wasm **`validate_manifest_wasm`** third arg = host version string.
- **CI**: `oclivenewnew` Rust (fmt / clippy / `cargo test`) + `npm run build` on Ubuntu and Windows; **oclive-pack-editor** and **oclive-launcher** workflows aligned on both OSes.
- **npm**: `npm run check:release` (full `cargo test` gate); README Sentry / offline installer notes.
- **UI**: identity HelpHint distinguishes relationship identity vs core personality archive; copy “人设回复” → “角色回复”.
- **API / UI**: `RoleInfo` / `RoleData` add **`personality_source`** (`vector` | `profile`) aligned with `evolution`; `roleStore` and debug “personality vector” HelpHint under **profile**.
- **Remote plugins**: `EventEstimator::estimate` and `event.estimate` `params` add **`personality_source`**; `prompt.build_prompt` `params` add top-level **`personality_source`** beside full `role`.
- **Main UI**: `RoleRuntimePanel` shows **personality source** (vector / archive) + HelpHint aligned with debug panel.

### Documentation

- **PLUGIN_V1 / Remote**: [PLUGIN_V1.md](creator-docs/plugin-and-architecture/PLUGIN_V1.md) — `RoleInfo` / `RoleData`, HTTP `/chat`, `prompt.build_prompt` **`personality_source`**; [REMOTE_PLUGIN_PROTOCOL.md](creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — new §3.4 and `event.estimate` param row.
- **Personality archive axis**: rewrote **[docs/personality-archive-notes.md](docs/personality-archive-notes.md)**; added **[docs/design-axis-evolution.md](docs/design-axis-evolution.md)**; cross-linked README, `creator-docs`, `roles/README_MANIFEST.md`, pack docs; **`roles/settings.template.json`** `evolution` includes **`personality_source`**.
- **[creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)**: local sidecar + BYOK for proprietary APIs.
- **[examples/remote_plugin_openai_compat/](examples/remote_plugin_openai_compat/)**: OpenAI-compatible `chat/completions` sidecar.
- **[examples/common/](examples/common/)**: shared JSON-RPC / non-LLM stubs for minimal sidecars.
- Roadmap docs: **PLUGIN_WEB_SECTION**, **COMMUNITY_WEB_VISION**, **MARKET_LAUNCHER_INTEGRATION**, **SOMEDAY_TOOLCHAIN_CI**, **BACKLOG_EXPERIENCE_AND_ECOSYSTEM**; **PROJECT_OVERVIEW**; launcher env/troubleshooting cross-links; **GITHUB_REPO_CHECKLIST**; Dependabot / PR templates / `workflow_dispatch` across repos.

---

## [0.2.0] — 2026-04-02

### Added

- Large pack import progress: backend `import_progress` events + frontend progress modal.
- Pre-import preview (`manifest.json` peek) and conflict dialog when role ID exists.
- Import **`.zip`** (same as `.ocpak`) and **extracted folders** (same layout as `roles/{roleId}/`); see `roles/README_MANIFEST.md`.
- Scene welcome: after `switch_scene`, read `scene.json` `welcome_message` (or stable random monologue) into chat as persona message.
- Relation tier upgrade: `send_message` adds `relation_state`; frontend inserts system message on upgrade.

### Changed

- Virtual scroll: `ChatMessageList` always uses virtual scroll when messages exist.
- Export filename default `{role_name}_{version}.ocpak` (sanitized).

### API

- `send_message` adds `relation_state`; `emotion` remains user-input seven-dim analysis.

### Frontend

- Header and chat still use `bot_emotion` for sprite / mood assets / emoji.

### Documentation

- Creator docs under **`creator-docs/`**; legacy `docs/*.md` note in `docs/README.md`; history in **`ARCHIVE_PROJECT_HISTORY.md`**.
- **`roles/README_MANIFEST.md`**: in-app import; **`CREATOR_WORKFLOW.md`**, **`DOCUMENTATION_INDEX.md`**, root **`README.md`** updated.
- **`roles/TESTING_ROLE_PACK_IMPORT.md`**: manual import checklist; zip root `manifest.json` precedence documented in **`role_pack.rs`**.
- See `handoff/20_SESSION_OPTIMIZATION_REPORT.md`.

---

## [0.1.0]

- Initial public baseline (first tagged version in repo).
