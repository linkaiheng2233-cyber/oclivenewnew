# Deep 路径 · Prompt 蒸馏与上下文延续（Wave D）

**状态**：T1/T2/T3 **已接 Stable 主链**（角色包资产 · `PromptBuilder` Deep 分支 · Ollama 前缀缓存）  
**前置**：Wave A/B 已交付（Turn Thinking · 规则 event · TTFT bench），见 [`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md)  
**关联愿景**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) §「TTFT 与 Deep 精炼」

---

## 1. 目标

| 路径 | 现状（2026-06） | 下一档目标 |
|------|-----------------|------------|
| **Fast**（Auto 闲聊） | co-present TTFT p50 **~243ms**（mumu · qwen2.5:7b · `desktop-latency`） | 维持；边际收益小（Ollama 直连下限 ~130ms） |
| **Fast 巩固**（Wave E） | Fast 闲聊仍写 favor / long_term（`legacy`） | **`strong_only`**：闲聊不写 long_term / favor / evolution；强事件仍写 · RFC [`RFC_TURN_THINKING_PERSISTENCE.md`](../rfc/RFC_TURN_THINKING_PERSISTENCE.md) |
| **Deep**（高情绪 / 长句 / 事件链） | 仍走 **全量** `PromptBuilder` + 可选 event LLM + 全记忆/知识/场景 enrichment | **离线蒸馏短 prompt（persona capsule）** + **稳定前缀 KV 延续**，压 Deep TTFT 与 prefill 成本 |

**产品原则**：蒸馏产物 **离线生成、包内分发**（编写器 / CLI），**禁止**在 `process_message` 主链再调 LLM 做「运行时压缩」。

---

## 2. 架构归类

**SSOT**：[`MODULE_MAP_AND_HANDOFF.md`](MODULE_MAP_AND_HANDOFF.md) §6（第 3 模块 event + `event_impact_llm`）· §12（Turn Thinking · 编排行策略）· §7（第 4 模块 prompt）。本文只记录 Wave D 设计，**不**维护归类表。

---

## 3. Deep Prompt 蒸馏

### 3.0 资产矩阵（model_tier × Turn Thinking）

| model_tier | Fast 轮 | Deep 轮 |
|------------|---------|---------|
| **Small**（≤13B · `7b`/`8b`/`13b` 启发式） | **persona capsule**（沿用 `deep_capsule` 文件名）替代 Tier0 全量注入 | **persona capsule** + Deep enrichment |
| **Large**（34B+ 启发式） | 全量 `core_personality` | 全量 `core_personality` + 全 enrichment（**KV 前缀延续**见 §4 · 第 7 月+ backlog） |

编排行 SSOT：`ModelTier` / `PersonaSource` — [`MODULE_MAP_AND_HANDOFF.md`](MODULE_MAP_AND_HANDOFF.md) §12 · 实现 `model_tier.rs` + `resolve_persona_source`。

### 3.1 角色包字段

| 字段 | 位置 | 说明 |
|------|------|------|
| `prompts/deep_capsule.txt` | 包内可选文件 | ≤ **2500** 汉字离线人格胶囊；Small 模型 Fast/Deep 均可**替换** Tier0 全量 `core_personality.txt` 注入；**不**删除磁盘上的 `core_personality.txt`（Large 模型与编写器仍读全文） |
| `meta.deep_capsule_enabled` | blueprint `meta` | 默认 `false`；`true` 且文件存在且模型为 Small 时启用（字段名为兼容保留） |
| 镜像 | `prompts/deep_capsule.md` | 编写器人类可读镜像（不参与运行时） |

**不变量**：

- `KERNEL_DIALOGUE_GUARDRAILS` **每轮恒追加**，不可被 capsule 替换（与 `reply_quality_anchor` 纪律相同）。
- `reply_quality_anchor`：Deep 仍可用包级锚点；capsule 只承担「人格差异压缩」，不重复 guardrails。
- 校验：`oclive_validation` UTF-8 · ≤2500 字；`enabled` 但缺文件 → **ERROR**（T1 已交付）。

### 3.2 人设一致性 checklist（Full Deep vs Capsule Deep）

固定 OOCP / bench 对比时逐条核对（mumu 样例）：

1. 身份：米黄色头发、个子小小的**可爱小女孩**沐沐（非御姐/职场大人）。
2. 关系：与用户同住屋檐下（妹妹+室友感），非亲兄妹。
3. 嘴硬根源：**害羞**而非恶意怼人；关心藏在行动与琐碎里。
4. 情感表达：说不出口「需要你/想你」→ 用行动或转移话题掩盖。
5. 称呼：可轮换「你/喂/笨蛋」，高好感后偶现亲昵称呼并立刻吐槽找补。
6. 场景不变性：学校/家/公司/游乐园/VS Code 等仅改「在做什么」，不改本质人设。
7. 长度：日常 1–3 句；深聊 4–5 句；纯文字无 Markdown。
8. 禁止：替用户做决定；低落时轻浮；固定开场复读；恶意欺负陌生人。
9. 甜食/番剧/小动物：自然提及，不每轮推销。
10. 低/中/高好感语气梯度与全量 Deep 一致（防御 → 松动 → 偶发半句真话）。

### 3.3 离线蒸馏流程（编写器 / CLI，T1）

1. **输入**：`core_personality.txt` + 可选 `scenes/` 摘要 + 固定 guardrails 清单（只读参考）。
2. **工具**：pack-editor「生成 Deep 胶囊」或 `oclive-cli pack distill-deep --role mumu`（占位，T1 实现）。
3. **方法**：创作者本地 LLM **一次性**摘要（或人工编辑）；产出写入 `prompts/deep_capsule.txt` + 校验通过。
4. **评测**：固定 OOCP / bench 用例对比 **Full Deep vs Capsule Deep**（人设一致性 checklist + TTFT）。

**禁止**：在 `EventEstimate` / `BuildPrompt` stage 内调用 LLM 动态压缩。

### 3.4 运行时接线（T2 · Stable 主链）

```text
resolve_turn_thinking → Fast / Deep
  → resolve_model_tier(ollama_model) + resolve_persona_source(tier, role, host)
  → co_present: 按 Fast / Deep 保持各自 enrichment 策略
  → PromptInput { persona_override: Option<&str> } + HostProfile
  → PromptBuilder:
       if PersonaSource::PersonaCapsule:
         build_core_hard_constraint ← deep_capsule（短）
       else:
         build_core_hard_constraint ← core_personality（全文）
  → LLM generate（见 §4 前缀延续）
```

`resolve_persona_source`：`Small + deep_capsule_enabled + 文件存在` → `PersonaCapsule`，Fast/Deep 共用；Large 模型保持 `FullCore`。发行版 `[turn_thinking] deep_capsule` 可强制开/关（字段名兼容保留，见 DISTRO_CAPABILITY_PROFILE）。

---

## 4. 上下文延续（KV · T3 实现）

Deep 路径 prompt 长、prefill 重；除缩短 capsule 外，应 **最大化前缀复用**。

**T3 采用（非 Ollama deprecated `context` 数组）**：`build_prompt_segments` 将 **稳定字节前缀** 排在字符串最前；同模型 + `keep_alive` 下依赖 **llama.cpp 字节级前缀 KV 复用**；用 `prompt_eval_duration` 观测 prefill 下降。

| 分段 | 内容 | 回合间是否变化 |
|------|------|----------------|
| **stable_prefix** | Tier0（capsule/core）· 世界观 · `reply_quality_anchor` · `KERNEL_DIALOGUE_GUARDRAILS` · 场景约束 | 同角色同场景同 persona **不变** |
| **dynamic_suffix** | 人格补充 · 状态/关系/事件 · CE hint · 记忆 · 用户身份 · 日程 · 用户句 | **每轮**变 |

**运行时开关**：`[turn_thinking] prompt_prefix_cache = true` 或 `OCLIVE_PROMPT_PREFIX_CACHE=1`；Fast/Deep 均可；且有效 LLM 为 Ollama、prompt 后端为内置实现。目录/远程 prompt 后端继续走自身 `build_prompt`，不得为缓存绕过插件契约。`SessionCache` 键 `srid:model:mode:persona:scene:user_identity` 仅用于遥测「预期命中」，**不**存 Ollama context。

**P0 禁令**：stable 段内不得含虚拟时间戳、轮次号、随机 token。

**Bench**：`OCLIVE_BENCH_TELEMETRY=1` + `node scripts/measure-ttft.mjs --deep-multi --profile desktop-latency`；门禁：连续 5 轮 Deep，round 2–5 `prompt_eval_ms` p50 &lt; round 1。

---

## 5. 分阶段交付

| 阶段 | 交付物 | 验收 |
|------|--------|------|
| **T0** | 本文 + 愿景条目 + `DISTRO_CAPABILITY_PROFILE` 开关说明 | 文档评审 · **Done** |
| **T1** | 角色包 schema · 编写器导出 · `oclive_validation` | mumu 样例 `deep_capsule.txt`；validate 通过 · **Done** |
| **T2** | `PromptBuilder` Deep 分支 · `co_present` 传 mode · bench | Deep TTFT 较全量下降 ≥20%（同模型同机）· **Done** |
| **T3** | 前缀分段 + Ollama llama.cpp 前缀缓存 + `keep_alive` | 连续 5 轮 Deep，第 2–5 轮 p50 `prompt_eval_ms` 低于第 1 轮 · **Done**（见 [`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md) `--deep-multi`） |

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
