# oclive documentation index and reading order

Creator and plugin documentation lives under repo root **`creator-docs/`** (topic subfolders). Pick a path by role.

**If you feel lost**: start with **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** (three repos, commands, checklist).

---

## Quick entry

| I want to… | Read |
|------------|------|
| **Install zip packs via launcher, pick local Ollama, one-click pull** | **[oclive-launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md)** (separate repo) |
| **High-coupling compile mode (Monolith)** | [RFC section](#rfc) (`monolith.toml`, compile-time welding) |
| **Project map / human roles / commands / release checklist** | **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** |
| **Kernel-centric diagram (six slots + Agent/MCP/Monolith)** | **[KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md)** |
| **Pure kernel boundary, soul delivery, embedded scope** | **[PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)** · **[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)** (K0–K5) |
| **Headless bring-up (`--api`, K1)** | **[examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md)** |
| **Fast triage after errors / filing issues** | **[ERROR_CODES.md](ERROR_CODES.md)** |
| **GitHub: Dependabot, CI, web settings** | **[../../creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](../../creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)** |
| **Replaceable modules + HTTP sidecar + update strategy** | **[../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)** |
| **BYOK sidecar to proprietary cloud models** | **[../../creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md](../../creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)** |
| **Sidecar example: OpenAI-compatible API** | **[../../examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)** |
| **Sidecar JSON-RPC shapes** | **[../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)** |
| **`plugin_backends` fields** | **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** |
| **Official CLI scaffold `oclive-cli`** | **[../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)** |
| **Seven slots + presets, switching to `remote` (authoritative)** | **[../cli/SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)** |
| **Role pack disk layout, multi-distro alignment, `oclive pack validate`** | **[../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)** |
| **Community role pack index JSON** | **[../role-pack/ROLE_PACK_INDEX.md](../role-pack/ROLE_PACK_INDEX.md)** |
| **Directory process plugins** | **[../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)** |
| **Bridge `invoke` table, permission aliases, error codes** | **[../../creator-docs/plugin-and-architecture/BRIDGE_API_REFERENCE.md](../../creator-docs/plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |
| **Config file locations** | **[../../creator-docs/guides/CONFIGURATION_FILES.md](../../creator-docs/guides/CONFIGURATION_FILES.md)** |
| **Open-source licensing policy** | **[../../creator-docs/LICENSE_POLICY.md](../../creator-docs/LICENSE_POLICY.md)** |
| **Manage plugins (enable/disable/reorder/local zip)** | **[../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)** (`Ctrl+Shift+F`) |
| **mumu default front-end slots** | **[../../creator-docs/FAQ.md](../../creator-docs/FAQ.md)** |
| **mumu UI release checklist** | **[../../creator-docs/guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md](../../creator-docs/guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md)** |
| **Plugin FAQ** | **[../../creator-docs/FAQ.md](../../creator-docs/FAQ.md)** |
| **Editor vs host compatibility** | **[../../creator-docs/COMPATIBILITY.md](../../creator-docs/COMPATIBILITY.md)** |
| **`memory = local`, `_local_plugins`, bridge spec** | **[../../creator-docs/plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../../creator-docs/plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)** |
| **Add a new built-in backend in Rust** | **[../../creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../../creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)** |
| **Author pack content only** | **[../../creator-docs/getting-started/CREATOR_WORKFLOW.md](../../creator-docs/getting-started/CREATOR_WORKFLOW.md)**, [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md), import checklist [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **Core / mutable personality archives, `personality_source`** | **[../../docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)** |
| **Design evolution (seven-dim vs archive axis)** | **[../../docs/design-axis-evolution.md](../../docs/design-axis-evolution.md)** |
| **Editor validation roadmap** | **[../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md](../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md)** |
| **Pack versioning, `schema_version`, `knowledge/`** | **[../../creator-docs/role-pack/PACK_VERSIONING.md](../../creator-docs/role-pack/PACK_VERSIONING.md)** · **[../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md)** |
| **Extension points and source map** | **[../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md](../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)** |
| **Vision and roadmap** | **[../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)** |
| **Open lab vision** | **[../../creator-docs/roadmap/VISION_OPEN_LAB.md](../../creator-docs/roadmap/VISION_OPEN_LAB.md)** |
| **Experience backlog** | **[../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** |
| **Someday toolchain / CI** | **[../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)** |
| **Market + launcher integration** | **[../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md](../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md)** |
| **Community web vision** | **[../../creator-docs/roadmap/COMMUNITY_WEB_VISION.md](../../creator-docs/roadmap/COMMUNITY_WEB_VISION.md)** |
| **Plugin web section + `plugins.json`** | **[../../creator-docs/roadmap/PLUGIN_WEB_SECTION.md](../../creator-docs/roadmap/PLUGIN_WEB_SECTION.md)** |
| **OVERVIEW alias** | **[../../creator-docs/getting-started/OVERVIEW.md](../../creator-docs/getting-started/OVERVIEW.md)** → [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) |
| **Lightweight / `cargo audit` / `cargo-bloat`** | **[../LIGHTWEIGHT_PROFILE.md](../LIGHTWEIGHT_PROFILE.md)** (English mirror of `creator-docs/development/LIGHTWEIGHT_PROFILE.md`) |
| **Known vulnerabilities (`cargo-audit`)** | **[../security/KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md)** |
| **Security audit scope** | **[../security/SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md)** |
| **Test output schema, OOCP suite, plugin integration tests** | **[../testing/TEST_OUTPUT_SCHEMA.md](../testing/TEST_OUTPUT_SCHEMA.md)** · **[../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md)** · **[../testing/OVERVIEW.md](../testing/OVERVIEW.md)** · **[../testing/ADAPTING_TEST_PLUGIN.md](../testing/ADAPTING_TEST_PLUGIN.md)** · **[../../creator-docs/testing/L03_GENERATION_CANCEL.md](../../creator-docs/testing/L03_GENERATION_CANCEL.md)** |

---

## RFC

Architecture-level design converges in RFCs (**drafts do not imply merged code or scaffold behavior**).

| Document | Notes |
|----------|--------|
| **[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)** | **Monolith**: `monolith.toml`, `--monolith`, dual `[[bin]]`; **`build` / `bench`** and partial welding (see RFC and CLI guide). |

---

## Suggested reading order (creators / sidecar devs)

1. [CREATOR_WORKFLOW.md](../../creator-docs/getting-started/CREATOR_WORKFLOW.md) — pack directories and load paths  
2. [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — six swappable backends + **agent**  
2b. [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) — directory plugins (`directory` enum, `directory_plugins` slots)  
3. [CREATOR_PLUGIN_ARCHITECTURE.md](../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) — three extension styles, env vars, “hot reload” boundaries  
4. [SIDECAR_LLM_USER_GUIDE.md](../../creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md) — **local sidecar + BYOK**  
5. [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — JSON-RPC methods, params/result, full JSON samples  
6. [examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md) — minimal Python sidecar  
6b. [examples/directory-plugin-minimal/README.md](../../examples/directory-plugin-minimal/README.md) — minimal directory plugin  
7. [examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md) — OpenAI-compatible `chat/completions`  
8. [examples/common/README.md](../../examples/common/README.md) — shared JSON-RPC / non-LLM stubs  

---

## Suggested reading order (host / Rust contributors)

1. [EXTENSION_POINTS.md](../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)  
2. [HOW_TO_REPLACE_MODULES.md](../../creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)  
3. Source: `src-tauri/src/domain/plugin_host.rs`, `src-tauri/src/infrastructure/remote_plugin/`, **`src-tauri/src/infrastructure/directory_plugins/`**  
4. Integration smoke: [`src-tauri/tests/plugin_backends_v2_resolve.rs`](../../src-tauri/tests/plugin_backends_v2_resolve.rs) (`cargo test --test plugin_backends_v2_resolve`)

---

## Relationship to root README

Build commands and tests: root **[README.md](../../README.md)**; **plugin and sidecar details are authoritative in `creator-docs/`** (and mirrored here in English where listed).

---

[中文](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
