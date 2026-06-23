# 人类开发者接手包

> **读者**：不用 Cursor 的 Rust / Vue 工程师，要在 3–5 个工作日内完成「clone → 跑通主仓 → 读懂主链 → 第一个内核 PR」。  
> **耗时**：按学习阶梯 L0–L6 约 **2–3 天**（有维护者带教可压到 **1–2 天**）。  
> **下一篇**：从 [00 愿景与定位](00_VISION_AND_POSITIONING.md) 或已会 Rust 则直接 [02 三十分钟跑通](02_THIRTY_MINUTE_START.md)。

本目录是 **窄入口、时间盒、验收标准** 的人类文档；契约细节与 RFC 仍在 [`creator-docs/`](../creator-docs/) 与 [`handoff/`](../handoff/)（**AI 接手包**，见 [`ai-package/README.md`](ai-package/README.md)）。

---

## 学习阶梯

| 层级 | 文档 | 核心问题 | 约耗时 |
|------|------|----------|--------|
| **L0** | [00 愿景与定位](00_VISION_AND_POSITIONING.md) | 这是什么、不是什么 | 15 min |
| **L1** | [01 简架构](01_ARCHITECTURE_SIMPLE.md) | 一轮对话怎么流 | 30 min |
| **L2** | [02 三十分钟跑通](02_THIRTY_MINUTE_START.md) | 主仓怎么跑、怎么验 | 30 min |
| **L3** | [03 术语表](03_GLOSSARY.md) + [04 工程约束](04_ENGINEERING_RULES.md) | 缩写与 PR 必守规则 | 45 min |
| **L4** | [05 调试](05_DEBUGGING.md) | 不用 AI 怎么查问题 | 30 min |
| **L5** | [06 内核学习路径](06_KERNEL_LEARNING_PATH.md) | 内核主链怎么读 | 半天 |
| **L6** | [07 常见任务](07_COMMON_TASKS.md) | 改 X 从哪下手 | 按需 |
| **L7** | [08 资料地图](08_REFERENCE_MAP.md) | 深文档去哪找 | 按需 |
| **L8** | [08 PR 门禁矩阵](08_PR_GATE_MATRIX.md) · [09 术语速查](09_GLOSSARY.md) · [10 Windows 附录](10_SETUP_WINDOWS.md) | 本地 CI 对照 · 缩写 · MSVC | 按需 |

英文镜像（L0–L3 + L7–L10 摘要）：[human-docs-en/README.md](../human-docs-en/README.md)

```mermaid
flowchart LR
  L0["L0 愿景\n00"] --> L1["L1 简架构\n01"]
  L1 --> L2["L2 跑通\n02"]
  L2 --> L3["L3 规则+术语\n03+04"]
  L3 --> L4["L4 调试\n05"]
  L4 --> L5["L5 主链\n06"]
  L5 --> L6["L6 首 PR\n07"]
  L6 --> L7["L7 按需\n08"]
```

**刻意延后到 L7**：`dual_core`、Monolith、蓝图 v3 迁移、handoff 归档、姊妹仓细节。

---

## 按角色分流

| 你是谁 | 建议路径 |
|--------|----------|
| **内核贡献者**（Rust 编排 / 持久化 / 插件 wiring） | L0 → L2 → L3 → L5 → L6；深度链 [BUS_FACTOR_NOTES](../handoff/BUS_FACTOR_NOTES.md) |
| **前端贡献者**（Vue / Tauri invoke） | L2 → [paths/frontend.md](paths/frontend.md) → L4 |
| **插件作者**（目录 / Remote 后端） | L0 → [paths/plugin-author.md](paths/plugin-author.md) → [08 资料地图](08_REFERENCE_MAP.md) §插件 |
| **集成方**（无头 HTTP / 硬件嵌入） | L2 → [paths/integrator.md](paths/integrator.md) |
| **Chat Pro 垂直 sprint**（视觉 / 语音） | [team/README.md](team/README.md) → **[SCOPE_AND_BOUNDARIES.md](team/SCOPE_AND_BOUNDARIES.md)** + 分轨任务 |

---

## 验收（自学完成标准）

- [ ] 仅读 [02](02_THIRTY_MINUTE_START.md) 能 `npm run tauri:dev` 并完成 `npm run check`
- [ ] 在 [03](03_GLOSSARY.md) 找到 `srid` 定义，无需读源码
- [ ] [06](06_KERNEL_LEARNING_PATH.md) 能指引到 `process_message.rs` 并完成 domain 单测草稿
- [ ] [04](04_ENGINEERING_RULES.md) 覆盖 PromptBuilder / guardrails / import 三条高频漏项

---

## 深度链接（SSOT）

| 主题 | 文档 |
|------|------|
| 架构总述 | [OCLIVE_ARCHITECTURE_OVERVIEW](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| 命名与 import | [NAMING_CONVENTIONS](../creator-docs/NAMING_CONVENTIONS.md) |
| 贡献与测试 | [CONTRIBUTING](../CONTRIBUTING.md) |
| **Chat Pro 垂直 sprint（组员）** | [team/README.md](team/README.md) |
| AI 接手包 | [ai-package/README.md](ai-package/README.md) |
