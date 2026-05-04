# oclive_prompt_builtin

**官方默认 Prompt 模块**（设施 crate / `*_builtin`）：Kernel V2 **阶段 7-1** 拆出，提供 **`PromptBuilder`** 正文（feature **`classic`**，默认开；关时用 `classic/stub`），以及可选的 **`PromptAssembler` 内置实现**（feature **`providers`**，隐含 `classic`）。术语见 `creator-docs/kernel/KERNEL_BOUNDARY.md`。

- **trait / `PromptInput`** 仍在 `oclive_kernel_core::prompt`。
- **编排、侧车 HTTP、完整 `Role`** 仍在 `oclive_kernel_runtime`。
- 目录插件示例见仓库 **`examples/oclive-prompt-builtin-directory/`**（`prompt.build_prompt` + `prompt.top_topic_hint`）；`build_prompt` 通过子进程调用 **`oclive_prompt_from_json`**（需 `prompt-from-json-bin` feature 构建），与进程内算法一致。
