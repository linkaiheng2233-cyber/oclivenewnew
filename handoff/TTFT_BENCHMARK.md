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
| `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt` | Small+Deep 用离线 capsule（见 [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md)） |
| `node scripts/measure-ttft.mjs --deep-only` | 仅测 Deep 轮 TTFT（长句触发） |
| `[turn_thinking] prompt_prefix_cache = true` | Deep+Ollama 稳定前缀分段 + `keep_alive`（`desktop-latency` 默认开启） |
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
| Fast 10 轮闲聊（短句「你好」等） | stream TTFT p50 **维持** Wave B 量级（~243ms）；`role_runtime.favorability` 变化 **&lt; 0.01** |
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

**Deep 路径（Wave D · Small+Deep capsule）**：启用 mumu `deep_capsule_enabled` + `--deep-only` 测 Deep TTFT；capsule ~2k 字 vs 全量 ~4.9k 字，目标 prefill 下降 ≥20%。人设 checklist 见 [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md) §3.2。

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

### 表 4 · Wave F 手测（`desktop-latency` · 可选）

| 场景 | 预期 |
|------|------|
| 短句 + 规则 Quarrel | Auto → **Deep**（`this_turn_event` prepass，无需长消息） |
| mumu `turn_thinking.latch` | Quarrel 后 **持续 Deep** 直至 Apology |
| `ephemeral_archive.ttl_turns=3` | 局面摘要 **3 轮** 后清空；Fast 轮仍注入 Prompt |
| Fast + ephemeral ≤200 字 | TTFT 增量手测目标 **&lt;50ms**（不阻塞 CI） |

示例包：`distros/chat-pro/roles/mumu/config.json` → `turn_thinking`（注释见 RFC §12 示例）。

## Related

- [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md)
- [`PERF_PHASES.md`](PERF_PHASES.md)
- [`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) §Wave A–D
