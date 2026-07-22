# oclive Breaking 变更流程

本文定义：**什么算 Breaking**、**谁做什么**、**如何审阅与留兼容层**，并与仓库内契约文档、校验 crate、CHANGELOG 对齐。目标：下游（编写器、启动器、侧车、角色包作者、姊妹仓）不因「悄悄改字段」而踩雷。

---

## 1. 什么是 Breaking 变更（oclive 语境）

凡满足以下任一情形，即视为 **Breaking**（需在 PR 中显式声明，并走本文流程）：

| 类别 | 示例 |
|------|------|
| **DTO / JSON 契约** | `SendMessageRequest` / `SendMessageResponse` 字段重命名、删除、语义改变；`KernelErrorBody` 的 `code` 改名或删除；Tauri 命令参数与前端 `invoke` 载荷键不一致。 |
| **枚举与字符串协议值** | 后端 `Emotion`、`RelationState`、插件后端 wire 字符串（如 `builtin_v2` → 改名）；HTTP OOCP 路径或 JSON 字段与文档不一致。 |
| **OOCP / HTTP 聊天 API** | 路由、请求体、成功/失败 JSON 形状变化；与 [`creator-docs/testing/OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md) 已登记场景不兼容。 |
| **manifest / settings 契约** | `manifest.json` 或 `settings.json` **新增必填**顶层键、删除键、或改变类型；`permissions` 允许值集合缩小（比「仅新增可选值」更狠）。 |
| **高风险 grant / 权限标识** | `high_risk_grants.json` 或 manifest `permissions` 与运行时门禁使用的 **规范键名** 变更；撤销旧别名且无读兼容期（见下文兼容层）。 |
| **数据库** | 迁移导致旧客户端无法读新库、或去掉 `role_runtime` / `app_settings` 等主表列且无回填脚本。 |
| **对外承诺的 CLI / 脚手架输出** | `oclive-cli` 生成目录结构或 `monolith.toml` 语义不可向后兼容（若仅内部模板可调，也须在 CHANGELOG 写明）。 |

**通常不算 Breaking**（仍建议在 CHANGELOG 记一笔）：

- 纯文档澄清、错别字、示例补充。
- **仅新增**可选 DTO 字段、可选 manifest 键、**仅新增**错误码（旧码保留）。
- 内部实现重构，对外 JSON 与命令签名不变。
- 性能优化不改变语义。

若有灰色地带：**按 Breaking 处理**（多写迁移总好过线上静默坏）。

---

## 2. 六步流程

### 步骤 1：识别

- 对照 [`creator-docs/COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md) 与 [`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)。
- 按 [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G17 列出生产者 → 契约 → 适配 → 消费者 → 状态/回退 → 测试；自问旧版宿主 / 编写器 / 侧车 / 官方插件是否仍能与新数据互通。若否 → Breaking。
- 即使最终判定为非 Breaking，也须保留关联影响核对；“仅新增可选字段”不能免除消费者、回退和测试同步。

### 步骤 2：声明

- 在 PR 标题或描述首段写明：**「BREAKING: …」** 或 **「破坏性变更：…」**。
- 使用下文 **PR 描述模板**，填「影响面」「迁移」「兼容层」「回滚」。

### 步骤 3：审阅

- **至少一名**维护者确认：
  - 是否需要 **兼容层**（读旧写新、别名、特性开关）及 **保留多久**。
  - 迁移路径是否可执行（命令、脚本、文档步骤是否闭环）。
- 触及安全 / 权限 / 持久化：优先拉高审阅优先级。

### 步骤 4：迁移指南

- 在 PR 或独立小节（可链到 issue）写清：**谁**（宿主作者 / 包作者 / 姊妹仓）**要改什么**。
- 使用下文 **迁移指南模板**。

### 步骤 5：更新校验（`oclive_validation`）

- 若 manifest / `settings` / 角色包顶层键变化：更新 **`kernel/crates/oclive_validation`**（Rust 校验 + 若有 `json_keys` 等与编写器对齐的源）。
- 运行：`cargo test -p oclive_validation`（及工作区相关测）；编写器侧若依赖 wasm，需按 [`oclive-pack-editor` README](https://github.com/linkaiheng2233-cyber/oclive-pack-editor/blob/main/README.md) 重建 **`npm run wasm:build`** 并跑其契约脚本（如 `contract:json-keys`）。

### 步骤 6：更新文档

以下文件按触及范围 **必查**（未改也要在 PR 说明「已核对无需改」）：

| 文档 | 用途 |
|------|------|
| [`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) | 插件 manifest、权限、`plugin_backends` |
| [`creator-docs/getting-started/ERROR_CODES.md`](../creator-docs/getting-started/ERROR_CODES.md) | 用户可见错误与机器码 |
| [`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) | `code` 命名规范 |
| [`creator-docs/COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md) | 版本与工具链一页表 |
| [`creator-docs-en/…`](../creator-docs-en/) 镜像 | 与中文契约同步或 CHANGELOG 声明例外 |
| **`CHANGELOG.md` / `CHANGELOG.en.md`** | 用户可感知变更（CONTRIBUTING 已要求） |

---

## 3. 兼容层要求

- **默认期望**：至少 **一个发布周期** 内，**读路径** 仍能接受旧数据（旧键、旧别名、旧枚举字符串），并写新格式或打日志提示弃用。
- **写路径**：新代码应写规范键；文档写明弃用时间表（若可承诺）。
- **反例**：仅改文档、不在运行时接受旧 grant 键 → 用户已落盘的 `high_risk_grants.json` 会整批失效。

### 实际案例：A4.2 grant 与 `permissions` 键统一

- **变更**：目录插件 manifest `permissions`、校验 crate、运行时门禁统一为 `process:spawn`、`network:*`、`mcp:http`、`mcp:stdio`；grant 与 API 对齐规范 id。
- **兼容层**：`high_risk_grants.rs` **读盘兼容旧 snake_case 键**；`high_risk` API **接受规范 id 与旧别名**（见 [`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)）。
- **迁移**：作者更新 manifest 权限数组；用户侧重新授权时使用新键；文档见 PLUGIN_V1 §权限规范。

### 近期非破坏性配置变更（记录）

以下 **`config.json` 可选键** 为 **向后兼容** 新增（未设则走默认值；不阻塞 `load_role`）：

| 日期 | 键 | 默认 | 说明 |
|------|-----|------|------|
| 2026-05 | `chat_storage.backend` | `hybrid` | 聊天存储后端：`hybrid` / `file` / `sqlite` |
| 2026-05 | `chat_storage.replay_similarity_threshold` | `0.6` | 记忆回放去重相似度阈值 |

详见 [CHAT_STORAGE_ARCHITECTURE.md](./CHAT_STORAGE_ARCHITECTURE.md) · [SETTINGS_REFERENCE.md §六](../creator-docs/cli/SETTINGS_REFERENCE.md)。

---

## 4. PR 描述模板（复制到 PR）

```markdown
## BREAKING: （一句话摘要）

### 影响面
- [ ] 桌面宿主 Tauri / HTTP API
- [ ] `distros/shared` / Chat Pro / Theater 消费者
- [ ] 官方目录插件的 Vue 入口 / iframe 回退 / Bridge / RPC
- [ ] 角色包 manifest / settings
- [ ] oclive_validation / 编写器 wasm
- [ ] 姊妹仓（launcher / pack-editor / market）需同步版本或行为

### 兼容层
- 读旧：是 / 否（说明哪些键/路径仍可读）
- 写新：是 / 否
- 计划移除旧兼容的时间：（版本或「待定」）

### 迁移指南
- 链接：（issue 或本文档 §）

### 校验与文档
- [ ] `kernel/crates/oclive_validation` 已更新并 `cargo test` 相关 crate
- [ ] PLUGIN_V1 / ERROR_CODES / COMPATIBILITY / KERNEL_ERROR_CODE_CONVENTION（勾选已改项）
- [ ] CHANGELOG.md + CHANGELOG.en.md

### 验证
- 本地命令：（列出）
```

---

## 5. 迁移指南模板（给下游作者）

```markdown
# 从 {旧版本} 迁移到 {新版本}：{主题}

## 你必须改什么
1. …

## 可选优化
1. …

## 回滚
若需留在旧行为：…

## 参考
- PR：#…
- 文档：…
```

---

## 6. 与发版清单的关系

发版前按 [CONTRIBUTING](../CONTRIBUTING.md) 与 CI 核对：Breaking 是否已写入 CHANGELOG、兼容表和对外说明。

---

## 7. 相关链接

- [`CONTRIBUTING.md`](../CONTRIBUTING.md) / [`CONTRIBUTING.en.md`](../CONTRIBUTING.en.md) — Breaking 小节入口
- [`handoff/PRODUCT_LINE_TASK_BUCKETS.md`](./PRODUCT_LINE_TASK_BUCKETS.md) — 当前产品工程执行视图
- [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) — 文档索引
