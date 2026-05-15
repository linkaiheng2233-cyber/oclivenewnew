# 性能基线与已知限制（A7）

本文面向**对外披露**：说明当前仓库在典型 Release 构建下的**体积采样**、**Monolith 模式**与**基准测试入口**，并列出产品形态下的**已知性能边界**。实现细节以源码与 CI 为准；数值会随依赖与编译选项漂移，请以 [`LIGHTWEIGHT_PROFILE.md`](../development/LIGHTWEIGHT_PROFILE.md) 中最新 **`cargo-bloat`** 采样为准。

---

## 1. 二进制体积（Release 采样）

以下数据摘自 **`creator-docs/development/LIGHTWEIGHT_PROFILE.md` §6.7**（**Windows x86_64**，**Release**，采样日期 **2026-05-12**；`cargo bloat --release -n 8`，可执行文件为 `oclivenewnew-tauri.exe`，`target-dir` 以外置配置为准）。

| 指标 | 数值 |
|------|------|
| **`.text` 段（`cargo-bloat` 报告）** | **约 7.6 MiB** |
| **PE 可执行文件大小（报告末行）** | **约 12.0 MiB** |

复测命令与 Top 符号节选见 **`LIGHTWEIGHT_PROFILE.md`** 正文。其它平台/构建类型未在文中逐项承诺；发布物请以各 OS 安装包或 CI 产物实测为准。

---

## 2. Monolith 模式与「消除虚调用」在说什么

**Monolith（高耦合编译）**面向无头/嵌入式脚手架：在编译期将七槽后端**焊接**为静态路径，使热点路径尽量走**直接调用**而非经 `PluginHost` 等层的**动态分派**（trait 对象、间接分支），从而有利于 **ICache / 内联** 与可预测的优化空间。

- **原理摘要**：将「运行时选后端」收缩为「编译期已知实现」，减少虚调用与分支预测压力；**不等价于**自动变快多少个百分点。  
- **如何自证**：在由 **`oclive-cli`** 生成的 Monolith 工程上，使用下文 **`oclive bench`** 对比 **`main.rs`** 与 **`main_monolith.rs`** 双二进制输出；结论以**你本机** JSON 报告为准。  
- **权威说明**：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · 命令与流程 [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)（`init --monolith`、`build`、`bench`）。

---

## 3. 基准测试方法：`oclive bench`

`oclive-cli` 提供 **`bench`** 子命令：在 `init`/`build` 产出双二进制后，对两者各跑若干次子进程，子进程内通过 **`OCLIVE_KERNEL_BENCH_ITERS`** 做热循环，输出 **JSON**（`schema_version: 1`）。

**常用示例**（详见 [OCLIVE_CLI_GUIDE.md § `bench`](../cli/OCLIVE_CLI_GUIDE.md)）：

```bash
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --json
```

- **`--save`** / **`--compare`**：写入/对比项目根 **`bench_history.json`**（本地文件，勿提交仓库）。  
- **JSON Schema（机器可读）**：仓库内 **`crates/oclive-cli/schemas/oclive_bench_report.schema.json`**。  
- **CI**：`.github/workflows/ci.yml` 中含轻量 **`cli-bench`** job（冒烟一轮，不设性能阈值）。

---

## 4. 已知性能限制（产品向）

| 限制 | 说明 |
|------|------|
| **单进程、单角色会话模型** | 桌面宿主以单进程内编排为主；多角色切换是顺序使用，不是多角色并行硬隔离（与嵌入式/云多租户不同）。 |
| **LLM 延迟完全依赖后端** | 本地 **Ollama**、**remote** HTTP、目录插件 LLM 等路径的尾延迟由**模型体量、量化、硬件、网络**决定；宿主侧不保证固定上界。 |
| **无硬实时（hard real-time）保证** | 对话、插件调用、磁盘与 UI 均在通用 OS 调度下运行；不适用于安全关键硬实时场景。 |
| **CPU 推理下的首 token 延迟** | 未配置 GPU / 未使用流式端点时，首包时间可能**明显长于**轻量云端 API；属模型与硬件范畴，非应用「卡顿」缺陷定义。 |

更细的工程基线（历史 perf handoff、CI 策略）见 **`handoff/PERF_*`** 与根目录 **`AGENTS.md`** 中性能文档索引。

---

## 修订记录

| 日期 | 说明 |
|------|------|
| 2026-05-15 | 初版：对齐 `LIGHTWEIGHT_PROFILE.md` §6.7 与 `oclive bench` / Schema 路径。 |

[English](../../creator-docs-en/getting-started/PERFORMANCE.md)
