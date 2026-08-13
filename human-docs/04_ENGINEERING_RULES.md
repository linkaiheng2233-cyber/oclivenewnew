# 04 · 工程约束（代码 7 条 + 文档纪律）

> **最后更新**：2026-06-26  
> **读者**：准备提内核 PR 的工程师。  
> **读完能做什么**：避免 review 高频打回项（编排位置、DTO 字段、Prompt、`import` 路径）。  
> **耗时**：约 25 分钟。  
> **下一篇**：[05 调试](05_DEBUGGING.md)。

与 [`.cursor/rules/oclivenewnew.mdc`](../.cursor/rules/oclivenewnew.mdc)、[CONTRIBUTING.md §工程约束](../CONTRIBUTING.md#工程约束) **三处镜像**；变更时须同 PR 同步。

---

## 1. 编排只在 `process_message` / `*_engine`

**规则**：对话主流程在 [`process_message.rs`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)；业务公式留在各 `*_engine` / analyzer。**API 层不堆业务**。

**违反时 review 常见意见**：「请把逻辑下沉到 `domain/`，`api/*.rs` 只做参数校验与 `*_impl` 委托。」

---

## 2. 持久化走 repository trait + 迁移 SSOT

**规则**：通过 `domain/repository.rs` trait 与 `infrastructure/repositories.rs` 实现；表结构以 [`kernel/crates/oclive_kernel_host/migrations/001_init.sql`](../kernel/crates/oclive_kernel_host/migrations/001_init.sql) 为准。

**违反时**：「表名/列名与迁移不一致」「禁止虚构 `memory_backend` 表」等。

---

## 3. Tauri 命令注册纪律

**规则**：命令只在 [`distros/desktop-tauri/src/api/*.rs`](../distros/desktop-tauri/src/api/)；仅在 [`lib.rs`](../distros/desktop-tauri/src/lib.rs) 用 `tauri::generate_handler!` 注册。

**违反时**：「请勿在 `lib.rs` 写业务」「新命令缺前端 `distros/shared/src/api/` camelCase 封装」。

---

## 4. DTO 契约

**规则**：前后端以 [`oclive_kernel_types/src/models/dto/mod.rs`](../kernel/crates/oclive_kernel_types/src/models/dto/mod.rs) 为准；回复字段 **`reply`**，不是 `response`；`Emotion` 以 [`emotion.rs`](../kernel/crates/oclive_kernel_types/src/models/emotion.rs) 为准（无未定义变体）。

**违反时**：「请改为 `reply`」「`Joy`/`Fearful` 不在契约枚举内」。

---

## 5. PromptBuilder 签名

**规则**：[`PromptBuilder::build_prompt`](../kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/mod.rs) 单参数 `input: &PromptInput<'_>`，返回 **`String`**（**不是 `Result`，不要用 `?`**）。

**违反时**：「`build_prompt` 不返回 Result，请在上层处理缺字段」。

---

## 6. guardrails 不可被包级锚点替换

**规则**：每轮恒追加 `KERNEL_DIALOGUE_GUARDRAILS`；角色包 `reply_quality_anchor` **仅替换** `DEFAULT_REPLY_QUALITY_ANCHOR`，**不可替换** guardrails。

**违反时**：「guardrails 是内核常量，不能进角色包覆盖」。

---

## 7. Canonical import（禁止自造槽位别名）

**规则**：见 [NAMING_CONVENTIONS §4.2](../creator-docs/NAMING_CONVENTIONS.md#42-canonical-import-路径)：

| 需要什么 | 从哪 import |
|----------|-------------|
| DTO / `AppError` | `oclive_kernel_types` |
| Trait 端口 | `oclive_kernel_contracts` |
| 编排 | `oclive_kernel_host::domain::…` |
| 路径 / 发现常量 | `oclive_kernel_runtime` |

六槽键用 `plugin_backends` / `slot_registry.type`；**禁止** `memory_backend` 等自造别名。

**违反时**：「请用 canonical crate，勿经 runtime 绕路取 DTO」。

---

## 8. 文档贡献纪律（人类版）

> AI 完整条文见 [AI_CHANGE_BOUNDARIES G10–G16](../handoff/AI_CHANGE_BOUNDARIES.md)。本节用 **人类可读** 方式说明：大项目靠文档有条理，**效率来自限制**——写得多不如写得 **准、准在一处**。

### 8.1 动笔前先找「唯一 SSOT」

1. 打开 [`handoff/README.md` §文档分责](../handoff/README.md) — 你的主题是否 **已有** 负责文档？  
2. **有** → 只改那一份（或只加一节），**不要** 新建 `handoff/某某.md`  
3. **没有** → 需要维护者/RFC 认定「必须新开」；并登记进分责表  

**模块 / 六槽 / 设施关系**：只维护 [`MODULE_MAP_AND_HANDOFF.md`](../handoff/MODULE_MAP_AND_HANDOFF.md)。人类入门读 [01 简架构](01_ARCHITECTURE_SIMPLE.md)；**按模块开工**读 [`modules/`](modules/README.md)（checklist · 链 SSOT）；定义仍只在 MODULE_MAP，**不要在 human-docs 复制整表**。

### 8.2 人类包 vs AI 包

| | human-docs（本目录） | handoff / creator-docs / AGENTS |
|--|----------------------|----------------------------------|
| 读者 | 人 · 顺序学 | AI · 索引 · 契约 |
| 篇幅 | **可长、可细、排版友好** | **短、链出、不重复** |
| 进度 | [human-docs/README 文档包进度](README.md#文档包进度与-ai-包同步--2026-06-25) | [TECHNICAL_DEBT §1](../handoff/TECHNICAL_DEBT_INVENTORY.md) |

改架构时：**同一次 PR** 更新 MODULE_MAP（若动模块）+ 相关 human-docs 节 + 本 README 进度表日期，避免「文档进度不统一」被误当技术债。

### 8.3 写作风格（人类阅读体验）

- 文首：**读者 · 读完能做什么 · 耗时 · 下一篇**（与本页一致）  
- 用 **表格** 列事实；用 **短节** + 小标题；避免单段超过 15 行  
- 状态词统一：`Done` / `OPEN` / `冻结` / `草案` / `已归档`  
- 跨主题：**链接 + 一句**，不粘贴 PLUGIN_V1 / MODULE_MAP 全文  
- **禁止** 把 `handoff/archive/` 或 `04_4.6` 当现行 truth  

### 8.4 PR 里改文档时的自检

- [ ] 只动了一个 SSOT 范围？  
- [ ] human-docs 与 handoff 进度表日期一致？  
- [ ] 英文 human-docs-en 若受影响，至少更新了 README 索引一行？  

---

## 验收

- [ ] 能说出 7 条代码约束各对应哪类 PR 失误  
- [ ] 能说出「改六槽定义去 MODULE_MAP，改 human 入门去 01/06」  
- [ ] 改 DTO 时知道先打开 `dto/`

---

## 深度链接

- [BREAKING_CHANGE_PROCESS](../handoff/BREAKING_CHANGE_PROCESS.md)
- [KERNEL_ERROR_CODE_CONVENTION](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)
- [handoff/README §文档分责](../handoff/README.md)
