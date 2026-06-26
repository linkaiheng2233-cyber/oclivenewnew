# RFC：Turn Thinking 持久化分流（Fast 短时 / Deep 长时）

| 元数据 | 值 |
|--------|-----|
| 状态 | **草案 · Wave E 实现中** |
| 受众 | 内核 / 发行版 / Cursor |
| 前置 | [MODULE_MAP §12](../../handoff/MODULE_MAP_AND_HANDOFF.md) · [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) · [DISTRO_CAPABILITY_PROFILE.md](../kernel/DISTRO_CAPABILITY_PROFILE.md) §3.2.1 |
| Bench | [TTFT_BENCHMARK.md](../../handoff/TTFT_BENCHMARK.md) Wave E · [DEEP_PROMPT_DISTILLATION.md](../../handoff/DEEP_PROMPT_DISTILLATION.md) |

---

## 1. 产品原则

| 存储层 | Fast 闲聊（`strong_only`） | Fast + 强事件 | Deep |
|--------|------------------------------|---------------|------|
| **聊天 turns**（`chat_storage`） | **每轮仍写** | 写 | 写 |
| **长期记忆**（`long_term_memory`） | **跳过**（`memory_importance = 0`） | 正常 | 正常 |
| **好感 / 关系** | **Δ ≈ 0** | 正常 | 正常 |
| **性格 / mutable profile 演化** | **跳过** | 正常 | 正常 |

**认知模型**：聊天 turns = 人类「工作记忆」（UI 可见）；长期记忆 / 好感 / 性格 = 「长时巩固」。Fast 闲聊在 `strong_only` 下不巩固，强关系事件仍巩固。

**默认**：`fast_persistence = "legacy"`（与 Wave A–D 行为一致）；bench profile `desktop-latency` 先开 `strong_only`，验收后合入 `desktop`。

---

## 2. 架构边界

| 约束 | 做法 |
|------|------|
| 编排 | 逻辑在 `turn_pipeline/*` + `turn_thinking.rs`；**不改** `process_message` 顶层 |
| DTO | **不改** `SendMessageResponse`；可选 `OCLIVE_BENCH_TELEMETRY=1` tracing |
| 六槽 | Turn Thinking **不是**第七槽；**不改** `slot_registry` |
| remote_life | **本 Wave 不改**（co-present only） |

---

## 3. 强事件白名单（SSOT）

与 `EventType` 对齐（`event_detector.rs` / `oclive_kernel_types`）：

| 强事件 | 说明 |
|--------|------|
| `Quarrel` | 争吵 |
| `Apology` | 道歉 |
| `Confession` | 告白 |
| `Praise` | 表扬 |

**非强事件**（Fast + `strong_only` 下不巩固）：`Ignore` · `Joke` · `Complaint` 等。

Quarrel 链仍触发 **Deep** 路由（全量 persist），与强事件白名单独立。

---

## 4. HostProfile 配置

`[turn_thinking]` 新增字段：

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `fast_persistence` | `"legacy"` \| `"strong_only"` | **`legacy`** | `legacy` = 现行为；`strong_only` = Fast 仅强事件巩固 |

环境变量（bench 可选）：`OCLIVE_FAST_PERSISTENCE=strong_only`（与 `OCLIVE_PROMPT_PREFIX_CACHE` 同级覆盖）。

---

## 5. 持久化策略 API（`TurnThinkingPlan`）

```rust
fn applies_full_persistence(&self, host: &HostProfile, event: &EventType) -> bool;
fn favor_delta_scale(&self, host: &HostProfile, event: &EventType) -> f64;
fn memory_importance_after_policy(&self, host: &HostProfile, event: &EventType, raw: f64) -> f64;
fn skip_mutable_profile_evolution(&self, host: &HostProfile, event: &EventType) -> bool;
```

| 模式 | `applies_full_persistence` |
|------|---------------------------|
| **Deep** | 始终 `true` |
| **Fast + legacy** | 始终 `true` |
| **Fast + strong_only + 强事件** | `true` |
| **Fast + strong_only + 其他** | `false` |

接入点：`co_present`（favor scale · `PersonalityEngine::evolve_by_event` gate）· `post`（`memory_importance` · `spawn_profile_evolution` gate）· `MiddleOutput.turn_thinking`。

现有杠杆：`chat_turn_atomic` 在 `memory_importance <= 0` 时跳过 `long_term_memory` insert（无需新表/迁移）。

---

## 6. 与聊天存储的边界

详见 [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)：本 RFC **只**分流 long_term / favor / evolution；**不**影响 `chat_storage` 每轮写入。

---

## 7. Breaking 语义

属**用户可感知行为变更**（Fast 闲聊不涨好感/不进长期记忆），**非** wire Breaking。见 [BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md) · CHANGELOG `[Unreleased]`。旧 session 数据不回滚。
