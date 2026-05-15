# oclive (oclivenewnew)

[中文](README.md)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)

A **local-first** desktop companion for roleplay dialogue: **Tauri + Vue 3 + Rust**. The engine supports scenes, virtual time, co-presence / remote presence, favorability and memory, and swappable subsystems (memory retrieval, emotion, event estimation, prompt assembly). Role content ships as **`roles/{roleId}/`** packs.

## Project status (summary)

| Area | Status |
|------|--------|
| **Kernel orchestration** | Main flow in **`process_message`** under **`src-tauri/src/domain/chat_engine/mod.rs`**; no entry blueprint DSL on the hot path; subsystems resolved via **`PluginHost`** (including **`agent`**). |
| **Testing (three layers)** | **Protocol (this repo):** `cargo test` in `src-tauri` + `tests/`; **OOCP HTTP black-box S0–S11** in [`examples/oocp-test-suite/`](examples/oocp-test-suite/) with CI job **`oocp-test-suite`** (Ubuntu), followed by **`scripts/e2e-core-api-restart.mjs`** (HTTP API process restart smoke). **Components (editor):** **oclive-pack-editor** Vitest / Playwright. **Plugins (editor):** directory-plugin patterns / **`official-vue-test-runner`** live in **oclive-pack-editor**. **Frontend smoke:** CI **`npm ci` + `npm run test:unit` + `npm run build`**. See [creator-docs/testing/OVERVIEW.md](creator-docs/testing/OVERVIEW.md) and [creator-docs/testing/OOCP_TEST_SUITE.md](creator-docs/testing/OOCP_TEST_SUITE.md). |
| **oclive-cli** | Workspace crate **`oclive-cli`**: **`oclive dev`** watches `roles/` for `manifest.json` / `settings.json`; **`oclive bench`** supports **`--save`** / **`--compare`**; **`oclive pack`** **`validate` / `create` / `publish`**; **Monolith** flow in [creator-docs/cli/OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md). |
| **Startup health** | One-shot checks before the first **`process_message`** (slots, pack files, SQLite **`health_ping`**, optional LLM probe). Skip with **`OCLIVE_SKIP_STARTUP_HEALTH`** / **`OCLIVE_SKIP_LLM_STARTUP_PROBE`**. See `src-tauri/src/domain/startup_health.rs`. |
| **Monolith** | Compile-time welded slots for headless scaffolds; RFC + CLI in [creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md). |
| **Security** | **`cargo audit` (0.22.1)** run; **known issues tracked** (currently **5** error-level advisories on the lockfile). See [creator-docs/security/KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md) and [creator-docs/security/SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md). |
| **CI gates** | **`rustfmt` + `clippy` (`-D warnings`) + `cargo test`** (`src-tauri`) + **`npm ci` / `npm run test:unit` / `npm run build`**; **`oocp-test-suite`** (OOCP S0–S11 + core HTTP restart smoke), **`cargo-audit`** (allow-fail), **remote-plugin-demo**. |
| **Lightweight baseline** | [creator-docs/development/LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md). |

Contributor notes: **[AGENTS.md](AGENTS.md)**.

## Performance

Release **`cargo-bloat` sampling**, **Monolith** vs **`oclive bench`**, and **known product limits**: **[creator-docs-en/getting-started/PERFORMANCE.md](creator-docs-en/getting-started/PERFORMANCE.md)** (Chinese: [creator-docs/getting-started/PERFORMANCE.md](creator-docs/getting-started/PERFORMANCE.md)). Figures track [LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.7.

## Support

- **Single entry point:** [**GitHub Issues**](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues) (this repository).  
- **Suggested titles:** `[bug]: …` · `[feat]: …` · `[support]: …` (matches issue templates).  
- **First response:** we usually triage within **3–5 business days** (best-effort volunteer window, **not an SLA**).  
- **Attach environment context:** **OS**; **app version** (e.g. `package.json` / `src-tauri/Cargo.toml` `version`); **`oclive-cli` version** (`crates/oclive-cli/Cargo.toml` or `cargo run -p oclive-cli -- --help`); plus a short summary from **Settings → General → Environment check**. **Do not** paste API keys, tokens, or full private paths.

**Self-serve:** [FAQ](creator-docs/FAQ.md) · [Documentation index](creator-docs/getting-started/DOCUMENTATION_INDEX.md) · [ERROR_CODES](creator-docs/getting-started/ERROR_CODES.md). For bugs, include **error code** and **minimal repro** when possible.

## Early adopters & known limits

- **0.2.x** desktop host focus; **no in-app updater** wired yet — ship **offline installers** (see **Observability & release** if present below).  
- **Ollama** is the default local LLM path; missing daemon or models will fail chat — see [CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md) and [ERROR_CODES.md](creator-docs/getting-started/ERROR_CODES.md) (§1.5 first-install subset).  
- **Remote / directory plugins / MCP** may require outbound network or subprocesses per manifest + host prompts — [DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md).  
- **Product P0 gates** are tracked in [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md) and [handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md).

## Observability (Sentry)

- **Opt-in at build time:** set **`VITE_SENTRY_DSN`** during the frontend build to ship a DSN; otherwise nothing is sent.
- **Vue only:** uncaught Vue errors may be reported; **Rust errors stay local** by default.
- **Privacy defaults:** `sendDefaultPii: false`, query strings stripped from captured request URLs.
- **User opt-out:** when a DSN is present, **Settings → General** offers **Disable crash reporting**; preference is stored under **`localStorage`** key **`oclive.telemetry.sentryOptOut`** (`1` = opted out). Uncheck and **restart the app** to resume reporting.

## Models, plugins, and data (three quick questions)

1. **Third-party models / APIs:** default **local Ollama**; cloud or sidecars are **user-configured** — [SIDECAR_LLM_USER_GUIDE.md](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md), [LICENSE_POLICY.md](creator-docs/LICENSE_POLICY.md).  
2. **Plugins:** follow **manifest permissions** and host grants; AGPL + plugin exception in [LICENSE](LICENSE).  
3. **Data on disk:** SQLite + `{app_data}` — [CONFIGURATION_FILES.md](creator-docs/guides/CONFIGURATION_FILES.md); do not paste private paths in public issues.

## English documentation hub

- **[creator-docs-en/README.md](creator-docs-en/README.md)** — index of English mirrors for core docs.

## Vision (open lab)

Local-first, swappable subsystems, role packs as the contract surface — see [creator-docs/roadmap/VISION_OPEN_LAB.md](creator-docs/roadmap/VISION_OPEN_LAB.md) and [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md).

## Documentation (creators & extensions)

Chinese hub: **[creator-docs/README.md](creator-docs/README.md)** · index **[DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md)**.

| Topic | Path |
|------|------|
| **Project overview (EN)** | [creator-docs-en/getting-started/PROJECT_OVERVIEW.md](creator-docs-en/getting-started/PROJECT_OVERVIEW.md) |
| **Kernel & modules diagram (EN)** | [creator-docs-en/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](creator-docs-en/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) |
| **Plugin contract summary (EN)** | [creator-docs-en/plugin-and-architecture/PLUGIN_V1.md](creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) |
| Plugin contract (full, ZH) | [creator-docs/plugin-and-architecture/PLUGIN_V1.md](creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| Role manifest | [roles/README_MANIFEST.md](roles/README_MANIFEST.md) |
| Directory plugins | [creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) |

Legacy `docs/*.md` → see [docs/README.md](docs/README.md).

## Repositories

| Part | Role |
|------|------|
| **This repo** | Runtime desktop client + dialogue engine |
| **Role packs** | Under `roles/`; on-disk layout is the contract |
| **oclive-pack-editor** | Separate repo — pack authoring |
| **oclive-launcher** | [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher) — paths, **`OCLIVE_ROLES_DIR`**, zip install |

## First run

1. Install Node.js, Rust, Ollama (default local LLM). See [CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md).
2. Set **`OCLIVE_ROLES_DIR`** to the roles root.
3. `npm install` then `npm run tauri:dev`.

## Requirements

- Node.js 18+, npm, Rust stable, Ollama (optional for some tests)
- Windows: Visual Studio Build Tools

## Develop

```bash
npm install
npm run tauri:dev
```

### Local HTTP API

Same binary with **`--api`** on **`127.0.0.1`** (default **8420**, **`OCLIVE_API_PORT`** / **`--port`**).

- **`GET /health`** → `ok`
- **`POST /chat`** → JSON with **`reply`**, `personality_source`, etc. Set **`OCLIVE_HTTP_API_MOCK_LLM=1`** for CI / headless.

## Test & CI

See **Testing** in this file (Chinese README mirrors commands). **`npm run check`**, **`npm run check:release`**, **`npm run test:unit`**.

## Disclaimer

Full text on **model weights & licenses**, **third-party plugin responsibility**, and **local data & telemetry**: **[creator-docs/legal/DISCLAIMER.md](creator-docs/legal/DISCLAIMER.md)** (English-focused mirror: [creator-docs-en/legal/DISCLAIMER.md](creator-docs-en/legal/DISCLAIMER.md)). Third-party risk cross-link: [SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md).

## License

AGPL-3.0 with Oclive plugin exception — [LICENSE](LICENSE).

## Contributing

- [CONTRIBUTING.md](CONTRIBUTING.md) · [CONTRIBUTING.en.md](CONTRIBUTING.en.md)
- [SECURITY.md](SECURITY.md)
