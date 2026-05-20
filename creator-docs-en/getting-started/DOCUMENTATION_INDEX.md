# oclive documentation index and reading order

Creator and plugin documentation lives under repo root **`creator-docs/`** (topic subfolders). Pick a path by role.

**If you feel lost**: start with **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** (three repos, commands, checklist). **To align status + goals + doc map by purpose**: **[PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md)**. **For semver, what ships, and changelog entry points only**: **[PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md)**.

---

## Engineering discipline (C2)

| Topic | Document |
|-------|----------|
| **Breaking change process** (definition, six steps, compatibility, PR/migration templates) | **[../../handoff/BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md)** |
| **Critical-path handoff (bus factor)** (`process_message`, `PluginHost`, errors, DB, tests/CI entry map) | **[../../handoff/BUS_FACTOR_NOTES.md](../../handoff/BUS_FACTOR_NOTES.md)** |

---

## Learning paths

Time-boxed “start here → ship”; **authoritative detail stays in each topic page and source**. Cross-check the same rows under **Quick entry**.

| Role | Start here |
|------|------------|
| **End users** (install → import pack → chat; no pack/plugin authoring) | **[USER_MANUAL.md](USER_MANUAL.md)** (Chinese: [../../creator-docs/getting-started/USER_MANUAL.md](../../creator-docs/getting-started/USER_MANUAL.md)) |
| **Role pack authors** | **[../role-pack/CREATOR_LEARNING_PATH.md](../role-pack/CREATOR_LEARNING_PATH.md)** (Chinese: [../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md](../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md)) |
| **Plugin authors** | **[../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)** (Chinese: [../../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](../../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)) |
| **Kernel / hardware integrators** | **[KERNEL_INTEGRATOR_LEARNING_PATH.md](KERNEL_INTEGRATOR_LEARNING_PATH.md)** (Chinese: [../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)) |

---

## Architecture (single-kernel, dual-mode · three module layers)

| Topic | Document |
|-------|----------|
| **Narrative, modules 1–6, facility submodules 1+, backend-module plugin modules (numbering)** | **[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)** (Chinese: [../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)) |
| Kernel factory & three layers | [KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md) |
| Kernel-centric diagram | [KERNEL_AND_MODULES_ARCHITECTURE.md](../../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) |
| Monolith (macro-mode) | [RFC_OCLIVE_MONOLITH_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) |

---

## Contracts quick map

| Topic | Document |
|-------|----------|
| Error codes & triage | **[ERROR_CODES.md](ERROR_CODES.md)** |
| Normative `code` + JSON bodies | **[KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md)** |
| `plugin_backends` + modules 1–6 | **[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)** · **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** |
| Remote HTTP JSON-RPC | **[../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)** |
| On-disk pack / `schema_version` | **[../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)** · **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)** |
| OOCP HTTP black-box (CI) | **[../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md)** |
| Directory whole-shell `invoke` · permissions & errors | **[../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |

---

## Quick entry

| I want to… | Read |
|------------|------|
| **Pick a role: learning paths hub** | **[Learning paths](#learning-paths)** · **[Contracts quick map](#contracts-quick-map)** |
| **End users: install → daily use (user manual)** | **[USER_MANUAL.md](USER_MANUAL.md)** (Chinese: [../../creator-docs/getting-started/USER_MANUAL.md](../../creator-docs/getting-started/USER_MANUAL.md)) |
| **Role pack authors: start → publish (learning path)** | **[../role-pack/CREATOR_LEARNING_PATH.md](../role-pack/CREATOR_LEARNING_PATH.md)** (Chinese: [../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md](../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md)) |
| **Plugin authors: directory / remote / marketplace (learning path)** | **[../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)** (Chinese: [../../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](../../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)) |
| **Directory LLM + llama.cpp (no Ollama, per role pack)** | **[../../examples/directory-plugin-llamacpp/README.en.md](../../examples/directory-plugin-llamacpp/README.en.md)** (Chinese: [../../examples/directory-plugin-llamacpp/README.md](../../examples/directory-plugin-llamacpp/README.md)) |
| **Kernel / hardware integrators: scaffold → device (learning path)** | **[KERNEL_INTEGRATOR_LEARNING_PATH.md](KERNEL_INTEGRATOR_LEARNING_PATH.md)** (Chinese: [../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)) |
| **Current status (version, what ships, changelog entry points)** | **[PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md)** |
| **Align progress and goals (one page: summary + doc map by purpose)** | **[PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md)** |
| **Product launch gates + kernel/platform gaps** | **[../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)** |
| **Four-repo i18n baseline (CJK scan, vue-i18n wiring)** | **[../../handoff/I18N_FOUR_REPO_BASELINE.md](../../handoff/I18N_FOUR_REPO_BASELINE.md)** |
| **Install zip packs via launcher, pick local Ollama, one-click pull** | **[oclive-launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md)** (separate repo) |
| **High-coupling compile mode (Monolith)** | [RFC section](#rfc) (`monolith.toml`, compile-time welding) |
| **Project map / human roles / commands / release checklist** | **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** |
| **Architecture narrative (thin kernel · dual-mode build · traits)** | **[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)** |
| **Kernel-centric diagram (six slots + Agent/MCP/Monolith)** | **[KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md)** |
| **Pure kernel boundary, soul delivery, embedded scope** | **[PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)** · **[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)** (K0–K5) |
| **Single-track platform path (scaffold → deploy)** | **[KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md)** |
| **Headless bring-up (`--api`, K1)** | **[examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md)** |
| **Fast triage after errors / filing issues** | **[ERROR_CODES.md](ERROR_CODES.md)** (tables) · **Normative `code` + JSON: [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md)** · **A3 closure** [EN](../../handoff/A3_CLOSURE_SUMMARY.en.md) / [ZH](../../handoff/A3_CLOSURE_SUMMARY.md) (Sentry / user-visible errors) |
| **GitHub: Dependabot, CI, web settings** | **[GITHUB_REPO_CHECKLIST.md](GITHUB_REPO_CHECKLIST.md)** |
| **Replaceable modules + HTTP sidecar + update strategy** | **[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)** |
| **BYOK sidecar to proprietary cloud models** | **[SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md)** |
| **Sidecar example: OpenAI-compatible API** | **[../../examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)** |
| **Sidecar JSON-RPC shapes** | **[../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)** |
| **`plugin_backends` fields** | **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** |
| **Directory plugin permissions (A4.2)** | **[PLUGIN_V1 §Permission specification](../plugin-and-architecture/PLUGIN_V1.md)** (ZH: [`creator-docs/.../PLUGIN_V1.md`](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)) |
| **Official CLI scaffold `oclive-cli` (registry, compose, publish, TUI, watch, debug)** | **[../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)** |
| **Kernel factory (registry, compose, templates, TUI, bench watch, debug)** | **[KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md)** (Chinese: [../../creator-docs/getting-started/KERNEL_FACTORY_VISION.md](../../creator-docs/getting-started/KERNEL_FACTORY_VISION.md)) |
| **Seven slots + presets, switching to `remote` (authoritative)** | **[../cli/SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)** |
| **Role pack disk layout, multi-distro alignment, `oclive pack validate`** | **[../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)** |
| **Community role pack index JSON** | **[../role-pack/ROLE_PACK_INDEX.md](../role-pack/ROLE_PACK_INDEX.md)** |
| **Directory process plugins** (`plugins/`, manifest, whole-shell, `directory_plugin_invoke`, dev mode; **plugin manager** `Ctrl+Shift+F`, enable/disable/reorder/local zip) | **[../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)** |
| **Bridge `invoke` table, permission aliases, error codes** | **[../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |
| **Config file locations** | **[../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)** |
| **Open-source licensing policy** | **[../LICENSE_POLICY.md](../LICENSE_POLICY.md)** |
| **mumu default slots + plugin FAQ (Vue not showing, iframe debug, deps; user Q&A)** | **[../FAQ.md](../FAQ.md)** |
| **mumu UI release checklist** | **[../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md](../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md)** |
| **Regression: Plugin Manager V2 & Complex Emotion** | **[../guides/REGRESSION_COMPLEX_EMOTION_QA.md](../guides/REGRESSION_COMPLEX_EMOTION_QA.md)** |
| **Editor vs host compatibility (A5 one-pager, EN/ZH)** | **[../COMPATIBILITY.md](../COMPATIBILITY.md)** (Chinese source [../../creator-docs/COMPATIBILITY.md](../../creator-docs/COMPATIBILITY.md)); closure [`../../handoff/A5_CLOSURE_SUMMARY.md`](../../handoff/A5_CLOSURE_SUMMARY.md) |
| **`memory = local`, `_local_plugins`, bridge spec** | **[../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)** |
| **Add a new built-in backend in Rust** | **[../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)** |
| **Author pack content only** | **[CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md)**, [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md), import checklist [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **Core / mutable personality archives, `personality_source`** | **[../../docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)** |
| **Design evolution (seven-dim vs archive axis)** | **[../../docs/design-axis-evolution.md](../../docs/design-axis-evolution.md)** |
| **Editor validation roadmap** | **[../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md](../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md)** |
| **Pack versioning, `schema_version`, `knowledge/`** | **[../../creator-docs/role-pack/PACK_VERSIONING.md](../../creator-docs/role-pack/PACK_VERSIONING.md)** · **[../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md)** |
| **Extension points and source map** | **[../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)** |
| **Vision and roadmap** | **[../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)** |
| **Open lab vision** | **[../../creator-docs/roadmap/VISION_OPEN_LAB.md](../../creator-docs/roadmap/VISION_OPEN_LAB.md)** |
| **Experience backlog** | **[../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** |
| **Someday toolchain / CI** | **[../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)** |
| **Market + launcher integration** | **[../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md](../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md)** |
| **Community web vision** | **[../../creator-docs/roadmap/COMMUNITY_WEB_VISION.md](../../creator-docs/roadmap/COMMUNITY_WEB_VISION.md)** |
| **Plugin web section + `plugins.json`** | **[../../creator-docs/roadmap/PLUGIN_WEB_SECTION.md](../../creator-docs/roadmap/PLUGIN_WEB_SECTION.md)** |
| **OVERVIEW alias** | **[OVERVIEW.md](OVERVIEW.md)** → [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) |
| **Lightweight / `cargo audit` / `cargo-bloat`** | **[../LIGHTWEIGHT_PROFILE.md](../LIGHTWEIGHT_PROFILE.md)** (English mirror of `creator-docs/development/LIGHTWEIGHT_PROFILE.md`) |
| **Performance & known limits (A7)** | **[PERFORMANCE.md](PERFORMANCE.md)** (Chinese: [`creator-docs/getting-started/PERFORMANCE.md`](../../creator-docs/getting-started/PERFORMANCE.md)) |
| **Support entry (A9 · GitHub Issues)** | Root [README.en.md](../../README.en.md) **Support** · [`.github/ISSUE_TEMPLATE`](../../.github/ISSUE_TEMPLATE) |
| **Disclaimer (A10 · models / plugins / data)** | **[../legal/DISCLAIMER.md](../legal/DISCLAIMER.md)** (Chinese: [`creator-docs/legal/DISCLAIMER.md`](../../creator-docs/legal/DISCLAIMER.md)) |
| **Known vulnerabilities (`cargo-audit`)** | **[../security/KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md)** |
| **Security audit scope** | **[../security/SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md)** |
| **Test output schema, OOCP suite, `invoke` hot-path matrix, A1 closure summary, plugin integration tests** | **[../testing/TEST_OUTPUT_SCHEMA.md](../testing/TEST_OUTPUT_SCHEMA.md)** · **[../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md)** · **[../testing/OVERVIEW.md](../testing/OVERVIEW.md)** · **[../../handoff/INVOKE_HOTPATH_MATRIX.md](../../handoff/INVOKE_HOTPATH_MATRIX.md)** · **[../../handoff/A1_CLOSURE_SUMMARY.md](../../handoff/A1_CLOSURE_SUMMARY.md)** · **[../testing/ADAPTING_TEST_PLUGIN.md](../testing/ADAPTING_TEST_PLUGIN.md)** · **[../../creator-docs/testing/L03_GENERATION_CANCEL.md](../../creator-docs/testing/L03_GENERATION_CANCEL.md)** |

---

## RFC

Architecture-level design converges in RFCs (**drafts do not imply merged code or scaffold behavior**).

| Document | Notes |
|----------|--------|
| **[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)** | **Monolith**: `monolith.toml`, `--monolith`, dual `[[bin]]`; **`build` / `bench`** and partial welding (see RFC and CLI guide). |

---

## Suggested reading order (creators / sidecar devs)

1. [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) — pack directories and load paths  
2. [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — six swappable backends + **agent**  
2b. [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) — directory plugins (`directory` enum, `directory_plugins` slots)  
3. [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) — three extension styles, env vars, “hot reload” boundaries  
4. [SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md) — **local sidecar + BYOK**  
5. [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — JSON-RPC methods, params/result, full JSON samples  
6. [examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md) — minimal Python sidecar  
6b. [examples/directory-plugin-minimal/README.md](../../examples/directory-plugin-minimal/README.md) — minimal directory plugin  
6c. [examples/directory-plugin-llamacpp/README.en.md](../../examples/directory-plugin-llamacpp/README.en.md) — directory LLM slot + local llama.cpp HTTP (Chinese: [README.md](../../examples/directory-plugin-llamacpp/README.md))  
7. [examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md) — OpenAI-compatible `chat/completions`  
8. [examples/common/README.md](../../examples/common/README.md) — shared JSON-RPC / non-LLM stubs  

---

## Suggested reading order (host / Rust contributors)

1. [EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)  
2. [HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)  
3. Source: `src-tauri/src/domain/plugin_host.rs`, `src-tauri/src/infrastructure/remote_plugin/`, **`src-tauri/src/infrastructure/directory_plugins/`**  
4. Integration smoke: [`src-tauri/tests/plugin_backends_v2_resolve.rs`](../../src-tauri/tests/plugin_backends_v2_resolve.rs) (`cargo test --test plugin_backends_v2_resolve`)

---

## Relationship to root README

Build commands and tests: root **[README.md](../../README.md)**; **plugin and sidecar details are authoritative in `creator-docs/`** (and mirrored here in English where listed).

---

[中文](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
