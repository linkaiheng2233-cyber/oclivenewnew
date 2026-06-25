# Deep 路径 · Prompt 蒸馏与上下文延续（Wave D）

**状态**：T0 契约 / 愿景（本文）· **未接 Stable 主链**  
**前置**：Wave A/B 已交付（Turn Thinking · 规则 event · TTFT bench），见 [`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md)  
**关联愿景**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) §「TTFT 与 Deep 精炼」

---

## 1. 目标

| 路径 | 现状（2026-06） | 下一档目标 |
|------|-----------------|------------|
| **Fast**（Auto 闲聊） | co-present TTFT p50 **~243ms**（mumu · qwen2.5:7b · `desktop-latency`） | 维持；边际收益小（Ollama 直连下限 ~130ms） |
| **Deep**（高情绪 / 长句 / 事件链） | 仍走 **全量** `PromptBuilder` + 可选 event LLM + 全记忆/知识/场景 enrichment | **离线蒸馏短 prompt（persona capsule）** + **稳定前缀 KV 延续**，压 Deep TTFT 与 prefill 成本 |

**产品原则**：蒸馏产物 **离线生成、包内分发**（编写器 / CLI），**禁止**在 `process_message` 主链再调 LLM 做「运行时压缩」。

---

## 2. 架构归类

**SSOT**：[`MODULE_MAP_AND_HANDOFF.md`](MODULE_MAP_AND_HANDOFF.md) §6（第 3 模块 event + `event_impact_llm`）· §12（Turn Thinking · 编排行策略）· §7（第 4 模块 prompt）。本文只记录 Wave D 设计，**不**维护归类表。

---

## 3. Deep Prompt 蒸馏（T0 契约）

### 3.1 角色包字段（草案）

| 字段 | 位置 | 说明 |
|------|------|------|
| `prompts/deep_capsule.txt` | 包内可选文件 | **Deep 专用**；≤ ~800 汉字（或 ~1.2k tokens）人格胶囊；**替换** Tier0 中全量 `core_personality.txt` 注入，**不**删磁盘上的 `core_personality.txt`（Fast 与编写器仍读全文） |
| `meta.deep_capsule_enabled` | manifest / blueprint meta | 默认 `false`；`true` 且文件存在时 Deep 路径启用 |
| 镜像 | `prompts/deep_capsule.md` | 编写器人类可读镜像（不参与运行时） |

**不变量**：

- `KERNEL_DIALOGUE_GUARDRAILS` **每轮恒追加**，不可被 capsule 替换（与 `reply_quality_anchor` 纪律相同）。
- `reply_quality_anchor`：Deep 仍可用包级锚点；capsule 只承担「人格差异压缩」，不重复 guardrails。
- 校验：`oclive_validation` 待增 `deep_capsule` 长度上限与 UTF-8 检查（T1）。

### 3.2 离线蒸馏流程（编写器 / CLI，T1）

1. **输入**：`core_personality.txt` + 可选 `scenes/` 摘要 + 固定 guardrails 清单（只读参考）。
2. **工具**：pack-editor「生成 Deep 胶囊」或 `oclive-cli pack distill-deep --role mumu`（占位，T1 实现）。
3. **方法**：创作者本地 LLM **一次性**摘要（或人工编辑）；产出写入 `prompts/deep_capsule.txt` + 校验通过。
4. **评测**：固定 OOCP / bench 用例对比 **Full Deep vs Capsule Deep**（人设一致性 checklist + TTFT）。

**禁止**：在 `EventEstimate` / `BuildPrompt` stage 内调用 LLM 动态压缩。

### 3.3 运行时接线（T2 · Stable 主链）

```text
resolve_turn_thinking → Deep
  → co_present: 全量 enrichment（与今一致）
  → PromptInput + HostProfile
  → PromptBuilder:
       if deep_capsule_enabled && mode == Deep:
         build_core_hard_constraint ← deep_capsule（短）
       else:
         build_core_hard_constraint ← core_personality（全文）
  → LLM generate（见 §4 前缀延续）
```

`TurnThinkingPlan` 可增 `use_deep_capsule(role, host) -> bool`（角色包 + 发行版 `[turn_thinking] deep_capsule = true` 可选强制）。

---

## 4. 上下文延续（KV · 继续）

Deep 路径 prompt 长、prefill 重；除缩短 capsule 外，应 **最大化前缀复用**：

| 分段 | 内容 | 回合间是否变化 |
|------|------|----------------|
| **P0 稳定前缀** | guardrails + deep_capsule（或 core）+ 静态角色元数据 | 同角色同会话 **不变** |
| **P1 半稳定** | reply_quality_anchor · user_identity 模板 · 关系基线 | 关系/身份切换时变 |
| **P2 可变后缀** | 记忆检索 · 复杂情感 hint · 场景/state · 用户句 | **每轮**变 |

**实现方向（T2–T3）**：

1. `PromptBuilder` 输出 `{ stable_prefix, mutable_suffix }` 或等价 hash（`SessionCache` 键：`role_id` + `srid` + capsule 版本）。
2. LLM 客户端（Ollama `/api/chat` 或 stream）：同前缀时依赖服务端 **KV cache**；文档化「勿在 P0 段注入时间戳」。
3. 可选：`OCLIVE_PROMPT_PREFIX_CACHE=1` 进程内缓存上一轮的 stable prefix token 长度，bench 对比 prefill ms。
4. **Monolith 态**：宏核焊接路径可进一步内联 prefix buffer（与 [`RFC_OCLIVE_MONOLITH_MODE.md`](../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) 低延迟叙事对齐）。

**与 Fast 的分工**：Fast 已靠 **减 stage + 规则 event** 达标；Deep 靠 **短 capsule + 前缀延续** 控长 prompt 成本，而非再砍 enrichment。

---

## 5. 分阶段交付

| 阶段 | 交付物 | 验收 |
|------|--------|------|
| **T0** | 本文 + 愿景条目 + `DISTRO_CAPABILITY_PROFILE` 开关说明 | 文档评审 |
| **T1** | 角色包 schema · 编写器导出 · `oclive_validation` | mumu 样例 `deep_capsule.txt`；validate 通过 |
| **T2** | `PromptBuilder` Deep 分支 · `co_present` 传 mode · bench | Deep TTFT 较全量下降 ≥20%（同模型同机） |
| **T3** | 前缀分段 + Ollama KV 延续 · `measure-ttft.mjs --deep-only` | 连续 5 轮 Deep，第 2–5 轮 p50 prefill 低于第 1 轮 |

---

## 6. 与 Wave C 边界

| Wave | 内容 | 关系 |
|------|------|------|
| **C** | Chat Pro UI 接 `/chat/stream` | 降低 **用户感知** 延迟；不改变 prompt 体积 |
| **D** | Deep 蒸馏 + 前缀延续 | 降低 **Deep prefill / TTFT** |

二者可并行；TTFT 门禁仍以 [`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md) 脚本为准。

---

## Related

- [`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md)
- [`PERF_PHASES.md`](PERF_PHASES.md)
- [`DISTRO_CAPABILITY_PROFILE.md`](../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)
- [`examples/distro-profiles/desktop-latency.oclive.toml`](../examples/distro-profiles/desktop-latency.oclive.toml)
