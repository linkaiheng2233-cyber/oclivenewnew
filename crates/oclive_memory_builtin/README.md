# oclive_memory_builtin

**官方默认记忆模块**（工程名：设施 crate / `*_builtin`）：Kernel V2 **阶段 5-1** 拆出，提供与历史行为一致的 **记忆排序 / 上下文装配 / 关键词过滤** 纯函数（`classic`），以及可选的 **`MemoryRetrieval` 内置实现**（feature `providers`）。术语对照见 `creator-docs/kernel/KERNEL_BOUNDARY.md` §1.1。

- **trait 定义**仍在 `oclive_kernel_core::memory_retrieval`。
- **SQLite / 长期记忆持久化**仍在 `oclive_kernel_runtime`（本 crate 不访问数据库）。
- 目录插件示例见仓库 **`examples/oclive-memory-builtin-directory/`**，通过 `plugin_backends.memory = directory` 与 `memory.rank` JSON-RPC 恢复与内置等价的排序能力。
