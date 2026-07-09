# L03: Cancel in-flight generation (`chat_generation_cancel`)

[中文](../../creator-docs/testing/L03_GENERATION_CANCEL.md)

## Conclusion (current `main`)

Search under `kernel/crates/oclive_kernel_host/src/domain/chat_engine/` for **cancel / abort / interrupt / stop / halt** and Tauri command **`chat_generation_cancel`**: **no** implemented API or engine hook for “cancel ongoing LLM generation” was found.

`process_message` is a single `await` driven main-chain orchestration with no cooperative cancellation token exposed to the frontend.

## Checklist status

Treat **L03** as **planned** (or remove from “done” lists) until one of:

- Tauri command (e.g. `cancel_chat_generation`) + cancellable LLM boundary in the engine; or
- Explicit product decision to drop the capability and document that.

## Related

- [SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md) may mention cancellable LLM expectations — **this file wins** on implementation status.
