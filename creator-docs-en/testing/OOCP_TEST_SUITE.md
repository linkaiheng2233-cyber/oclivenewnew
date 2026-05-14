# OOCP protocol test suite (S0–S11)

**Status (`main`)**: Checked in under **`examples/oocp-test-suite/`** (`run.mjs` + JSON schema); CI workflow **`.github/workflows/ci.yml`** job **`oocp-test-suite`** builds `oclivenewnew-tauri`, starts the **`--api` HTTP** service, polls **`GET /health`**, runs **`node run.mjs`** (failure fails the job).

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
| S1 | `POST /chat` empty message → 400, `error.code=empty_message` |
| S2 | Invalid `role_path` → 400, `invalid_role_path` or `load_role_failed` |
| S3 | `role_path=""` → 400 with error body |
| S4 | Valid chat → 200, top-level `reply` non-empty |
| S5 | Success body includes `personality_source` (`vector` \| `profile`) |
| S6 | When `session_id` is sent, it is echoed back |
| S7 | When `scene_id` is sent, response `scene_id` matches |
| S8 | Chinese + emoji user line → 200 |
| S9 | Long user line (400 chars) → 200 |
| S10 | Two consecutive rounds with same `session_id` → both 200 |
| S11 | Success body includes `api_version`, `schema`, `timestamp` |

## Conformance report

`npm run test:json` emits JSON; field set is defined in `examples/oocp-test-suite/schemas/oclive.protocol_conformance_report.v1.schema.json`.

## Relationship to full OOCP

The main app **`--api`** mode is **HTTP** (`GET /health`, `POST /chat`), **without** a WebSocket method chain. This suite validates the **HTTP try-chat contract** and orchestration results; if WS semantics from the spec land later, extend scripts and CI steps under this directory.

**Doc alignment**: Matches this repo’s root **`README.md`** / **`AGENTS.md`** on CI job name **`oocp-test-suite`**, scenario count **S0–S11**, and directory **`examples/oocp-test-suite/`**.

## Test stack overview

See [`OVERVIEW.md`](./OVERVIEW.md) in the same folder.
