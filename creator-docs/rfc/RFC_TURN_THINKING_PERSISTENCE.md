# RFC：Turn Thinking 持久化分流（Fast 短时 / Deep 长时）

| 元数据 | 值 |
|--------|-----|
| 状态 | **Wave E Done · Wave F Done** |
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

---

## 8. Wave E 状态

| 项 | 状态 |
|----|------|
| `fast_persistence` `legacy` \| `strong_only` | **Done** |
| `TurnThinkingPlan` 持久化 API | **Done** |
| 接入 `co_present` / `post` | **Done** |

---

## 9. 规则信号 SSOT（Wave F · Deep 路由）

角色包 `config.json` → `turn_thinking.deep_when` 与 Host 默认 OR 合并（`effective.or = host.or ++ pack.or`；`pack.and` 追加）。**无 UI 开关**。

| signal | 输入 | Host 默认 OR |
|--------|------|--------------|
| `long_message` | `min_chars`（默认 80） | 是 |
| `high_arousal` | anger≥0.45 \|\| sadness≥0.45 \|\| fear≥0.4 | 是 |
| `high_sadness` / `high_anger` / `high_fear` | 分项（包 AND 用） | 否 |
| `this_turn_event` | 规则 `EventDetector` **本句**（Router **前** prepass） | **Quarrel, Confession** |
| `recent_event` | `get_events(role, 8)` | Quarrel |
| `keyword` | 子串 | 认真 / 很重要 / 别敷衍 |
| `deep_latch_active` | DB 位 | 否（latch 激活后命中） |

**AND 组**：`{ "all": [ signal, ... ] }` — 组内全 true → Deep。

实现：`turn_thinking.rs` · `evaluate_policy` · `co_present` 本句 rule event 提前。

---

## 10. Deep latch 状态机

配置：`config.json` → `turn_thinking.latch`（角色包；Host 无默认）。

| 转换 | 条件 | DB |
|------|------|-----|
| **enter** | `this_turn_event` ∈ `enter_on` | `deep_latch_active = 1` |
| **active** | `deep_latch_active = 1` | 每轮 Auto 路由 **恒 Deep**（直至 exit） |
| **exit** | `this_turn_event` ∈ `exit_on` | `deep_latch_active = 0` |

典型：`enter_on: [Quarrel]` · `exit_on: [Apology]` — 大事件后持续 Deep 直到和解。**不做**「连 Fast N 轮强制 Deep」。

迁移：`035_turn_thinking_runtime.sql` → `role_runtime.deep_latch_active`。

---

## 11. ephemeral_archive 与 mutable 边界

| 层 | 来源 | 寿命 | 写入 |
|----|------|------|------|
| 核心 | `core_personality.txt` | 包固定 | 创作者 |
| 可变 | `role_runtime.mutable_personality` | 长期（profile 模式） | 主链 LLM / 演化 |
| **临时** | `role_runtime.ephemeral_personality` | **TTL 轮数** | **规则模板**（无主链 LLM） |

配置：`turn_thinking.ephemeral_archive` — `enabled` · `ttl_turns`（1–8，默认 3）· `max_chars`（默认 200）· `update_on_events`。

每轮 post 末尾：`ttl = max(0, ttl−1)`；0 清空 text；命中 `update_on_events` 或 latch enter/exit 时写入局面摘要并重置 TTL。

Prompt：`【局面摘要】` 注入 `build_personality_supplement` **之后**、语气区块之前；**Fast + Deep** 均注入。

与 mutable **独立**；冲突时以核心档案为准（Prompt 段首声明）。

---

## 12. 合并与默认行为

```rust
fn effective_turn_thinking_policy(host: &HostProfile, role: &Role) -> TurnThinkingPolicy;
```

| 场景 | 行为 |
|------|------|
| 无 `config.json` → `turn_thinking` | Host 默认 OR（含 `this_turn_event` Quarrel/Confession） |
| 有节 | `or = host.or ++ pack.or`；`and += pack.and`；latch / ephemeral 取自包 |
| Deep | 绑定 Wave E 全量 persistence（`applies_full_persistence` 不变） |

校验：`oclive pack validate` · signal 枚举 · TTL 1–8 · `max_chars` 上限 — 见 [ROLE_PACK_SPEC.md §9.11](../role-pack/ROLE_PACK_SPEC.md#911-turn_thinkingwave-f).

**明确不做**：`remote_life` · 主链 LLM 写 ephemeral · 运行时 prompt 压缩 · 六槽 / 玩家 Deep/Fast UI · pack-editor UI（**PE-TURN-01** 不阻塞 MVP）。
