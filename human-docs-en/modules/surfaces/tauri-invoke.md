# Surface pack · Tauri invoke (EN summary)

> Full checklist (ZH): [`human-docs/modules/surfaces/tauri-invoke.md`](../../human-docs/modules/surfaces/tauri-invoke.md)  
> Matrix SSOT: [INVOKE_HOTPATH_MATRIX](../../handoff/INVOKE_HOTPATH_MATRIX.md)

**You plug in**: Rust `distros/desktop-tauri/src/api/<topic>.rs` · register **only** in `lib.rs` → `generate_handler!` · frontend `distros/shared/src/api/*.ts` (**camelCase**) · delegate to `oclive_kernel_host::service::*_impl`.

**Do**: New command api module + TS wrapper · sync DTOs with `oclive_kernel_types` · hot-path tests `invoke_hotpath_matrix`.

**Don't**: Put business logic in `lib.rs` · expose snake_case to frontend · stack `process_message` orchestration in API layer.

**Read next**: [07 common tasks §1 (EN)](../../07_COMMON_TASKS.md) · [BUS_FACTOR](../../handoff/BUS_FACTOR_NOTES.md) · [frontend-chat-pro](frontend-chat-pro.md) · [ERROR_CODES](../../creator-docs/getting-started/ERROR_CODES.md).
