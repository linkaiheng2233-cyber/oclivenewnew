# TTFT 基准复现（Chat Pro co-present）

**指标**：`POST /chat/stream` 从请求发出到首个 `event:token` 的墙钟时间（p50 门禁 **≤ 1000ms**）。

**架构归类**（规则 event · Turn Thinking · Prompt 分层）：[`MODULE_MAP_AND_HANDOFF.md`](MODULE_MAP_AND_HANDOFF.md) §6–§12。

## 环境

- 角色：`distros/chat-pro/roles/mumu`
- 场景：`home`（脚本自动 co-present setup）
- 模型：本地 Ollama `qwen2.5:7b`
- 发行版：建议 `OCLIVE_DISTRO_PROFILE=examples/distro-profiles/desktop-latency.oclive.toml`（Turn Thinking Auto + 规则 event）

## 命令

```powershell
# 终端 1
$env:OCLIVE_APP_DATA = "D:\oclivenewnew\temp\oclive_ttft_bench"
$env:OCLIVE_DISTRO_PROFILE = "D:\oclivenewnew\examples\distro-profiles\desktop-latency.oclive.toml"
D:\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclivenewnew-tauri.exe --api --port 8420

# 终端 2
cd D:\oclivenewnew
node -e "fetch('http://127.0.0.1:8420/llm/user_settings',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({roleId:'mumu',provider:'local',ollamaModel:'qwen2.5:7b'})})"
node scripts/measure-ttft.mjs --runs 5 --ollama-model qwen2.5:7b
```

## Stage 分解（可选）

```powershell
$env:RUST_LOG = "oclive_turn=debug"
```

## 相关开关

| 开关 | 作用 |
|------|------|
| `[host_flags] event_impact_llm = false` | 全局跳过 event `generate_tag` |
| `[turn_thinking] default = "auto"` | 闲聊 Fast / 高情绪 Deep |
| `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt` | Small+Deep 用离线 capsule（见 [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md)） |
| `node scripts/measure-ttft.mjs --deep-only` | 仅测 Deep 轮 TTFT（长句触发） |
| `OCLIVE_EVENT_IMPACT_LLM=0` | 环境变量等价关闭 event LLM |

## 实测摘要（2026-06 · mumu · qwen2.5:7b · `desktop-latency`）

| 场景 | Stream TTFT p50 | 备注 |
|------|-----------------|------|
| 优化前（event LLM + 全链） | ~2468ms | FAIL |
| **Wave A/B 后（Auto → Fast）** | **~243ms** | PASS |
| Ollama 直连极短 prompt | ~130ms | 物理下限参考 |

**Deep 路径（Wave D · Small+Deep capsule）**：启用 mumu `deep_capsule_enabled` + `--deep-only` 测 Deep TTFT；capsule ~2k 字 vs 全量 ~4.9k 字，目标 prefill 下降 ≥20%。人设 checklist 见 [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md) §3.2。

## Related

- [`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md)
- [`PERF_PHASES.md`](PERF_PHASES.md)
- [`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) §Wave A–D
