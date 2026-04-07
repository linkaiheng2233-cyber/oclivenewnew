# Perf Baseline 2026-04-01

基线来源：`src-tauri/tests/perf_chat_turns.rs`（内存 DB + Mock LLM）

## Command

```bash
cd src-tauri
cargo test --test perf_chat_turns -- --ignored --nocapture
```

## Result

```text
perf_chat_turn_distribution rounds=200 failed=0 p50_ms=0 p95_ms=0 p99_ms=1 max_ms=1
```

## Interpretation

- 该结果反映“逻辑链路开销基线”（非真实模型调用场景）。
- 对比价值：
  - 后续如 p95/p99 明显上升，优先排查事务步骤与 SQL 变更。
  - 与真实 LLM 场景分离记录，避免混淆。

## Next Baseline Plan

- 接入真实 Ollama（固定模型）再跑一版基线（记录网络与推理耗时）。
- 分别记录：
  - `Mock LLM` 基线（纯后端逻辑）
  - `Real LLM` 基线（端到端体验）

