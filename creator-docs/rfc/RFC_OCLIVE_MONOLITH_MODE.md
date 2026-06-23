# RFC：oclive 高耦合编译模式（Monolith / 焊接模式）

| 元数据 | 值 |
|--------|-----|
| 状态 | **已落地**：`oclive-cli` 提供 **`init` / `build` / `bench`**、焊接计划校验、双入口 **`main.rs` + `main_monolith.rs`**；**七焊接键**静态入口以 **`vendor/oclive_monolith_builtin`**（模板见 `kernel/crates/oclive-cli/monolith_vendor/`）为权威来源，可替换为真实 `oclive_*_builtin`。 |
| 入口 | **`oclive init`** 创建脚手架；**`cargo run -p oclive-cli -- build|bench`** 维护与对比 Monolith 产物 |
| 编译配置 | **`monolith.toml`**：由 `init` 生成，**`oclive build`** 读取并再生成 `process_message_monolith.rs`；**不参与运行时**，与角色包/宿主加载路径无关 |
| 与蓝图边界 | **`pipeline.ocblueprint`**（或 `*.ocblueprint`）描述 **运行时** 编排；**焊接范围不以蓝图字段承载**（避免与 `PIPELINE_SCHEMA` 运行时语义混淆）。Monolith 仅通过 **`monolith.toml` + Cargo feature** 生效。 |
| 受众 | **仅开发者**；普通用户只使用开发者构建的发行版，不接触终端 |
| 本质 | **编译期**优化路径：以模块可替换性换取极限性能（可选、默认关闭），用于打破 **第 1–6 模块** 抽象在 **高频热路径** 上的性能天花板 |

**相关文档**：[`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)（模块编号）、[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)（第 1–6 模块与 `PluginHost`）、[`PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md)（蓝图运行时 Schema）、[`OCLIVE_CLI_GUIDE.md`](../cli/OCLIVE_CLI_GUIDE.md)（脚手架用法）。

**与实现一致（仓库主分支）**：`cargo run -p oclive-cli -- init … --monolith`（非交互，且项目类型为 **kernel_server**）或交互选择 **高耦合** 时，生成 **`monolith.toml`**、`vendor/oclive_monolith_builtin/`、`src/process_message_monolith.rs`，并在 `Cargo.toml` 中声明 **`[features] monolith`** 与第二 **`[[bin]]`**。修改 `monolith.toml` 后执行 **`cargo run -p oclive-cli -- build -o <项目根>`** 可再生成焊接源码并默认连续执行两次 `cargo build`（若 `enabled = true` 则第二次带 **`monolith`** feature）。**`cargo run -p oclive-cli -- bench`** 输出 JSON 延迟报告（Schema：`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`）。嵌入式 **library** 忽略 `--monolith` 且不生成 `monolith.toml`。

## 1. 问题陈述

oclive 的 **六宿主后端模块（第 1–6 模块）插件化架构**（`plugin_backends` + `PluginHost` + trait 门面）在提供 **生态灵活性**（builtin / remote / directory、侧车、目录插件）的同时，引入 **架构性开销**：

- **trait 虚调用**：`process_message` 编排链上多次动态分发；
- **模块间 DTO**：边界处为通用性往往走堆分配与拷贝路径（本 RFC 中「序列化」泛指跨抽象边界的打包成本，不限于 JSON）；
- **锁与运行时结构**：各子系统独立持锁时，高频路径上竞争与同步成本可见。

对 **不需要模块可替换性** 的高频场景（游戏 NPC、实时语音、嵌入式角色等），上述成本构成 **架构层面的性能天花板**：瓶颈不只在某一算法，而在 **抽象边界** 的累积。

本 RFC 提出 **高耦合编译模式（Monolith）**：开发者通过脚手架与 **`monolith.toml`** 声明焊接范围，在 **编译期** 将选定模块改为 **静态调用具体实现**，从而 **打破** 仅依赖低耦合插件架构时的热路径上限；未焊接模块仍走 **`PluginHost` + trait**。

---

## 2. 设计目标

| 目标 | 说明 |
|------|------|
| **入口在脚手架** | 仅在 **`oclive init`** 终端流程中暴露「开发者编译选项」；**不**写入桌面应用设置，不进入任何 GUI。 |
| **与 `monolith.toml` 结合** | 脚手架在初始化时 **生成** `monolith.toml`；**`oclive build`**（或项目内 `build.rs`/xtask）在编译时 **读取**，用于选择焊接集合与生成代码。**该文件不参与运行时加载。** |
| **编译期优化产物** | 启用 **`Cargo` feature `monolith`** 时，生成或编译 **第二套** 编排实现，消除 **被焊接模块** 的 trait 虚调用，产出可对比的第二二进制（见第 3 节命名）。 |
| **普通用户无感** | 终端用户只安装/运行开发者提供的 **已构建** 二进制；是否焊接、焊接哪些模块，全是 **开发者构建时** 决策。 |
| **打破架构性能限制** | 在明确放弃部分可替换性的前提下，为 **延迟敏感** 场景提供 **可选** 的极限性能路径，与标准低耦合模式 **长期并存**。 |

---

## 3. 脚手架交互设计

以下为 **`oclive init`** 在实现本 RFC 后的 **目标交互**（文案可调整，行为不变）。当前仓库仍以 **`cargo run -p oclive-cli -- init`** 为入口。

```text
$ oclive init my-fast-npc
  欢迎使用 oclive 脚手架！
  ? 项目名称: my-fast-npc
  …（角色包、第 1–6 模块后端等既有步骤）
  ? 是否启用开发者编译选项? [y/N]: y
```

进入开发者编译选项后：

```text
  ? 编译模式:
    ● 标准模式（低耦合，保留模块可替换性，推荐）
    ○ 高耦合模式 — 七焊接键全部静态焊接（第 1–6 模块 + `complex_emotion`）
    ○ 高耦合模式 — 自定义焊接范围（生成后编辑 monolith.toml，再运行 oclive build）
```

生成结束时的 **产物命名目标**（两版本可并存、便于对比）：

```text
  项目已创建！
    标准模式二进制:     target/release/my-fast-npc
    高耦合模式二进制:   target/release/my-fast-npc-monolith
```

### 3.1 关键约束

1. **默认始终为「标准模式」**：「是否启用开发者编译选项」默认 **`N`**；进入后默认选中 **标准模式**。
2. **`-monolith` 后缀**：高耦合 Release 二进制名与标准产物区分，**禁止覆盖**默认 `target/release/<name>`。
3. **两版本可并存**：同一 `Cargo.toml` 通过 **`feature = "monolith"`** 与/或第二 **`[[bin]]`** / 不同输出名构建；CI 与本地可分别 `cargo build` / `cargo build --features monolith`。

---

## 4. 与 `monolith.toml` 的结合

### 4.1 文件职责

| 文件 | 生命周期 | 说明 |
|------|-----------|------|
| **`monolith.toml`** | **仅编译期** | 由 **`oclive init`** 生成/更新；**`oclive build`** 读取；**不**随角色包分发，**不**被宿主 `load_role` 解析。 |
| **`pipeline.ocblueprint` 等** | **运行时**（若使用蓝图） | 与 Monolith **解耦**；焊接范围 **不** 放在蓝图 JSON 内，避免与 `PIPELINE_SCHEMA` 演进绑死。 |

### 4.2 建议 TOML 格式（二选一）

**显式焊接列表**（`exclude` 须为空数组）：

```toml
[monolith]
enabled = true
weld_modules = ["emotion", "memory", "prompt", "llm"]
exclude = []
```

**全焊再排除**（`weld_modules` 须为空数组）：

```toml
[monolith]
enabled = true
weld_modules = []
exclude = ["event", "agent", "complex_emotion"]
```

### 4.3 字段语义

| 字段 | 语义 |
|------|------|
| **`enabled`** | 是否为该项目启用 Monolith 编译路径（生成 `process_message_monolith.rs` 或等价物，并打开 `monolith` feature 相关 cfg）。 |
| **`weld_modules`** | 参与焊接的键名列表（**七焊接键**：第 1–6 模块对应键 + `complex_emotion` 设施焊接键）。**空数组** 表示 **全部可焊接键均参与焊接**（再应用 `exclude`）。 |
| **`exclude`** | 当且仅当 **`weld_modules` 为空** 时有效：从「全槽焊接」集合中排除；被排除槽在生成代码中走 **`PluginHost` + trait** 占位路径。 |

**校验（当前实现）**：**`weld_modules` 与 `exclude` 不得同时非空**；槽名须为七键之一。`enabled == false` 时 **`oclive build`** 仍会再生成 `process_message_monolith.rs`，但 **跳过** 第二次 `cargo build --features monolith`。

### 4.4 脚手架与构建行为

- **`oclive init`**：在开发者选择高耦合时 **写入** `monolith.toml` 与 vendor 片段模板。
- **`oclive build`**（`cargo run -p oclive-cli -- build`）：读取 `monolith.toml`，校验、生成 `process_message_monolith.rs` 与 `vendor/oclive_monolith_builtin`；默认再执行 **`cargo build`** 两次（若 `enabled = true` 则第二次带 **`monolith`** feature）。`--no-cargo` 仅生成；`--release` 与 `--features` 传给 `cargo build`。
- **`oclive bench`**：再生成 + 双构建后，对两个二进制各跑 `--runs` 次子进程（子进程内 `OCLIVE_KERNEL_BENCH_ITERS` 次热循环），输出 JSON（见 Schema）。

---

## 5. 技术方案概要

### 5.1 `Cargo` feature：`monolith`

- 新增 **`monolith`** feature（名称可最终裁定，本文统一用 `monolith`）。
- **未启用**：`process_message` 保持 **`pl.<module>…`** trait 虚调用。
- **启用**：对被焊接模块，改为 **直接调用具体实现**（示意）：

```rust
// 低耦合（现状）
pl.emotion.analyze(&input);
pl.memory.rank_memories(&input);

// 高耦合（目标形态示意；符号以稳定焊接 API 为准）
oclive_emotion_builtin::analyze(&input);
oclive_memory_builtin::rank_memories(&input);
```

未参与焊接的模块 **仍** 通过 **`PluginHost` + trait**。

### 5.2 由 `monolith.toml` 驱动的代码生成

- **`oclive init`** / **`oclive build`** 根据 **`weld_modules`** 与 **`exclude`** 生成或刷新 **`src/process_message_monolith.rs`**（或等价模块）：
  - 被焊接子系统：静态调用；
  - 其余子系统：保留 `pl.*`；
  - 主入口通过 **`cfg(feature = "monolith")`**、`include!` 或薄封装切换，避免完全复制整文件（实现细节待定）。

### 5.3 与主仓关系

主仓需 eventually 提供 **稳定的焊接符号边界**（公开 `fn` 或小门面 crate），否则生成项目只能依赖 `pub(crate)` 内部实现，升级成本高（见第 8 节）。

---

## 6. 脚手架生成的目录结构（目标）

```text
my-fast-npc/
├── monolith.toml              # 编译期配置（init 生成，build 读取）
├── Cargo.toml                 # 含 [features] monolith = … 等
├── src/
│   ├── main.rs
│   └── process_message_monolith.rs   # 由 monolith.toml 驱动生成（实现阶段）
├── target/release/            # 本地构建示意（通常不提交）
│   ├── my-fast-npc
│   └── my-fast-npc-monolith
└── README.md                  # 说明两构建目标、feature、与 RFC 链接
```

---

## 7. 预期收益

- 消除 **被焊接模块** 在热路径上的 **trait 虚调用**（每轮 `process_message` 视编排深度减少若干次动态分发）。
- 焊接边界内 **DTO 可栈传递**，减少堆分配与跨边界拷贝。
- **锁合并为单锁**（或缩小临界区）在部分模块组合下可行，**非自动保证**，需审计。
- **二进制更小**：未使用后端实现可通过裁剪与 LTO 获益（幅度依赖具体链接单元）。

具体数字以 **`oclive bench`**（第 9 节）实测为准，本 RFC **不作承诺**。

---

## 8. 风险与局限性

- **被焊接模块无法热替换**：不能通过插件市场或 `settings.json` 切换为 remote/directory；变更须 **重新编译**。
- **双路径维护成本**：`monolith` 与默认路径需同步演进，测试矩阵膨胀。
- **性能提升需基准验证**：网络 I/O 或 LLM 主导时，焊接收益可能不明显。
- **API 与符号稳定性**：直接绑定实现会增加升级耦合，需独立「焊接 API 层」设计。

---

## 9. 路线图状态（与主分支实现同步）

| 阶段 | 状态 |
|------|------|
| **第一阶段（脚手架）** | **已完成**：`oclive-cli init` 支持 **`--monolith`**（非交互）与交互「开发者编译选项」；生成 **`monolith.toml`**、**`vendor/oclive_monolith_builtin/`**（七焊接键静态入口的**权威来源**）、**`process_message_monolith.rs`**；**双 `[[bin]]`** 且 **标准入口 `src/main.rs` / Monolith 入口 `src/main_monolith.rs`**（避免 Cargo 同路径警告）；`cargo test -p oclive-cli` 含 release 双构建 E2E。 |
| **第二阶段（自定义焊接）** | **已完成**：**`weld_modules` / `exclude`** 互斥校验、部分焊接代码生成；**`oclive build`** 读取 `monolith.toml` 再生成焊接源码并可选双构建。 |
| **第三阶段（`oclive bench`）** | **已完成**：**`oclive bench`** 对比标准与 **`-monolith`** 二进制，输出 JSON（Schema：`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`）。 |
| **第四阶段（真实符号）** | **已完成（vendor 路径）**：已焊接键静态链接 **`oclive_monolith_builtin`**；主仓 **不** 在 `src-tauri` 另起一套焊接桩。后续若拆分 **`oclive_*_builtin`** crate，仅替换生成项目的依赖与调用点，脚手架 **仍** 以 `kernel/crates/oclive-cli/monolith_vendor/oclive_monolith_builtin/` 为模板权威来源。 |

---

## 10. 验收标准（本 RFC 自身）

- [x] 第 1–9 节齐全：问题、目标、交互、`monolith.toml` 格式与语义、技术概要、目录结构、收益、风险、路线图。
- [x] **与蓝图边界明确**：焊接配置 **`monolith.toml` 专属**；**不**要求向 `pipeline.ocblueprint` 增加 `monolith` JSON 段（若未来合并，须另开修订 RFC 并更新 `PIPELINE_SCHEMA`）。
- [x] 终端流程写清：**默认标准模式**、**`-monolith` 双产物**。
- [x] **第一阶段脚手架**：`oclive-cli` 已实现 **`--monolith`**、交互开发者选项、**`process_message_monolith.rs`**、**`main.rs` + `main_monolith.rs`** 双入口与 **`cargo test -p oclive-cli`** 中 release 双构建验证。

---

## 11. 参考与索引

- 脚手架：[`creator-docs/cli/OCLIVE_CLI_GUIDE.md`](../cli/OCLIVE_CLI_GUIDE.md)
- 第 1–6 模块契约：[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md) · 编号总览：[`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)
- 蓝图 Schema（运行时）：[`creator-docs/kernel/PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md)

讨论请链接本文：`creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md`。

---

[English](../../creator-docs-en/rfc/RFC_OCLIVE_MONOLITH_MODE.md)
