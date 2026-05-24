# A3 closure summary (crashes & diagnostics, 2026-05-15)

[中文原文](./A3_CLOSURE_SUMMARY.md)

## A3.1 Sentry (off by default, user can opt out, README-aligned)

- **Build time**: Sentry may initialize only when **`VITE_SENTRY_DSN`** is set at build time; no DSN → no reporting.
- **Runtime**: **`src/utils/telemetrySentry.ts`** — if **`localStorage`** key **`oclive.telemetry.sentryOptOut`** is **`1`**, **`Sentry.init`** is skipped (kept in sync with Settings).
- **Settings UI**: When the build ships with a DSN, **Settings → General** shows **Crash diagnostics (Sentry)**; checking **Disable crash reporting** calls **`Sentry.close`** and persists opt-out; clearing the checkbox prompts a **restart** to resume.
- **Privacy defaults**: `sendDefaultPii: false`, `tracesSampleRate: 0`, **`beforeSend`** strips **query strings** from captured request URLs.
- **Docs**: root **`README.md`** / **`README.en.md`** (Observability section) describe the above.

## A3.2 User-visible errors (JSON `code` + frontend mapping)

- **Directory-plugin `ApiError`** (`src-tauri/src/api/error.rs`): same **one-line `KernelErrorBody` JSON** as the kernel (no `[CODE]` on the primary path); `map_directory_rpc_url_error` and plugin-bridge **`Result<_, String>`** paths use **`to_kernel_json()`** (or `String::from(api_error)`); closures avoid ambiguous **`Into`** inference.
- **Unknown codes**: **`apiErrors.UNKNOWN_WITH_CODE`** (en/zh) + **`toFriendlyErrorMessage`** when no dedicated `apiErrors.<code>` entry exists.
- **Cleanup**: `reset_plugin_state_to_role_default` maps `load_role` failures with **`to_frontend_error()`** (kernel JSON).

## Suggested verification

- `npm run test:unit`
- `cargo test -p oclivenewnew-tauri` (at least `api::error` unit tests)
- Manual: on a DSN-enabled build, toggle disable/re-enable in Settings and confirm restart toasts and the opt-out key.

## Related

- Release checklist: `handoff/PRODUCT_RELEASE_CHECKLIST.md` §A3  
- Product gap list: `handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md` §A3  
- Error code norm: `creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md` (EN mirror: `creator-docs-en/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`)
