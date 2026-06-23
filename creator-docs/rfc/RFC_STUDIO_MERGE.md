# RFC：oclive 工作室合并（启动器 + 编写器）

| 元数据 | 值 |
|--------|-----|
| 状态 | **已落地**：独立仓库 **[oclive-studio](https://github.com/linkaiheng2233-cyber/oclive-studio)**；原 **oclive-launcher**、**oclive-pack-editor** 归档为 Deprecated |
| 配置 SSOT | **`studio-config.json`**（`rolesDir` → `OCLIVE_ROLES_DIR`，LLM 与运行时路径） |
| 角色包 SSOT | **v2** `pipeline.ocblueprint`；见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |
| 用户文档 | [`handoff/studio/USER_GUIDE.md`](../../handoff/studio/USER_GUIDE.md) · [CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md) |

## 1. 目标

- 单一安装物内提供 **启动模式**（环境诊断、拉起 `oclivenewnew`）与 **创作模式**（编辑 v2 蓝图、校验、试聊、导出）。
- 与运行时仅通过磁盘 **roles 根** 对接，无复杂 IPC。

## 2. 非目标

- 不在工作室进程内嵌入完整 `process_message` 编排；试聊通过 **`--api`** 调用主仓内核。

## 3. 验收（摘要）

- 可配置 roles 根并导入/导出与 `distros/chat-pro/roles/{id}/` 一致的包树。
- 创作模式可编辑 **`pipeline.ocblueprint`** 并通过 `oclive pack validate` 等价规则校验。
- 试聊注入 `OCLIVE_ROLES_DIR` 与 LLM 环境变量（可覆盖 legacy `plugin_backends.llm` 运行时行为，见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)）。

---

[English](../../creator-docs-en/rfc/RFC_STUDIO_MERGE.md)
