# OCLive 用户、创作者与插件文档

**SSOT 范围**：公开使用说明、角色包契约、插件契约与集成规范。
**最后更新**：2026-07-19。
**主仓开发者**从 [human-docs](../human-docs/README.md) 开始；维护者 / AI 从 [handoff](../handoff/README.md) 开始。

## 按身份开始

| 我是谁 | 第一篇 | 接下来 |
|--------|--------|--------|
| 普通用户 | [用户手册](getting-started/USER_MANUAL.md) | [FAQ](FAQ.md) · [错误排查](getting-started/ERROR_CODES.md) |
| 角色包创作者 | [30 分钟黄金路径](getting-started/CREATOR_GOLDEN_PATH.md) | [创作者学习路径](role-pack/CREATOR_LEARNING_PATH.md) · [角色包规范](role-pack/ROLE_PACK_SPEC.md) |
| 插件作者 | [插件作者学习路径](plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) | [PLUGIN_V1](plugin-and-architecture/PLUGIN_V1.md) |
| 硬件 / 无头集成方 | [内核集成路径](getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) | [HostProfile](kernel/DISTRO_CAPABILITY_PROFILE.md) |
| 不确定该读什么 | [完整文档索引](getting-started/DOCUMENTATION_INDEX.md) | — |

## 目录职责

| 目录 | 只负责 |
|------|--------|
| `getting-started/` | 用户、创作者和集成方的入口；仅 `PROJECT_CURRENT_STATUS` 维护版本事实 |
| `role-pack/` | 角色包格式、版本、迁移、身份、知识与跨宿主规则 |
| `plugin-and-architecture/` | 六槽、目录插件、Remote、Agent 与 Bridge 契约 |
| `kernel/` | 发行版能力、内核生命周期与模块语义 |
| `testing/` | 测试分层、OOCP、输出契约与 fuzz |
| `security/` | 安全范围、供应链与已知风险 |
| `development/` | 发版版本、轻量化和开发维护规则 |
| `rfc/` | 尚需保留决策上下文的设计提案；状态以文首为准 |
| `roadmap/` | 中长期愿景；不作为当前完成度或契约 truth |

## 常用 SSOT

| 主题 | 文档 |
|------|------|
| 角色包 vs 蓝图 | [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md) |
| 角色包磁盘格式 | [ROLE_PACK_SPEC](role-pack/ROLE_PACK_SPEC.md) |
| 六槽与 DTO | [PLUGIN_V1](plugin-and-architecture/PLUGIN_V1.md) |
| 目录插件权限 | [DIRECTORY_PLUGINS](plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| 版本兼容 | [COMPATIBILITY](COMPATIBILITY.md) |
| 项目当前版本 | [PROJECT_CURRENT_STATUS](getting-started/PROJECT_CURRENT_STATUS.md) |
| 文档总索引 | [DOCUMENTATION_INDEX](getting-started/DOCUMENTATION_INDEX.md) |

英文镜像见 [creator-docs-en](../creator-docs-en/README.md)。修改已有双语契约时应同步镜像；历史阶段报告不应复制进本目录。
