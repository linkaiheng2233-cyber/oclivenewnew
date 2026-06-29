# Kernel error codes and JSON body convention (single source of truth)

**Status**: current contract (matches `oclive_kernel_runtime::KernelErrorBody` and `AppError::code`).

## 1. Machine `code` (one naming rule only)

- **Shape**: **`SCREAMING_SNAKE_CASE`** only (uppercase ASCII + underscores).
- **Origin**:
  - Most errors map to [`AppError`](../../kernel/crates/oclive_kernel_runtime/src/error.rs); **`code` must equal `AppError::code()`** (e.g. `ROLE_NOT_FOUND`, `LLM_ERROR`, `TXN_*`).
  - **Host directory-plugin `ApiError`** (`distros/desktop-tauri/src/api/error.rs`): same **one-line `KernelErrorBody` JSON** (`code` stays `SCREAMING_SNAKE_CASE`, e.g. **`API_PLUGIN_NOT_FOUND`**).
  - **HTTP `POST /chat` boundary** (request checks, `spawn_blocking` panic, etc.) uses the in-crate constant module **`http_chat_codes`** (no duplicate string literals):
    - `EMPTY_MESSAGE`
    - `INVALID_ROLE_PATH`
    - `LOAD_ROLE_TASK_PANIC`
- **Do not**: use camelCase or lowercase snake in the JSON `code` field (legacy OOCP try-chat names are removed); do not introduce a second naming style.

## 2. Payload shape `KernelErrorBody`

| Field | Rule |
|-------|------|
| `code` | Machine code from §1. |
| `message` | Technical English (`AppError` `Display` or route-built); user-facing copy comes from **`code` → i18n** (e.g. `apiErrors.*`). |
| `hint` | Optional; omitted by default in kernel; HTTP try-chat may add a local-language next step. |

## 3. Transport (wrapper only; fields identical)

| Channel | Form |
|---------|------|
| **Tauri `invoke` failure** | Failure string is **one line of JSON**, i.e. a single serialized `KernelErrorBody` (not `[CODE] message` as the primary format). |
| **HTTP `POST /chat` failure** | **`{ "error": KernelErrorBody }`**. |

Clients should **`JSON.parse` first**; on failure, fall back to legacy **`[CODE]`** prefixes (old logs/builds).

## 4. Relation to JSON-RPC sidecar errors

Sidecars use **JSON-RPC numeric `code` + lowercase snake `message` names**; see [ERROR_CODES.md §2](ERROR_CODES.md). **Do not** put RPC integer codes into `KernelErrorBody.code`; when both exist, keep each protocol on its own fields.

## 5. Change discipline

- New **`AppError` variant**: implement `code()`, extend frontend **`apiErrors` (en/zh)**, update [ERROR_CODES.md](ERROR_CODES.md) tables if user-facing.
- New **HTTP-only boundary code**: add a `pub const` in `http_chat_codes` first, then use it in routes; register in [ERROR_CODES.md](ERROR_CODES.md) §1.

## 6. Code and patch notes

- Rust: `kernel/crates/oclive_kernel_runtime/src/error.rs` (`KernelErrorBody`, `AppError`, `http_chat_codes`).
- HTTP: `distros/desktop-tauri/src/http_api.rs`.
- Patch summary: `handoff/A2_KERNEL_JSON_ERROR_PATCH.md`.
- **A3 (crash reporting & user-visible error polish)**: [`handoff/archive/A3_CLOSURE_SUMMARY.en.md`](../../handoff/archive/A3_CLOSURE_SUMMARY.en.md) · [`handoff/archive/A3_CLOSURE_SUMMARY.md`](../../handoff/archive/A3_CLOSURE_SUMMARY.md).

[中文原文](../../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)
