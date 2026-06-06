# `oclive_monolith_builtin`（脚手架七焊接键焊接桩）

本目录是 **Monolith 模式下七焊接键静态入口的权威来源**（第 1–6 模块 + `complex_emotion` 设施焊接键）：`oclive-cli` 在 `init --monolith` / `oclive build` 时将其复制到生成项目的 **`vendor/oclive_monolith_builtin/`**，与 `process_message_monolith.rs` 中的 `oclive_monolith_builtin::<slot>::invoke()` 一一对应。

## 主仓对接约定

- **不要**在 `crates/oclive_kernel_host/src/domain/` 等处维护第二套同名焊接桩，以免与脚手架分叉。
- 接入真实内核时：在生成（或 fork 的）内核工程中，将 **`vendor/oclive_monolith_builtin`** 替换为对正式 `oclive_*_builtin` / `oclive_kernel_runtime` 符号的依赖，并保持 **焊接键名** 与 [`PLUGIN_V1.md`](../../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)、[`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../../../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) 一致。

## 源码路径

- 权威实现：`crates/oclive-cli/monolith_vendor/oclive_monolith_builtin/lib.rs`（由 `monolith_codegen::copy_monolith_vendor` 嵌入复制）。
