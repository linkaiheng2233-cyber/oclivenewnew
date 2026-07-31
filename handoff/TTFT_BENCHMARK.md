# TTFT 基准复现（Chat Pro co-present）

**指标**：`POST /chat/stream` 从请求发出到首个 `event:token` 的墙钟时间（p50 门禁 **≤ 1000ms**）。

**架构归类**（规则 event · Turn Thinking · Prompt 分层）：[`MODULE_MAP_AND_HANDOFF.md`](MODULE_MAP_AND_HANDOFF.md) §6–§12。

## 环境

- 角色：`distros/chat-pro/roles/mumu`
- 场景：`home`（脚本自动 co-present setup）
- 模型：本地 Ollama `qwen2.5:7b`

## Profile 区分

| Profile | `distro_id` | 用途 | `OCLIVE_DISTRO_PROFILE` |
|---------|-------------|------|---------------------------|
| **`desktop-latency`** | `desktop-latency` | 开发 bench（`event_impact_llm = false`） | `examples/distro-profiles/desktop-latency.oclive.toml` |
| **`desktop`** | `desktop` | **正式用户默认**（Release bundled） | `examples/distro-profiles/desktop.oclive.toml` 或 `distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml` |

**说明**：Wave C 流式 UI 改善**感知延迟**（逐字显示），不改变 stream TTFT 数值本身；blocking `/chat` 整段延迟仍可作为对照。

## 命令

### 表 1 · `desktop-latency`（开发 bench）

```powershell
# 终端 1
$env:OCLIVE_APP_DATA = "D:\oclivenewnew\temp\oclive_ttft_bench"
$env:OCLIVE_DISTRO_PROFILE = "D:\oclivenewnew\examples\distro-profiles\desktop-latency.oclive.toml"
D:\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclivenewnew-tauri.exe --api --port 8420

# 终端 2
cd D:\oclivenewnew
node -e "fetch('http://127.0.0.1:8420/llm/user_settings',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({roleId:'mumu',provider:'local',ollamaModel:'qwen2.5:7b'})})"
node scripts/measure-ttft.mjs --profile desktop-latency --runs 5 --ollama-model qwen2.5:7b
```

### 表 2 · `desktop`（正式 profile）

```powershell
# 终端 1 — 将 OCLIVE_DISTRO_PROFILE 换为 desktop.oclive.toml
$env:OCLIVE_DISTRO_PROFILE = "D:\oclivenewnew\examples\distro-profiles\desktop.oclive.toml"

# 终端 2
node scripts/measure-ttft.mjs --profile desktop --runs 5 --ollama-model qwen2.5:7b
```

`--profile` 会打印期望的 `OCLIVE_DISTRO_PROFILE` 路径，并通过 `/health` 的 `distro_id` 校验是否匹配。

### 表 3 · 多轮 Deep prefill（Wave D-T3）

与 stream **TTFT** 不同：测 Ollama `prompt_eval_duration`（内核 `llm_prompt_eval_ms`）。

```powershell
# 终端 1 — 在表 1 基础上增加：
$env:OCLIVE_BENCH_TELEMETRY = "1"
# desktop-latency.oclive.toml 已含 prompt_prefix_cache = true

# 终端 2
node scripts/measure-ttft.mjs --profile desktop-latency --deep-multi --runs 5 --ollama-model qwen2.5:7b
```

**门禁**：round 2–5 `prompt_eval_ms` p50 **&lt;** round 1（同角色同场景同模型、顺序请求）。

## Stage 分解（可选）

```powershell
$env:RUST_LOG = "oclive_turn=debug"
```

## 相关开关

| 开关 | 作用 |
|------|------|
| `[host_flags] event_impact_llm = false` | 全局跳过 event `generate_tag`（`desktop-latency` 默认） |
| `[turn_thinking] default = "auto"` | 闲聊 Fast / 高情绪 Deep |
| `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt` | Small 模型 Fast/Deep 共用离线 persona capsule（沿用字段/文件名，见 [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md)） |
| `node scripts/measure-ttft.mjs --deep-only` | 仅测 Deep 轮 TTFT（长句触发） |
| `[turn_thinking] prompt_prefix_cache = true` | Fast/Deep + Ollama + 内置 prompt 后端使用稳定前缀分段与 `keep_alive`（`desktop-latency` 默认开启；目录/远程 prompt 不改写） |
| `OCLIVE_PROMPT_PREFIX_CACHE=1` | 环境变量强制开启前缀缓存（覆盖 profile） |
| `OCLIVE_BENCH_TELEMETRY=1` | `SendMessageResponse.llm_prompt_eval_ms`（仅 bench，不进产品 UI） |
| `node scripts/measure-ttft.mjs --deep-multi --runs 5` | 连续 5 轮 Deep **prefill**（`prompt_eval_ms`，非 stream TTFT） |
| `OCLIVE_EVENT_IMPACT_LLM=0` | 环境变量等价关闭 event LLM |
| `[turn_thinking] fast_persistence = "strong_only"` | Fast 闲聊不写 long_term / favor / evolution（强事件仍写）；见 Wave E |
| `OCLIVE_FAST_PERSISTENCE=strong_only` | 环境变量强制 `strong_only`（覆盖 profile） |

## Wave E · Fast 持久化分流（手测）

**Profile**：`desktop-latency` 与正式 `desktop` 均已启用 `fast_persistence = "strong_only"`（RFC [`RFC_TURN_THINKING_PERSISTENCE.md`](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)）。

```powershell
# 终端 1 — 表 1 环境 + desktop-latency profile（已含 fast_persistence）
$env:OCLIVE_DISTRO_PROFILE = "D:\oclivenewnew\examples\distro-profiles\desktop-latency.oclive.toml"

# 终端 2 — TTFT 回归（Fast 闲聊仍 ~243ms 量级）
node scripts/measure-ttft.mjs --profile desktop-latency --runs 5 --ollama-model qwen2.5:7b
```

| 场景 | 期望 |
|------|------|
| Fast 10 轮闲聊（短句「你好」等） | stream TTFT p50 **维持** Wave B 量级（历史 ~243ms）；`role_runtime.favorability` 变化 **&lt; 0.01** |
| 1 轮 Deep（≥80 字或高情绪句） | `long_term_memory` 有新增；好感可观测变化 |
| Fast + 强事件（争吵/道歉等） | 仍写入 long_term / favor（与 `legacy` 一致） |

**说明**：聊天 turns 每轮仍写（UI 可见）；本 Wave 只分流 long_term / favor / evolution。

## OOCP S15（流式 SSE smoke）

默认套件含 **S15**（`POST /chat/stream` → `event:token` + `event:done` 含非空 `reply`）：

```powershell
# 终端 1 — Mock LLM，无需 Ollama
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$env:OCLIVE_APP_DATA = "D:\oclivenewnew\temp\oclive_oocp"
D:\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclivenewnew-tauri.exe --api --port 8420

# 终端 2
cd D:\oclivenewnew
node examples/oocp-test-suite/run.mjs
```

详见 [`creator-docs/testing/OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md) §S15。

## 实测摘要（2026-06 · mumu · qwen2.5:7b）

### 表 1 · `desktop-latency`（开发 bench）

| 场景 | Stream TTFT p50 | 备注 |
|------|-----------------|------|
| 优化前（event LLM + 全链） | ~2468ms | FAIL |
| **Wave A/B 后（Auto → Fast）** | **~243ms** | PASS |
| Ollama 直连极短 prompt | ~130ms | 物理下限参考 |

### 表 2 · `desktop`（正式 profile）

在 `desktop.oclive.toml`（`event_impact_llm` 默认 true）下复测；Deep 轮可能调 event LLM，TTFT 高于 Fast 路径属预期。发版前用 `--profile desktop` 记录一行 p50 填入 [`PERF_PHASES.md`](PERF_PHASES.md)。

**Deep 路径（Wave D · Small persona capsule）**：启用 mumu `deep_capsule_enabled` + `--deep-only` 测 Deep TTFT；capsule ~2k 字 vs 全量 ~4.9k 字，目标 prefill 下降 ≥20%。人设 checklist 见 [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md) §3.2。

### 表 3 · 多轮 Deep prefill（Wave D-T3 · `desktop-latency`）

| 轮次 | `prompt_eval_ms` |
|------|------------------|
| Round 1（冷前缀） | **1443** |
| Round 2 | 511 |
| Round 3 | 28 |
| Round 4 | 1450 |
| Round 5 | 26 |
| **Round 2–5 p50** | **28**（&lt; Round 1 · **PASS**） |

环境：mumu · `qwen2.5:7b` · `prompt_prefix_cache=true` · `OCLIVE_BENCH_TELEMETRY=1` · 2026-06-26。

### 表 5 · 当前优化波次（2026-07-23 · `desktop-latency`）

同一角色、场景、模型与本地 Ollama 0.32.1 下，Fast 由全量 core（prompt 约 4071 token，stream TTFT p50 **1761ms**）切换为 persona capsule + 稳定前缀布局后，7 轮 stream TTFT 为 **min 193 / p50 221 / max 1320ms**；首轮冷 prompt eval **686ms**，后续命中稳定前缀为 **27–29ms**。一次 max 属于冷/调度离群值，p50 通过 1s 门禁。直连 Ollama 极短 prompt p50 **151ms**。

该结果证明 `keep_alive` 本身只保证模型驻留；稳定前缀必须先把输入压到上下文窗口内，才能让 Ollama 复用 KV。目录/远程 prompt 后端不使用该重排路径。

### 表 4 · Wave F 手测（`desktop-latency` · 可选）

| 场景 | 预期 |
|------|------|
| 短句 + 规则 Quarrel | Auto → **Deep**（`this_turn_event` prepass，无需长消息） |
| mumu `turn_thinking.latch` | Quarrel 后 **持续 Deep** 直至 Apology |
| `ephemeral_archive.ttl_turns=3` | 局面摘要 **3 轮** 后清空；Fast 轮仍注入 Prompt |
| Fast + ephemeral ≤200 字 | TTFT 增量手测目标 **&lt;50ms**（不阻塞 CI） |

示例包：`distros/chat-pro/roles/mumu/config.json` → `turn_thinking`（注释见 RFC §12 示例）。

## Chat Pro 成人 staged-beat 队列（D29）

后台多拍使用单个全局串行生成泵；缓存深度不会并行加载多份模型。启动带真实本地模型的 Chat Pro HTTP 内核后运行：

```powershell
node scripts/measure-adult-stage.mjs `
  --base http://127.0.0.1:8430 `
  --role gentle-landlady `
  --scene default `
  --caps 1,2,4,8
```

脚本会为每个深度创建独立会话，按序执行 begin/beat/list/cancel，验证结构化字段和持久化数量，只输出耗时、长度、状态与可选 NVIDIA GPU 采样，不打印生成正文。

2026-07-27 参考结果：RTX 5060 Laptop 8GB、Qwen2.5 7B Q4_K_M + 消融 LoRA、全 GPU offload；15/15 拍结构化成功且零回退，热态单拍 p50 **1754ms**、p95 **2112ms**，显存 **6208～6228MiB**。缓存深度 8 未见延迟或显存递增。默认值保持 `2`，同档 8GB + 7B Q4 建议 `2～4`；更高值的主要风险是持续功耗以及用户输入使未展示剧情失效。

语音共存使用：

```powershell
python scripts/stress-voice-gpu-runtime.py --gpu-layers 24 --voice-runs 5
```

同机 CosyVoice2 mixed-fp16 结果：峰值 **6751/8151MiB**、峰值余量 **1400MiB**、稳态增长 **0MiB**；LLM 热态 TTFT p50 **142ms**，语音 TTFC p50 **4293ms**。后台队列只缓存文本，不预生成语音；返回前台后仍按单拍顺序合成。

2026-07-28 关联竞态与恢复修复后的复验：Ollama `qwen2.5:7b` 深度 1/2/4 共 **7/7** 拍结构化成功、零回退，热态总体 p50 **1047ms**，显存稳定 **5523MiB**；另以深度 8 连跑三轮共 **24/24** 拍，p50 为 **896/893/942ms**，内核工作集 **38.9→39.6MiB** 后稳定、句柄 **242→243** 后稳定。`--gpu-layers 24 --voice-runs 20` 共存压力中峰值 **6657/8151MiB**、余量 **1494MiB**、稳态增长 **0MiB**，热态 LLM TTFT p50 **145ms**、语音 TTFC p50 **3927ms**。同次普通聊天 7 轮真实流式 TTFT p50 **209ms**（直连 Ollama p50 **123ms**）。

2026-07-30 Resource Coordinator Stage 2.3 复验（HEAD `d43f5cf8`，RTX 5060 Laptop **8151MiB**）：`--gpu-layers 24 --voice-runs 10` 的 direct runtime 共存短压测通过，峰值 **6759/8151MiB**、最小余量 **1392MiB**、稳态增长 **17MiB**，未见 OOM、泄漏趋势或残留模型进程；LLM TTFT p50 **179ms**、p95 **1659ms**，语音 TTFC p50 **5008ms**、p95 **6288ms**。全 GPU `--gpu-layers 99 --expect-admission-denied` 在 CosyVoice 分配显存前以 `gpu_admission_denied` 安全拒绝（语音峰值分配 **0MiB**），同时 LLM 仍可用，TTFT **1062ms**。单独语音对照中，短句 TTFC p50/p95 为 **1918/2158ms**，与压力脚本相同的较长片段为 **5538/6254ms**；另一次三轮共存样本出现 **8995ms** 最大值，说明当前 8 秒绝对上限易受长片段和冷抖动影响。以上是适配器直连与短时压力证据，不替代完整 Tauri 宿主 `LLM suspend → Voice → confirmed unload → LLM recover` 实机闭环和长时间 soak。

## Related

- [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md)
- [`PERF_PHASES.md`](PERF_PHASES.md)
- [`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) §Wave A–D
