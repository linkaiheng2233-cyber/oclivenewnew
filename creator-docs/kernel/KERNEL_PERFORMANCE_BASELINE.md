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
cargo bench -p oclive_kernel_runtime --bench kernel_pipeline_blueprint -- --verbose
cargo bench -p oclive_kernel_runtime --bench kernel_plugins_persistence -- --verbose
```

**CI 基线对比**：机器可读数值见同目录 [`kernel_perf_baseline_v0.json`](./kernel_perf_baseline_v0.json)；GitHub Actions 工作流在 **手动触发** 且勾选 **`bench_full`** 时运行上述 benches，并由 `scripts/criterion_compare_baseline.py` 与 JSON 对比，超约 **15%** 退化输出 `::warning::`（不阻塞主 CI 矩阵）。

## 蓝图解释器（`kernel_pipeline_blueprint`）

对比 **无 `pipeline.ocblueprint` 文件**（默认入口线性序列）与 **示例蓝图**（`simple_companion` / `memory_heavy`）下单次 `process_message` 的耗时。临时目录自 `roles/shimeng` 克隆并写入蓝图；需本机存在 `roles/shimeng`。

| 基准 ID | 含义 | 耗时（`--quick` 估计值，本机一次采样） | 备注 |
|---------|------|----------------------------------------|------|
| `process_message_default_no_blueprint` | 无蓝图文件，走默认入口 | **~3.38–3.50 ms** | 与 `kernel_hot_paths` 同量级（Mock LLM） |
| `process_message_blueprint_simple_companion` | 与默认八步等价的蓝图 | **~3.67–3.86 ms** | 解释器 + 与默认同序原子 |
| `process_message_blueprint_memory_heavy_parallel` | 含只读 `parallel` + 生成 | **~4.19–4.42 ms** | `join!` 调度与额外只读步 |

**复现**：`cargo bench -p oclive_kernel_runtime --bench kernel_pipeline_blueprint -- --verbose`（完整 Criterion）；快速冒烟可加 `-- --quick`。

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
| 2026-05-12 | bench(pipeline) 蓝图对照 | 新增 `kernel_pipeline_blueprint`；记录默认 vs 蓝图 vs `memory_heavy` 并行 | `process_message_*`（本表「蓝图解释器」节） | （见上节 quick） | （见上节 quick） | 蓝图路径略增开销；并行示例因 join 与额外步略高 |
| 2026-05-06 | perf(kernel) IO 缓存 | **`McpClient`**：`mcp-servers/` 目录 mtime 未变则跳过重读 JSON；**目录插件**：`rescan_plugin_roots` 命中 **mtime 指纹磁盘缓存**（`OCLIVE_BUST_PLUGIN_SCAN_CACHE` 可 bust） | `directory_rescan_and_bootstrap_dto`、MCP 列表路径 | 以本机 `cargo bench -p oclive_kernel_runtime --bench kernel_hot_paths --bench kernel_plugins_persistence -- --quick` 复测为准 | 同上 | 二次 rescan / 高频 `list_mcp_servers` 预期改善；全量 Criterion 以 CI `bench_full` 或本地 verbose 为准 |

## 冷启动与缓存（手测记录位）

在 **`RUST_LOG=oclive_startup=info`** 下关注 `phase=kernel_app_state_total_ms`；二次启动若数据目录未变，目录插件 rescan 日志中应出现 **`disk_cache_hit=true`**（指纹与 `.oclive_plugin_scan_cache_v1.json` 一致）。

| 日期 | 环境 | kernel_app_state_total_ms（首次 / 二次） | 备注 |
|------|------|-------------------------------------------|------|
| 2026-05-06 | （请本机填写） | / | 二次启动关注 `oclive_plugin` 的 `disk_cache_hit` |

## 长时内存冒烟（可选，非 RSS）

Mock LLM 下连续 **100** 轮 `process_message` 后断言 `count_memories` 上限（防明显逻辑泄漏）。集成测试 target 默认不编译（feature **`slow-long-tests`**），且测试体为 **`#[ignore]`**：

```bash
cargo test -p oclive_kernel_runtime --features "full,slow-long-tests" --test p_long_chat_memory_bounds -- --ignored --nocapture
```

GitHub Actions：手动触发 CI 并勾选 **`memory_smoke`**（见 `.github/workflows/ci.yml`）。

---

*文档版本：与 Baseline v0 数据同步生成；Criterion 报告目录默认在 `target/criterion/`（本仓库可能因 `target-dir` 重定向至外部路径）。*
