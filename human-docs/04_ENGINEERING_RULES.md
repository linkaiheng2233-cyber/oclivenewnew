# 04 · 工程约束（7 条）

> **读者**：准备提内核 PR 的工程师。  
> **读完能做什么**：避免 review 高频打回项（编排位置、DTO 字段、Prompt、`import` 路径）。  
> **耗时**：约 25 分钟。  
> **下一篇**：[05 调试](05_DEBUGGING.md)。

与 [`.cursor/rules/oclivenewnew.mdc`](../.cursor/rules/oclivenewnew.mdc)、[CONTRIBUTING.md §工程约束](../CONTRIBUTING.md#工程约束) **三处镜像**；变更时须同 PR 同步。

---

## 1. 编排只在 `process_message` / `*_engine`

**规则**：对话主流程在 [`process_message.rs`](../crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)；业务公式留在各 `*_engine` / analyzer。**API 层不堆业务**。

**违反时 review 常见意见**：「请把逻辑下沉到 `domain/`，`api/*.rs` 只做参数校验与 `*_impl` 委托。」

---

## 2. 持久化走 repository trait + 迁移 SSOT

**规则**：通过 `domain/repository.rs` trait 与 `infrastructure/repositories.rs` 实现；表结构以 [`crates/oclive_kernel_host/migrations/001_init.sql`](../crates/oclive_kernel_host/migrations/001_init.sql) 为准。

**违反时**：「表名/列名与迁移不一致」「禁止虚构 `memory_backend` 表」等。

---

## 3. Tauri 命令注册纪律

**规则**：命令只在 [`src-tauri/src/api/*.rs`](../src-tauri/src/api/)；仅在 [`lib.rs`](../src-tauri/src/lib.rs) 用 `tauri::generate_handler!` 注册。

**违反时**：「请勿在 `lib.rs` 写业务」「新命令缺前端 `src/api/` camelCase 封装」。

---

## 4. DTO 契约

**规则**：前后端以 [`oclive_kernel_types/src/models/dto.rs`](../crates/oclive_kernel_types/src/models/dto.rs) 为准；回复字段 **`reply`**，不是 `response`；`Emotion` 以 [`emotion.rs`](../crates/oclive_kernel_types/src/models/emotion.rs) 为准（无未定义变体）。

**违反时**：「请改为 `reply`」「`Joy`/`Fearful` 不在契约枚举内」。

---

## 5. PromptBuilder 签名

**规则**：[`PromptBuilder::build_prompt`](../crates/oclive_kernel_runtime/src/domain/prompt_builder/mod.rs) 单参数 `input: &PromptInput<'_>`，返回 **`String`**（**不是 `Result`，不要用 `?`**）。

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

## 验收

- [ ] 能说出 7 条各对应哪类典型 PR 失误
- [ ] 改 DTO 时知道先打开 `dto.rs`

---

## 深度链接

- [BREAKING_CHANGE_PROCESS](../handoff/BREAKING_CHANGE_PROCESS.md)
- [KERNEL_ERROR_CODE_CONVENTION](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)
