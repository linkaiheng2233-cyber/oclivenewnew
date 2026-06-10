# 08 · 资料地图（按主题折叠）

> **读者**：L0–L6 完成后按需深挖的工程师。  
> **读完能做什么**：在 AI 接手包（`creator-docs/` / `handoff/`）中按主题找 SSOT，而非 134 链平铺。  
> **耗时**：按需。  
> **下一篇**：回 [human-docs/README](README.md) 或 [ai-package/README](ai-package/README.md)。

**英文缺口**：内核/角色包深度文部分仅中文；入门 L0–L2 有 [CONTRIBUTING.en.md](../CONTRIBUTING.en.md) 与 [creator-docs-en/](../creator-docs-en/)。

---

## 1. 架构

| 文档 | 用途 |
|------|------|
| [OCLIVE_ARCHITECTURE_OVERVIEW](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) | 六槽 + 设施子模块总述 |
| [crates/README](../crates/README.md) | Crate 依赖与改哪 |
| [DESIGN_DECISIONS](../creator-docs/architecture/DESIGN_DECISIONS.md) | 取舍记录 |
| [ARCHITECTURE_LAYERING](../handoff/ARCHITECTURE_LAYERING.md) | 分层 ratchet |
| [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md) | 角色 vs 蓝图 |

---

## 2. 契约与命名

| 文档 | 用途 |
|------|------|
| [NAMING_CONVENTIONS](../creator-docs/NAMING_CONVENTIONS.md) | 权威名、import、禁止别名 |
| [dto.rs](../crates/oclive_kernel_types/src/models/dto.rs) | HTTP/IPC 字段 |
| [KERNEL_ERROR_CODE_CONVENTION](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) | 错误 JSON |
| [COMPATIBILITY](../creator-docs/COMPATIBILITY.md) | 版本兼容表 |
| [BREAKING_CHANGE_PROCESS](../handoff/BREAKING_CHANGE_PROCESS.md) | Breaking 流程 |

---

## 3. 角色包

| 文档 | 用途 |
|------|------|
| [ROLE_PACK_SPEC](../creator-docs/role-pack/ROLE_PACK_SPEC.md) | 包规范 |
| [CREATOR_LEARNING_PATH](../creator-docs/role-pack/CREATOR_LEARNING_PATH.md) | 创作者路径 |
| [roles/README_MANIFEST](../roles/README_MANIFEST.md) | manifest 字段 |
| [PACK_VERSIONING](../creator-docs/role-pack/PACK_VERSIONING.md) | 版本规则 |
| [CROSS_HOST_MEMORY](../creator-docs/role-pack/CROSS_HOST_MEMORY.md) | 跨宿主记忆 |

---

## 4. 插件

| 文档 | 用途 |
|------|------|
| [PLUGIN_V1](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) | 插件契约 |
| [PLUGIN_AUTHOR_LEARNING_PATH](../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) | 作者路径 |
| [DIRECTORY_PLUGINS](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) | 目录插件 |
| [REMOTE_PLUGIN_PROTOCOL](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) | HTTP JSON-RPC |
| [EXTENSION_POINTS](../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md) | 扩展点索引 |

---

## 5. 内核生命周期

| 文档 | 用途 |
|------|------|
| [DISTRO_KERNEL_LIFECYCLE](../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) | attach/spawn |
| [OCLIVE_APP_DATA](../creator-docs/kernel/OCLIVE_APP_DATA.md) | 数据目录 |
| [KERNEL_INTEGRATOR_LEARNING_PATH](../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) | 集成方 |
| [OCLIVE_CLI_GUIDE](../creator-docs/cli/OCLIVE_CLI_GUIDE.md) | CLI 脚手架 |
| [BUS_FACTOR_NOTES](../handoff/BUS_FACTOR_NOTES.md) | 关键路径 |

---

## 6. 测试与 CI

| 文档 | 用途 |
|------|------|
| [CONTRIBUTING §测试](../CONTRIBUTING.md#测试要求合并前建议全绿) | 本地命令 |
| [OOCP_TEST_SUITE](../creator-docs/testing/OOCP_TEST_SUITE.md) | HTTP 黑盒 |
| [OVERVIEW](../creator-docs/testing/OVERVIEW.md) | 三层测试 |
| [INVOKE_HOTPATH_MATRIX](../handoff/INVOKE_HOTPATH_MATRIX.md) | invoke 矩阵 |
| [DIMENSION5_CLOSURE_SIGNOFF](../handoff/DIMENSION5_CLOSURE_SIGNOFF.md) | ratchet 门禁 |

---

## 7. 发版与安全

| 文档 | 用途 |
|------|------|
| [RELEASE_VERSIONING](../creator-docs/development/RELEASE_VERSIONING.md) | SemVer |
| [PRODUCT_RELEASE_CHECKLIST](../handoff/PRODUCT_RELEASE_CHECKLIST.md) | 发版勾选 |
| [KNOWN_VULNERABILITIES](../creator-docs/security/KNOWN_VULNERABILITIES.md) | 供应链 |
| [LIGHTWEIGHT_PROFILE](../creator-docs/development/LIGHTWEIGHT_PROFILE.md) | 包体基线 |
| [CHANGELOG](../CHANGELOG.md) | 用户可见变更 |

---

## 8. handoff 深读（维护者）

| 文档 | 用途 |
|------|------|
| [handoff/README](../handoff/README.md) | 活跃 vs 归档 |
| [TECHNICAL_DEBT_INVENTORY](../handoff/TECHNICAL_DEBT_INVENTORY.md) | 技术债 |
| [04_4.6_PROJECT_TRUTH_CHECKLIST](../handoff/04_4.6_PROJECT_TRUTH_CHECKLIST.md) | 认知清单 |
| [CHAT_STORAGE_ARCHITECTURE](../handoff/CHAT_STORAGE_ARCHITECTURE.md) | 聊天存储 |
| [DOCUMENTATION_INDEX](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) | 全量索引 |

---

## 验收

- [ ] 改插件时先打开 §4 而非全目录搜索
- [ ] 知道契约以 `creator-docs/` 中文为准
