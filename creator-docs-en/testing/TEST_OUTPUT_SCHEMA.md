# Test output and contracts (TEST_OUTPUT_SCHEMA)

## Tauri / Rust

- **Main chat response**: front/back contract is defined in **`src-tauri/src/models/dto.rs`**; the user-visible reply field is **`reply`** (not `response`).
- **Integration / API tests**: `src-tauri/tests/*.rs` assert structure with **`serde_json`**; there is **no** unified machine-readable schema file; if JSON fixtures are introduced, place them under `src-tauri/tests/fixtures/` and index them here.

## Frontend

- **Current CI gate**: `npm run build` (production bundle must compile).
- **Unit tests (Vitest, etc.)**: `package.json` **does not** configure `test:unit`; if added later, document **`npm run test:unit`** here with coverage / snapshot policy.

## Local HTTP API (`--api`)

- **`POST /chat`**: success body includes **`reply: string`**; aligned with `SendMessageResponse`. See root [README.md](../../README.md) section “Local HTTP API”.
