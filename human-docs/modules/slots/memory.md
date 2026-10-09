# 六槽开工包 · `memory`

> **读者**：编写基础 Memory 实现，或维护参考 Host 的记忆检索、STM/LTM 策略和后端的工程师。
> **读完能做什么**：选择自己的接入路径；基础实现者能运行已有消费者案例，参考 Host 维护者能区分聊天日志/STM/LTM 与回想流程。
> **耗时**：基础案例约 **5–10 min**；参考 Host 开工包约 **50 min**
> **SSOT 范围**：人类开工路由与 checklist；公共接入点见 [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，参考 Host 定义见 [§4](../../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)
> **最后更新**：2026-10-09
> **下一篇**：[chat-storage](../side-channels/chat-storage.md) · [side-channels 索引](../side-channels/)

---

## 0. 先选接入路径

| 你要做什么 | 从哪里开工 |
|---|---|
| 写一个通过现有公共接口被调用的基础检索实现 | 下方 **MemoryBase 案例**；按调用方提供的材料和资源运行 |
| 改参考 Host 的丰富记忆后端、持久化或回想策略 | **§1–§6**；沿已有 `MemoryRetrieval`、配置和存储接线 |

### MemoryBase 案例：先跑通，再替换自己的实现

1. 打开作者自己的 [`memory.rs`](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs)，读它的检索约定与 `impl MemoryBase`；再打开 [`main.rs`](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs)，看 `MinimalRoleBaseBindings.memory` 如何借用该实现，以及 `MemoryBase::retrieve(&consumer, request)` 如何通过已有消费者调用。
2. 在仓库根运行下面两个命令。这里复用已验证的离线命令，要求本机已准备 Rust 依赖缓存；缺缓存时先完成依赖准备。案例的实现与 runtime 库分开编译；无需先配置蓝图、STM/LTM、模型或启动参考 Host。

   ```powershell
   cargo run --locked --offline -p oclive_kernel_runtime --example external_memory_base
   cargo test --locked --offline -p oclive_kernel_runtime --example external_memory_base
   ```

3. 现有案例运行输出为 `external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0`。它比较同一批材料下的两个实现，各自使用自己的检索约定；新实现选最近两条，已有关键词实现选三条。`literal:<exact substring>` 属于这个作者的实现约定，不是 Base 规定的通用查询语法。
4. 编写自己的实现时，先替换算法和对应的调用要求，再用消费者调用核对实际输出、正常空结果与完整失败。资源、材料来源与异步执行器由调用方提供；本例只驱动立即就绪的内存实现，不能把其中的 `immediate` helper 当作任意异步实现的执行器。

**本路径的验收 checklist**：

- [ ] 实现说明了自己接受的查询与材料约定；未支持的要求保留完整失败。
- [ ] 实际通过现有消费者调用，正常空结果和失败能区分；不把“调用成功”当作“找到了所有相关事实”。
- [ ] 来源与资源范围由调用方明确，未隐式读取旧材料、数据库或取得新权限。

公共请求、失败与资源绑定仍查 [接入点清单](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。本例只调用 Memory；绑定六个引用不要求六槽每轮全部执行。它也没有将自定义实现注册到参考 Host 的蓝图后端，进入该路径仍需按下面的实际接线处理。

---

## 1. 参考 Host：你插在哪

- **MODULE_MAP**：[§4 第 1 模块 · `memory`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)
- **当前配置 / 运行时折叠**：蓝图 `slot_registry.type: memory` → `PluginBackends.memory`；legacy v1 才是 `settings.json.plugin_backends.memory`
- **Trait**：`MemoryRetrieval`（`oclive_kernel_contracts`）
- **主链 hook**：memory 槽在 `turn_pipeline/pre.rs` 检索；参考 Host 在 `post_llm` 写入 STM/LTM，写入不属于 `MemoryRetrieval` trait
- **Event Ring hook**：`co_present/run_middle.rs` 从本轮相关记忆选候选 → `memory.recall.candidate` → `memory.recollection.activated`（可能无）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 检索算法、decay、archive 阈值、`remote` / `directory` / `local` 协议 | 用 `chat_messages` 当真源；删聊天清记忆表 |
| `BuiltinMemoryRetrieval` + `MemoryEngine` 参数 | 角色任务改 `slot_registry`（G1） |
| 多 memory 实例 **去重合并** 检索；`none` 时按契约返回空列表 | 把 `none` 写成回退 `builtin`，或误称共景禁用；权威语义见 [MODULE_NONE_SEMANTICS](../../../creator-docs/kernel/MODULE_NONE_SEMANTICS.md) |

**三套存储**：聊天日志 ≠ `short_term_memory` ≠ `long_term_memory` — 深读 [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md)，勿与本包重复整表。

memory 只有候选提案权：事件信封只携带记忆 ID 与分数，决策模块采纳后，正文才在 Ring 外按 ID 回取并进入单轮回想 Prompt。memory 不能自行宣布“角色已经想起”。完整边界见 [EVENT_RING](../../../creator-docs/plugin-and-architecture/EVENT_RING.md)。

---

## 3. 阅读清单

1. [MODULE_MAP §4](../../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)
2. [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md)
3. [01 简架构 §记忆三套](../../01_ARCHITECTURE_SIMPLE.md#记忆三套存储最易混--必背)
4. [CROSS_HOST_MEMORY](../../../creator-docs/role-pack/CROSS_HOST_MEMORY.md)（跨宿主时）
5. 迁移 SSOT：`kernel/crates/oclive_kernel_host/migrations/001_init.sql` — `short_term_memory` · `long_term_memory`
6. [EVENT_RING](../../../creator-docs/plugin-and-architecture/EVENT_RING.md) — 回想提案、采纳与隐私边界

---

## 4. 开发流程

- [ ] 说清本改动影响 ② STM / ③ LTM 哪一侧（不进 ① 聊天日志）
- [ ] 改检索 → `pre.rs` / `MemoryEngine`；改写入 → `post_llm` / archive 策略
- [ ] 换 backend → 蓝图 + [SLOT_BACKEND_REALITY_MATRIX](../../../handoff/SLOT_BACKEND_REALITY_MATRIX.md)
- [ ] 「记忆回放」走 `replay_memory_extraction`，不覆盖 LTM 全文
- [ ] domain 单测优先于端到端
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] UI 删聊天记录后 STM/LTM 仍按设计保留或衰减
- [ ] Prompt 中记忆段落来自 memory 槽检索，非直接拼 chat 行
- [ ] 多 memory 实例检索去重合并
- [ ] Fast Turn Thinking `strong_only` 时持久化行为符合 RFC（链 [orchestration/turn-thinking](../orchestration/turn-thinking.md)）
- [ ] 未采纳回想时仍沿用普通 memory Prompt；事件中不复制记忆正文

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `prompt` | 检索结果注入 `PromptInput` |
| `emotion` / `event` | emotion 同在 pre 提供用户情绪；event 在 middle 先规则初估、再按 Turn Thinking 策略可选调用槽位 |
| `chat-storage` | ① 聊天日志独立通道；回放合并进 LTM |
| Turn Thinking | Fast 档可能跳过部分 LTM 写入（HostProfile） |
