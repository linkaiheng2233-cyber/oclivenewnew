# 代码注释英文化 · 执行计划（供 auto 执行）

> 目标：把仓库**代码注释**从中文统一为英文，提升人类（含国际/开源/招聘场景）可读性。
> 本文件是给执行 Agent（auto）的**操作说明书**，请逐条遵守，尤其是「硬性边界」。

---

## 0. 背景与关键事实（先读）

- 项目规模：约 402 个 Rust 文件、260 个 TS/Vue 文件、263 个 Markdown 文档。
- **文档已是中英双语**：`creator-docs/`（中文 SSOT）已有英文镜像 `creator-docs-en/`，且根目录有 `README.en.md` / `CHANGELOG.en.md` / `CONTRIBUTING.en.md`。
  - **因此本计划不大规模翻译文档**，避免与现有英文镜像重复/冲突。
- **真正的英文缺口在「代码注释」**：Rust 的 `//!` / `///` / `//` 与前端 `//` / `/* */` / JSDoc 注释目前为中文，无英文镜像。本计划只处理这部分。

---

## 1. 硬性边界（违反即视为错误）

1. **只改注释与文档字符串，绝不改动任何代码逻辑、标识符、字符串字面量。**
   - 可改：`//!`、`///`、`//`、`/* */`、Rust doc 注释、前端 JSDoc/行内注释。
   - **禁改**：变量名、函数名、类型名、模块名、`const` 值、用户可见文案、i18n 文案、SQL、JSON、测试断言、`tracing!` 里的 `target:` 与字段名、错误码字符串、面向用户的 message 字符串。
2. **不改动 `cfg`、`#[doc]` 属性的语义**；`#![doc = "..."]` 内文本可译，但不得改变属性结构。
3. **不动以下文件/目录**（它们是给开发者/AI 的中文约束或已双语，翻译会有害）：
   - `AGENTS.md`、各子仓 `AGENTS.md`
   - `.cursor/rules/**`
   - `creator-docs/**`（中文 SSOT）与 `creator-docs-en/**`（已英文）
   - `handoff/**`（内部交接，含本文件）
   - 根 `README.md` / `CHANGELOG.md` / `CONTRIBUTING.md`（中文版，已有 `.en` 镜像）
   - `dev-notes/roadshow/项目说明.md`、`通知.markdown`、`handoff/archive/ARCHIVE_PROJECT_HISTORY.md`
   - 任何 `roles/**` 内的角色包内容文件（人设文本等）
4. **doc 注释里的代码链接不要破坏**：如 `[`SlotResolver`](../slot_resolver.rs)`、`[`crate::xxx`]` 这类 intra-doc link 的**路径与符号保持原样**，只翻译周围的中文描述。
5. **保留 Markdown/格式**：注释里的 `**加粗**`、反引号代码、列表、代码块原样保留，只替换中文文字。
6. **不确定就跳过并记录**：若某条注释含义不清、或涉及可能改变行为的内容，**保留原中文**并在最终报告里列出待人工确认，**不要瞎译**。
7. **每个阶段结束必须验证**（见第 6 节），验证不过不得进入下一阶段。

---

## 2. 术语对照表（务必统一，保证全仓一致）

> 优先与 `creator-docs-en/` 中已有英文译法保持一致；下表为基线，遇冲突以 `creator-docs-en/` 为准。

| 中文 | 英文（统一用法） |
|------|------------------|
| 契约型薄核 / 薄内核 | contract-based thin kernel |
| 纯净内核 | pure kernel |
| 外核 | exokernel |
| 角色包 | role pack |
| 蓝图 | blueprint |
| 宿主 | host |
| 发行版 | release / distribution |
| 单核双态 | single-core dual-state |
| 双核（运行时） | dual-core (runtime) |
| 稳定核 | stable core |
| 实验核 | experimental core |
| 高耦合 / 焊接模式 | high-coupling / Monolith (weld) mode |
| 七焊接键 | seven weld keys |
| 槽位 / 槽 | slot |
| 槽位注册表 | slot registry |
| 多实例槽位 | multi-instance slot |
| 合并策略 | merge policy |
| 串行 last-wins | serial last-wins |
| 并发/并行 | concurrent / parallel |
| 共景 | co-present |
| 异地 | remote-presence |
| 编排 | orchestration |
| 主编排入口 | main orchestration entry |
| 情绪分析 | emotion analysis |
| 复杂情感 | complex emotion |
| 叙事提示 | narrative hint |
| 事件估计 | event estimate |
| 记忆检索 | memory retrieval |
| 长期/短期记忆 | long-term / short-term memory |
| 记忆回放 | memory replay |
| 去重合并 | dedup-merge |
| 好感（度） | favorability |
| 关系阶段/状态 | relation state |
| 性格向量 | personality vector |
| Prompt 组装器 | prompt assembler |
| 回合 | turn |
| 回滚快照 | rollback snapshot |
| 静默降级 | silent fallback / graceful degradation |
| 目录插件 | directory plugin |
| 侧车 | sidecar |
| 后端模块 | backend module |
| 权限授权 | permission grant |
| 启动健康检查 | startup health check |
| 混合存储 | hybrid storage |
| 文件镜像 | file mirror |
| 脚手架 | scaffold |
| 内核工厂 | kernel factory |

风格要求：
- 注释用**简洁、规范的技术英文**；术语首次出现可保留原结构（如 `` `slot_registry` ``）。
- doc 注释（`///`、`//!`）尽量符合 rustdoc 习惯：首句为简短摘要，空行后展开。
- 不逐字硬译，以**传达原意**为先；保留原注释里的「为何这样做/局限」这类设计意图。

---

## 3. 分阶段顺序（按「人类读代码的主链路」优先）

> 每个文件的操作见第 5 节。完成一个阶段→验证→提交→下一阶段。

> **Note (2026-06)**: Paths below live under **`crates/oclive_kernel_host/src/domain/`** (migrated from legacy `src-tauri/src/domain/`). Comment migration targets **`kernel_host`**, not the Tauri shell.

### 阶段 A：后端主链路模块顶部 `//!` 与公共 `///`
优先级最高，是读代码的人最先看的「地图」。建议顺序：

1. `crates/oclive_kernel_host/src/domain/chat_engine/mod.rs`（主编排入口 `process_message`）
2. `crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/`（`mod.rs`、`common.rs`、`co_present.rs`、`context.rs` 等）
3. `crates/oclive_kernel_host/src/domain/slot_runner.rs` 及 `slot_runner/llm_merge.rs`
4. `crates/oclive_kernel_host/src/domain/slot_resolver.rs`
5. `crates/oclive_kernel_host/src/domain/dual_pipeline.rs`、`dual_pipeline_steps.rs`、`dual_pipeline_registry.rs`
6. `crates/oclive_kernel_runtime/src/domain/complex_emotion.rs`、`complex_emotion_store.rs`
7. `crates/oclive_kernel_runtime/src/domain/prompt_builder/mod.rs`、`sections.rs`、`prompt_assembler.rs`
8. `crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs`、`emotion_analyzer.rs`、`event_estimator.rs`
9. `src-tauri/src/state/mod.rs`、`state/app_state_builder.rs`
10. `crates/oclive_kernel_host/src/domain/startup_health.rs`、`error.rs`

### 阶段 B：后端其余 `domain/` 与 `infrastructure/`
- `src-tauri/src/infrastructure/**`（`db/**`、`chat_storage/**`、`mcp_client.rs`、`remote_plugin/**` 等）
- `crates/oclive_kernel_host/src/domain/ports/plugin_host/`、`agent.rs`、其余 `domain/*.rs`

### 阶段 C：后端 API 层与其余
- `src-tauri/src/api/**`
- `src-tauri/src/models/**`、`src-tauri/src/lib.rs`、`main.rs`
- `crates/**`（`oclive-cli`、`oclive_kernel_runtime` 等）的注释

### 阶段 D：前端核心
1. `src/stores/**`（`chatStore.ts`、`roleStore.ts` 等）
2. `src/composables/**`
3. `src/api/**`、`src/utils/**`、`src/lib/**`
4. `src/components/**`、`src/views/**`（`.vue` 文件 `<script>` 内注释）

> i18n 文案文件 `distros/shared/src/i18n/**` 属于「用户可见文案」，**不在范围内**，跳过。

---

## 4. 如何找到中文注释（检索方法）

用 ripgrep 按 Unicode 中文区间检索（只在代码目录）：

```
rg -n "[\x{4e00}-\x{9fff}]" src-tauri/src --glob "*.rs"
rg -n "[\x{4e00}-\x{9fff}]" src --glob "*.ts" --glob "*.vue"
rg -n "[\x{4e00}-\x{9fff}]" crates --glob "*.rs"
```

对每个命中行，判断它是否在**注释**里：
- 是注释 → 按术语表译为英文。
- **不是注释**（是字符串字面量、i18n、用户文案、测试数据、SQL 等）→ **跳过，不动**。

---

## 5. 单文件操作流程

1. 读取文件，定位所有**中文注释**行（区分注释与字符串字面量）。
2. 逐条把中文注释替换为英文（用 StrReplace，保留缩进与 Markdown 结构）。
3. 不改任何代码 token；intra-doc link 路径/符号保持原样。
4. 含义不清的，保留中文并记入「待确认清单」。

---

## 6. 验证（每阶段结束必须做）

后端（阶段 A/B/C 之后）：

```
cd src-tauri
cargo check -p oclivenewnew-tauri
cargo check -p oclivenewnew-tauri --features dual_core
```

> 仅改注释，理论上恒过；若失败，说明误删了 `//!`/`///` 影响了 doc 属性或误碰代码，须回退该处。
> 如需校验 doc 链接：`cargo doc --no-deps -p oclivenewnew-tauri`（可选）。

前端（阶段 D 之后）：

```
npm run build
npm run test:unit
```

每阶段验证通过后，按逻辑分组提交（commit 信息示例）：

```
docs(comments): translate <module/area> comments to English (no logic change)
```

---

## 7. 明确不做（避免过度工程）

- 不翻译 `creator-docs/**`、`handoff/**`、`AGENTS.md`、`.cursor/rules/**`、中文 README/CHANGELOG/CONTRIBUTING（已有英文镜像或属内部约束）。
- 不动 i18n 用户文案、用户可见字符串、错误码字符串、SQL、测试数据。
- 不顺手重命名、不顺手重构、不调整代码结构。
- 不为「行数/风格统一」改动无中文注释的文件。
- 不新增大量注释；只把**已有中文注释**转英文（缺注释的关键公共 API 可酌情补一句英文摘要，但不强求、不铺开）。

---

## 8. 交付物

1. 全仓代码注释（按阶段）英文化，且 `cargo check`（含 `--features dual_core`）与 `npm run build` / `test:unit` 全绿。
2. 一份「待确认清单」：列出因含义不清而保留中文的注释位置（文件:行）。
3. 分阶段的提交记录。

---

## 9. 执行状态（2026-06-01）

**已完成（已提交）**

| 范围 | 提交示例 |
|------|----------|
| 根目录文档整理 + 冻结标注 | `533bb0e2` |
| `src-tauri` 注释（infra/domain/api/叶子） | `f11cf7a2` … `e2f8b7b8` |
| `oclive-cli` + 部分 kernel crates | `ec74fd17`, `fc5f8a8d` |
| `kernel_types` / `kernel_runtime` / `validation` | `95379e7f` |
| 前端 TS/Vue（更早批次） | `cedc66db` 等 |

**验证**：`cargo check --workspace` 绿；`src/` 前端无中文注释行。

**刻意保留中文的注释行（非漂移，勿再改）**

| 位置 | 原因 |
|------|------|
| `src-tauri/.../function_call_parser.rs` `///` 示例 JSON 含 `深圳` | 文档内嵌示例数据，与测试用例一致 |
| `crates/oclive-cli/.../monolith_codegen.rs` `r#"..."#` 生成模板内的 `//` 行 | 写入生成文件的模板文案，非维护者注释 |
| `oclive_kernel_contracts` 部分 `///` 代码示例（`你好`、`查一下北京天气` 等） | doc 内嵌**示例字符串字面量**，非注释正文 |
| `oclive_kernel_runtime/.../emotion_analyzer.rs` 示例输入 `"我很开心"` | 同上 |
| `profile_personality.rs` 注释中的 `重要记忆` 等小节名 | 与运行时 `const` 段标题字符串一致 |

**仍含中文的代码（预期，不在本计划范围）**

- LLM Prompt 模板字符串（`prompt_builder/mod.rs`、`sections.rs` 等）
- CLI 用户可见输出、clap `help`/`about`
- 校验/错误/日志/测试数据中的中文字符串
- i18n、`roles/**` 人设文本

---

*本计划仅约束代码注释英文化；业务功能、契约语义、用户可见文案一律不变。*
