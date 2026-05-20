# Monolith 焊接对比报告（模板）

填写本表以记录 **标准版** 与 **Monolith 焊接版** 子进程基准差异，作为调整 `monolith.toml` 中 `weld_modules` 的依据。

> 桌面宿主主路径不走本报告；仅供 **纯内核 / 无头脚手架** 性能调优。

English: **`WELD_BENCH_REPORT.en.md`**

## 1. 环境与命令

| 项 | 填写 |
|----|------|
| 项目名 | |
| 日期 | |
| `monolith.toml` weld_modules | |
| 命令 | `cargo run -p oclive-cli -- bench --release --runs 20 -o <项目根>` |
| 或 init 自动 | `oclive init --monolith --monolith-bench-preset <latency\|memory\|embedded>` → `bench_results/report.json` |

## 2. 延迟对比（单位：ms，子进程 wall time）

| 槽位 / 指标 | 标准版 p50 | 焊接版 p50 | 标准版 P95 | 焊接版 P95 | 提升 %（p50） |
|-------------|-----------|-----------|-----------|-----------|---------------|
| **整体子进程** | | | | | |
| memory | — | — | — | — | — |
| emotion | — | — | — | — | — |
| event | — | — | — | — | — |
| prompt | — | — | — | — | — |
| llm | — | — | — | — | — |
| agent | — | — | — | — | — |
| complex_emotion | — | — | — | — | — |

说明：当前 `oclive bench` 采样**整二进制**一轮热路径，非单槽拆分；上表「整体」行填 `bench_results/report.json` 中 `standard_ms` / `monolith_ms`；单槽行可在接入分槽 benchmark 后补充。

**提升 %（p50）** ≈ `(标准版 p50 - 焊接版 p50) / 标准版 p50 × 100`。

## 3. 多维度指标（`bench_results/report.json` schema v2）

| 维度 | JSON 字段 | 单位 | 说明 |
|------|-----------|------|------|
| 二进制大小 | `binary_size.standard` / `.monolith` | bytes | `target/release/{package}` 与 `{package}-monolith` 文件大小 |
| 峰值内存 | `peak_memory.standard` / `.monolith` | MiB | 各轮子进程 RSS 峰值的最大值（`sysinfo` 采样） |
| 编译时间 | `build_time.standard` / `.monolith` | 秒 | `cargo build --release` 标准次与 Monolith 次分别计时（需 `--release`） |

终端 `oclive bench` 在延迟表后会打印上述三行对比；`--json` 输出含完整结构。

## 4. 结论与下一步

- [ ] 焊接版明显更快 → 保持或扩大 `weld_modules`
- [ ] 差异不大 → 缩小焊接范围以减体积（如 `--monolith-preset embedded`）
- [ ] 焊接版更慢 → 检查误焊、调试符号、或减少焊槽

调整 `monolith.toml` 后执行：

```bash
cargo run -p oclive-cli -- build -o .
cargo run -p oclive-cli -- bench --release -o . --runs 20 --save
```

编排语义见 **`ORCHESTRATION_REFERENCE.md`**；RFC 见主仓 **`RFC_OCLIVE_MONOLITH_MODE.md`**。
