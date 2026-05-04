# oclive_kernel_models

介于 **`oclive_kernel_core`**（trait / 端口）与 **`oclive_kernel_runtime`**（编排与基础设施）之间的 **纯数据** 层：事件类型、七维人格向量、角色包中与 Prompt/事件估计相关的配置片段等。

- **不依赖** `oclive_kernel_core`（与之并行）。
- **不包含** Repository、`PluginHost`、网络或异步运行时逻辑。

`PromptInput` 本体仍留在 `kernel_runtime`（远程 `prompt.build_prompt` 需序列化完整 `Role`）；本 crate 提供其依赖的结构类型。
