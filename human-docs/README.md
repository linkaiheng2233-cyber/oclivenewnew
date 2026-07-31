# 人类开发者接手包

**读者**：准备修改 OCLive 主仓代码的 Rust / Vue 开发者。
**目标**：30 分钟跑起来，1 小时找到所属模块，随后只读与任务有关的文档。
**最后更新**：2026-07-19。

创作者不需要读本包，请直接走 [创作者黄金路径](../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md)。AI Agent 从 [AGENTS.md](../AGENTS.md) 开始。

## 30 分钟开始工作

1. 阅读 [02 · 三十分钟跑通](02_THIRTY_MINUTE_START.md)，完成依赖安装与基础门禁。
2. 阅读 [04 · 工程约束](04_ENGINEERING_RULES.md)，知道分层、测试和文档边界。
3. 在 [模块选择器](modules/README.md) 选与你任务对应的一份开工包。
4. 按开工包给出的源码锚点和验收命令工作；不要默认通读整个 `handoff/`。

## 按角色选择路径

| 你要做什么 | 最短路径 |
|------------|----------|
| Vue / Chat Pro 界面 | [02 跑通](02_THIRTY_MINUTE_START.md) → [前端路径](paths/frontend.md) → [Chat Pro 开工包](modules/surfaces/frontend-chat-pro.md) |
| Rust 内核 / 主编排 | [01 简架构](01_ARCHITECTURE_SIMPLE.md) → [06 内核路径](06_KERNEL_LEARNING_PATH.md) → [BUS_FACTOR](../handoff/BUS_FACTOR_NOTES.md) |
| 六槽模块 | [03 术语](03_GLOSSARY.md) → [模块选择器](modules/README.md) → 对应 `modules/slots/*.md` |
| 插件开发 | [插件作者路径](paths/plugin-author.md) → [插件契约](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 硬件 / 无头 / 新发行版 | [集成路径](paths/integrator.md) → [HostProfile](../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) |
| 第一个小 PR | [07 常见任务](07_COMMON_TASKS.md) → [PR 门禁](08_PR_GATE_MATRIX.md) → [Good First Issues](../handoff/GOOD_FIRST_ISSUES.md) |

## 学习阶梯

| 层 | 文档 | 解决的问题 | 何时读 |
|----|------|------------|--------|
| L0 | [00 愿景](00_VISION_AND_POSITIONING.md) | 为什么做、边界是什么 | 第一天 |
| L1 | [01 简架构](01_ARCHITECTURE_SIMPLE.md) | 一轮对话、六槽、三套记忆 | 第一天 |
| L2 | [02 跑通](02_THIRTY_MINUTE_START.md) | 构建与本地验证 | 必读 |
| L3 | [03 术语](03_GLOSSARY.md) · [04 规则](04_ENGINEERING_RULES.md) | 代码语言与贡献纪律 | 必读 |
| L4 | [05 调试](05_DEBUGGING.md) | 如何定位常见故障 | 遇到问题时 |
| L5 | [06 内核路径](06_KERNEL_LEARNING_PATH.md) | 深入 `process_message` | 仅内核维护者 |
| L6 | [07 常见任务](07_COMMON_TASKS.md) | 从任务到文件与测试 | 开工时 |
| L7 | [08 资料地图](08_REFERENCE_MAP.md) | 查专题 SSOT | 按需 |

补充入口：[Windows 环境](10_SETUP_WINDOWS.md) · [PR 门禁矩阵](08_PR_GATE_MATRIX.md) · [英文镜像](../human-docs-en/README.md)。

## 文档使用原则

- 本目录负责“怎么学、怎么开始”，不复制契约长表。
- 模块定义只查 [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md)；公开契约只查 [creator-docs](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)。
- 当前进度和债务只查 [TECHNICAL_DEBT_INVENTORY](../handoff/TECHNICAL_DEBT_INVENTORY.md)。
- `handoff/archive/` 是历史记录，不是当前实现依据。

完成标准：能运行基础门禁、找到任务所属模块、指出相关源码与 SSOT，并知道本次改动应跑哪些测试。
