# Backend Perf Runbook

用于封板前后手动压测后端 chat turn 性能，输出延迟分布并观察慢事务风险。

## 1) 执行命令

在 `src-tauri/` 下运行：

```bash
cargo test --test perf_chat_turns -- --ignored --nocapture
```

## 2) 输出说明

测试会打印：

- `rounds`: 压测轮数（当前 200）
- `failed`: 失败次数（应为 0）
- `p50_ms`: 中位延迟
- `p95_ms`: 95 分位延迟
- `p99_ms`: 99 分位延迟
- `max_ms`: 最大延迟

示例：

```text
perf_chat_turn_distribution rounds=200 failed=0 p50_ms=8 p95_ms=24 p99_ms=40 max_ms=66
```

## 3) 参考阈值（当前阶段）

- `failed == 0`
- `p95_ms < 300`（和事务慢警告阈值保持一致）
- `p99_ms < 800`（不触发 critical 级别慢事务风险）

> 注：本压测为本地内存 DB + Mock LLM，主要用于回归比较，不等同于线上真实性能。

## 4) 当阈值异常时排查

1. 检查是否频繁触发 `TXN_SLOW_WARN` / `TXN_SLOW_CRITICAL`
2. 查看 `tx step` 日志，定位慢步骤（memory/event/favorability/personality）
3. 检查是否出现高频 `TXN_*` 失败码
4. 对比最近迁移或索引变更（`003_perf_indexes.sql`）

