# kernel_embed_minimal

可复制到独立 crate 的**最小嵌入式示例**（依赖路径按你的 monorepo 调整）。

- **契约**：`SendMessageResponse` 字段 **`reply`**（见 `crates/oclive_kernel_runtime/src/models/dto.rs`）。
- **运行**：在 oclivenewnew **仓库根**执行 `cargo run -p kernel_embed_minimal`（需要 `roles/shimeng`）。
- **文档**：[KERNEL_SDK.md](../../creator-docs/kernel/KERNEL_SDK.md) §7。
