# 内核性能基线（Criterion）

本文档记录 **`oclive_kernel_runtime`** 在固定 Criterion 场景下的**首份量化数据**，供后续优化与回归对比。更新基线时请保留历史行或在本节追加新表，并在「优化对比记录」中登记变更。

## 测试环境（Baseline v0）

| 项目 | 值 |
|------|-----|
| 日期（UTC） | 2026-05-04 |
| 操作系统 | Windows 11 x64 |
| CPU | Intel Core Ultra 9 275HX |
| 内存（系统报告） | 约 15.4 GiB 物理内存 |
| Rust | rustc 1.95.0 (2026-04-14)，cargo 1.95.0 |
| 编译配置 | `cargo bench` 默认 **bench profile**（优化，带调试信息因 profile 而异） |
| 仓库 / 分支 | `oclivenewnew`，`feat/oocp-v0-1`（记录时） |
| 前置条件 | 仓库内存在 `roles/shimeng`；目录插件基准依赖 `cargo bench` 注入的 `CARGO_BIN_EXE_oclive_test_dir_plugin` |

**说明**：以下数值来自 Criterion 单次完整运行；方括号内为 **95% 置信区间** `[下界 估计值 上界]`（与 Criterion 终端输出一致）。部分项附带 **median** 行（终端 `median [...]`）。测试时建议关闭高负载后台任务；本机结果仅作相对参照，跨机器对比需在同一环境复跑。

## 复现命令

```bash
cargo bench -p oclive_kernel_runtime --bench kernel_hot_paths -- --verbose
cargo bench -p oclive_kernel_runtime --bench kernel_plugins_persistence -- --verbose
```

**CI 基线对比**：机器可读数值见同目录 [`kernel_perf_baseline_v0.json`](./kernel_perf_baseline_v0.json)；GitHub Actions 工作流在 **手动触发** 且勾选 **`bench_full`** 时运行上述 benches，并由 `scripts/criterion_compare_baseline.py` 与 JSON 对比，超约 **15%** 退化输出 `::warning::`（不阻塞主 CI 矩阵）。

## 热点路径（`kernel_hot_paths`）

| 基准 ID | 含义 | 耗时（95% CI） | 备注 |
|---------|------|----------------|------|
| `process_message_once_mock_llm` | 单次 `process_message`（Mock LLM，`shimeng`） | **12.522 ms ~ 13.474 ms ~ 14.891 ms** | median 约 **12.208 ~ 12.510 ms** |
| `process_message_10_rounds/sequential_10` | 连续 **10** 轮 `process_message`（同会话） | **125.12 ms ~ 131.61 ms ~ 138.22 ms** | 单轮均值约 12.5~13.9 ms（与上项同量级） |
| `load_role_toggle_two_roles` | `load_role` 在 **bench_lr_a / bench_lr_b** 间各一次 | **3.7013 ms ~ 3.7184 ms ~ 3.7363 ms** | 自 `shimeng` 克隆的两角色目录 |

## 插件与持久化（`kernel_plugins_persistence`）

| 基准 ID | 含义 | 耗时（95% CI） | 备注 |
|---------|------|----------------|------|
| `directory_rescan_and_bootstrap_dto` | `rescan_plugin_roots` + `directory_plugin_bootstrap_dto` | **124.29 µs ~ 126.66 µs ~ 129.60 µs** | median 约 **123.43 ~ 125.26 µs** |
| `directory_plugin_rpc_ping` | `invoke_directory_plugin_rpc("ping", …)` | **368.89 µs ~ 385.72 µs ~ 403.63 µs** | median 约 **353.47 ~ 381.89 µs**（含子进程 HTTP 侧车） |
| `mcp_call_tool_denied_stdio_no_grant` | MCP **stdio** 工具调用在未授予 `process:spawn` 时快速拒绝 | **1.9215 ms ~ 1.9814 ms ~ 2.0579 ms** | 权限检查路径，不起真实子进程 |
| `memory_save_and_load_32` | `save_memory` + `load_memories(32)` | **1.9199 ms ~ 2.0328 ms ~ 2.1862 ms** | median 约 **1.8285 ~ 1.9318 ms** |
| `role_pack_zip/export_import_roundtrip` | `export_role_pack` + `import_role_pack` 各一轮（极小 `mumu` 包） | **1.7590 ms ~ 1.7979 ms ~ 1.8375 ms** | median 约 **1.7380 ~ 1.8078 ms** |

## 后续优化对比记录

在此追加行登记每次有意图的性能工作（PR / 提交 / 说明）。对比时请使用**相同命令**与尽量一致的环境。

| 日期 | PR / 提交 | 变更摘要 | 受影响基准 | 变更前（估计值） | 变更后（估计值） | 结论 |
|------|-----------|----------|------------|------------------|------------------|------|
| — | — | （占位） | — | — | — | — |

---

*文档版本：与 Baseline v0 数据同步生成；Criterion 报告目录默认在 `target/criterion/`（本仓库可能因 `target-dir` 重定向至外部路径）。*
