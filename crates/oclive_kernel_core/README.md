# `oclive_kernel_core`

V2 **极薄内核**的物理地基（首阶段）：与具体存储、LLM、市场等实现解耦的 **错误类型**、**记忆模型** 与 **Repository trait**。

- **默认实现**仍在 `oclive_kernel_runtime`（及后续可选 `default-*` feature）。
- 桌面宿主将 `AppError` 映射为 `String` / `InvokeError` 时，请在 `src-tauri` 侧使用 `map_err(|e| e.to_frontend_error())` 等显式转换（不再在 runtime 上提供 `tauri_invoke` 特性）。
