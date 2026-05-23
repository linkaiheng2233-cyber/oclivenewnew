# OOCP protocol test suite (S0–S12, 13 scenarios; optional S13)

**Status (`main`)**: Checked in under **`examples/oocp-test-suite/`** (`run.mjs` + JSON schema); CI workflow **`.github/workflows/ci.yml`** job **`oocp-test-suite`** builds `oclivenewnew-tauri`, starts the **`--api` HTTP** service, polls **`GET /health`**, runs **`node run.mjs`**, then runs **`scripts/e2e-core-api-restart.mjs`** (restart process, chat again; failure fails the job). The **`frontend`** job runs **Playwright + `vite preview` first-screen smoke** (**A1.1b**) on **Ubuntu** after **`npm run build`** (Windows `frontend` skips Playwright).

## A1.1 PoC: core HTTP restart smoke

- **Script:** repo root **`scripts/e2e-core-api-restart.mjs`** (Node 20+ `fetch`, no extra npm deps).  
- **Behaviour:** **start `--api` → `/health` → `POST /chat` → terminate → start again → `/health` + `POST /chat`** on the same port; both cycles must pass. Defaults to **`OCLIVE_HTTP_API_MOCK_LLM=1`** (**no Ollama**).  
- **Local:** after `cargo build -p oclivenewnew-tauri`, from repo root **`npm run test:e2e:core-api-restart`** (or set `OCLIVE_ROLES_DIR` / `OCLIVE_E2E_PORT` / `OCLIVE_E2E_BINARY`).  
- **Scope:** **host process** “restart and recover” for HTTP (**A1.1a**); **`vite build` + `vite preview` + Playwright** first-screen smoke is **A1.1b** below (CI **`frontend`** job). **Installer / native Tauri window / full WebDriver** is tracked as **A1.1c** in [PRODUCT_RELEASE_CHECKLIST.md](../../handoff/PRODUCT_RELEASE_CHECKLIST.md).

## A1.1b: Web preview shell (Playwright)

- **Specs:** repo root [`e2e/preview-shell.spec.ts`](../../e2e/preview-shell.spec.ts) (`#app` mount + document title).  
- **Local:** `npm run build && npm run test:e2e:preview` (first time: `npx playwright install chromium`; Linux: `npx playwright install --with-deps chromium`).  
- **CI:** **`frontend`** job on **Ubuntu** starts **`vite preview`** in the background (default port **4180**), sets **`PW_TEST_USE_EXTERNAL=1`**, then runs **`npm run test:e2e:preview`**; **`PLAYWRIGHT_DISABLE_HEADLESS_SHELL=1`** reduces extra browser downloads.

## How to run

- **Locally**: see [`examples/oocp-test-suite/README.md`](../../examples/oocp-test-suite/README.md).
- **Environment variables**:
  - `OCLIVE_API_BASE`: default `http://127.0.0.1:8420`
  - `OCLIVE_OOCP_ROLE_PATH`: role pack directory (default `<repo>/roles/mumu`)
  - **`OCLIVE_HTTP_API_MOCK_LLM=1`** (with `--api` only): in-memory store + fixed-reply mock LLM; **enabled by default in CI**, no local Ollama required.

## Scenario table (HTTP black-box)

| ID | Assertion focus |
|----|-----------------|
| S0 | `GET /health` → 200, body `ok` |
| S1 | `POST /chat` empty message → 400, `error.code=EMPTY_MESSAGE` |
| S2 | Invalid `role_path` → 400, `INVALID_ROLE_PATH` or a kernel load code (e.g. `ROLE_NOT_FOUND`) |
| S3 | `role_path=""` → 400 with error body |
| S4 | Valid chat → 200, top-level `reply` non-empty |
| S5 | Success body includes `personality_source` (`vector` \| `profile`) |
| S6 | When `session_id` is sent, it is echoed back |
| S7 | When `scene_id` is sent, response `scene_id` matches |
| S8 | Chinese + emoji user line → 200 |
| S9 | Long user line (400 chars) → 200 |
| S10 | Two consecutive rounds with same `session_id` → both 200 |
| S11 | Success body includes `api_version`, `schema`, `timestamp` |
| S12 | Error body `error.code` is a **string** (`KernelErrorBody`), not a JSON-RPC integer code |

**Default suite:** `run.mjs` runs **S0–S12** (**13** scenarios). **S13** (dual-core experimental failure silently falls back to Stable with `reply`) is optional: `--include-s13` or `OCLIVE_OOCP_INCLUDE_S13=1`.

## Conformance report

`npm run test:json` emits JSON; field set is defined in `examples/oocp-test-suite/schemas/oclive.protocol_conformance_report.v1.schema.json`.

## Relationship to full OOCP

The main app **`--api`** mode is **HTTP** (`GET /health`, `POST /chat`), **without** a WebSocket method chain. This suite validates the **HTTP try-chat contract** and orchestration results; if WS semantics from the spec land later, extend scripts and CI steps under this directory.

**Doc alignment**: Matches root **`README.md`** / **`AGENTS.md`**: **OOCP 13 scenarios (S0–S12)**, plus optional **S13** dual-core fallback; CI job **`oocp-test-suite`**; directory **`examples/oocp-test-suite/`**.

## Test stack overview

See [`OVERVIEW.md`](./OVERVIEW.md) in the same folder.

---

[中文](../../creator-docs/testing/OOCP_TEST_SUITE.md)
