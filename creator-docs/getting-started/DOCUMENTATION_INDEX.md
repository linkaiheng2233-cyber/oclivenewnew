# A.I.Live 文档索引

**SSOT 范围**：本文只负责“去哪里读”，不复制架构、契约、进度或测试表。
**最后更新**：2026-07-28。
**原则**：先按身份选择一条入口；遇到具体问题再查专题文档。

## 先选择你的身份

| 我是谁 | 第一篇 | 预计开始工作 | 下一步 |
|--------|--------|--------------|--------|
| **普通用户** | [用户手册](USER_MANUAL.md) | 15 分钟 | [错误与排障](ERROR_CODES.md) |
| **角色包创作者** | [创作者黄金路径](CREATOR_GOLDEN_PATH.md) | 30 分钟 | [创作者学习路径](../role-pack/CREATOR_LEARNING_PATH.md) |
| **主仓开发者** | [人类开发者接手包](../../human-docs/README.md) | 30 分钟跑通 | [按模块选择开工包](../../human-docs/modules/README.md) |
| **插件作者** | [插件作者学习路径](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) | 30–60 分钟 | [插件契约](../plugin-and-architecture/PLUGIN_V1.md) |
| **硬件 / 无头集成方** | [内核集成路径](KERNEL_INTEGRATOR_LEARNING_PATH.md) | 约 1 小时 | [发行版能力](../kernel/DISTRO_CAPABILITY_PROFILE.md) |
| **维护者 / AI Agent** | [AGENTS.md](../../AGENTS.md) | 按任务 | [AI 深读索引](../../handoff/AI_READING_INDEX.md) |

## 创作者专题

| 要做什么 | SSOT |
|----------|------|
| 角色包与蓝图职责边界 | [ROLE_PACK_BOUNDARY](../../handoff/ROLE_PACK_BOUNDARY.md) |
| 角色包磁盘格式 | [ROLE_PACK_SPEC](../role-pack/ROLE_PACK_SPEC.md) |
| 人格、记忆种子与身份文件 | [ROLE_PACK_SPEC](../role-pack/ROLE_PACK_SPEC.md) · [角色包创作流程](CREATOR_WORKFLOW.md) |
| Chat Pro 成人角色扩展 | [ROLE_PACK_SPEC · `adult_extension.json`](../role-pack/ROLE_PACK_SPEC.md#chat-pro-成人角色扩展adult_extensionjson--可选) |
| 世界观与知识文件 | [WORLDVIEW_KNOWLEDGE](../role-pack/WORLDVIEW_KNOWLEDGE.md) |
| 包版本与迁移 | [PACK_VERSIONING](../role-pack/PACK_VERSIONING.md) · [v1→v2](../role-pack/V1_TO_V2_MIGRATION.md) · [v2→v3](../role-pack/V2_TO_V3_MIGRATION.md) |
| 编写器兼容 | [COMPATIBILITY](../COMPATIBILITY.md) |

## 开发者专题

| 要改什么 | 入口 |
|----------|------|
| 一轮对话主链 | [内核学习路径](../../human-docs/06_KERNEL_LEARNING_PATH.md) · [关键文件锚点](../../handoff/BUS_FACTOR_NOTES.md) |
| 模块定义与六槽关系 | [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md) |
| DTO 与编排契约 | [PLUGIN_V1](../plugin-and-architecture/PLUGIN_V1.md) |
| canonical import 与术语 | [NAMING_CONVENTIONS](../NAMING_CONVENTIONS.md) |
| 聊天、短期与长期记忆 | [CHAT_STORAGE_ARCHITECTURE](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) |
| Tauri invoke 热路径 | [INVOKE_HOTPATH_MATRIX](../../handoff/INVOKE_HOTPATH_MATRIX.md) |
| 破坏性变更 | [BREAKING_CHANGE_PROCESS](../../handoff/BREAKING_CHANGE_PROCESS.md) |
| 当前技术债与冻结项 | [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |

## 插件与集成专题

| 主题 | SSOT |
|------|------|
| 插件总体契约 | [PLUGIN_V1](../plugin-and-architecture/PLUGIN_V1.md) |
| 目录插件与权限 | [DIRECTORY_PLUGINS](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Remote JSON-RPC | [REMOTE_PLUGIN_PROTOCOL](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| llama.cpp LoRA GGUF / `.ocadapter` | [LORA_ADAPTER_PACKAGE](../plugin-and-architecture/LORA_ADAPTER_PACKAGE.md) |
| Agent / MCP | [AGENT_REMOTE_PROTOCOL](../plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md) |
| Bridge API | [BRIDGE_API_REFERENCE](../plugin-and-architecture/BRIDGE_API_REFERENCE.md) |
| HostProfile / 跨平台 | [DISTRO_CAPABILITY_PROFILE](../kernel/DISTRO_CAPABILITY_PROFILE.md) |
| 内核生命周期 | [DISTRO_KERNEL_LIFECYCLE](../kernel/DISTRO_KERNEL_LIFECYCLE.md) |
| 蓝图扩展外壳 / ExecutionPlan / 统一资源协调 | [RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md) |
| Scaffold Package / 自定义脚手架 | [RFC_SCAFFOLD_PACKAGE_V1](../rfc/RFC_SCAFFOLD_PACKAGE_V1.md) |

## 测试、安全与发布

| 主题 | SSOT |
|------|------|
| 测试分层 | [测试总览](../testing/OVERVIEW.md) |
| OOCP 黑盒 | [OOCP_TEST_SUITE](../testing/OOCP_TEST_SUITE.md) |
| 安全范围 | [SECURITY_AUDIT_SCOPE](../security/SECURITY_AUDIT_SCOPE.md) |
| 已知供应链风险 | [KNOWN_VULNERABILITIES](../security/KNOWN_VULNERABILITIES.md) |
| 版本事实 | [PROJECT_CURRENT_STATUS](PROJECT_CURRENT_STATUS.md) |
| 发版版本规则 | [RELEASE_VERSIONING](../development/RELEASE_VERSIONING.md) |
| 贡献与本地门禁 | [CONTRIBUTING](../../CONTRIBUTING.md) |

## 文档边界

| 目录 | 只负责 |
|------|--------|
| `human-docs/` | 人类开发者顺序学习与模块开工包 |
| `creator-docs/` | 用户、创作者、插件作者的现行契约 |
| `handoff/` | 维护者 / AI 的工程 SSOT、技术债和关键路径 |
| `handoff/archive/` | 历史记录；禁止作为现行 truth |
| `*-en/` | 已有中文 SSOT 的英文镜像 |

文档归属与新增限制见 [handoff/README](../../handoff/README.md)。找不到位置时，优先修正上述入口或扩展既有 SSOT，不新建第二份总览。
