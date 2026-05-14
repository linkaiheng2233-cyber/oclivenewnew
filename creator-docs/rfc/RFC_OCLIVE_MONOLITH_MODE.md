# RFC：oclive 高耦合编译模式（Monolith / 焊接模式）

| 元数据 | 值 |
|--------|-----|
| 状态 | **第一阶段已实现**（`oclive-cli`：`--monolith` / 交互开发者选项；占位焊接源码；真实 `oclive_*_builtin` 接入为后续工作） |
| 入口 | **`oclive init` 脚手架**，终端交互，**不**进入任何 GUI 设置 |
| 编译配置 | **`monolith.toml`**：由 `init` 生成，`oclive build`（或等价构建脚本）读取；**不参与运行时**，与角色包/宿主加载路径无关 |
| 与蓝图边界 | **`pipeline.ocblueprint`**（或 `*.ocblueprint`）描述 **运行时** 编排；**焊接范围不以蓝图字段承载**（避免与 `PIPELINE_SCHEMA` 运行时语义混淆）。Monolith 仅通过 **`monolith.toml` + Cargo feature** 生效。 |
| 受众 | **仅开发者**；普通用户只使用开发者构建的发行版，不接触终端 |
| 本质 | **编译期**优化路径：以模块可替换性换取极限性能（可选、默认关闭），用于打破七槽抽象在 **高频热路径** 上的性能天花板 |

**相关文档**：[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)（七槽与 `PluginHost`）、[`PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md)（蓝图运行时 Schema）、[`OCLIVE_CLI_GUIDE.md`](../cli/OCLIVE_CLI_GUIDE.md)（脚手架用法）。

**第一阶段落地（与实现一致）**：`cargo run -p oclive-cli -- init … --monolith`（非交互，且项目类型为 **kernel_server**）或交互末尾选择 **高耦合模式 → 全部模块** 时，生成根目录 **`monolith.toml`**、`src/process_message_monolith.rs`，并在 `Cargo.toml` 中声明 **`[features] monolith`** 与第二 **`[[bin]]`**（`{package}-monolith`，`required-features = ["monolith"]`）。占位实现为 **同 crate 内静态 `welded_*` 模块**，保证 `cargo build --release` 与 `cargo build --release --features monolith` 均可通过；**`oclive build` 独立子命令尚未实现**，当前以 **`cargo build`** 为读取/编译入口。嵌入式 **library** 项目忽略 `--monolith` 且不生成 `monolith.toml`。

## 1. 问题陈述

oclive 的 **七槽插件化架构**（`plugin_backends` + `PluginHost` + trait 门面）在提供 **生态灵活性**（builtin / remote / directory、侧车、目录插件）的同时，引入 **架构性开销**：

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
  …（角色包、七槽后端等既有步骤）
  ? 是否启用开发者编译选项? [y/N]: y
```

进入开发者编译选项后：

```text
  ── 开发者编译选项 ──
  ? 编译模式:
    ● 标准模式（低耦合，保留模块可替换性，推荐）
    ○ 高耦合模式（焊接选定模块，消除虚调用，极限性能）
```

选择 **高耦合模式** 后（示意）：

```text
  ? 焊接范围:
    ● 全部模块（推荐：最大性能提升）
    ○ 自定义（编辑 monolith.toml 中的 weld_modules / exclude）
  ? monolith.toml 路径: ./monolith.toml
  （若选自定义：脚手架生成模板并提示保存路径）
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

### 4.2 建议 TOML 格式

```toml
[monolith]
enabled = true
weld_modules = ["emotion", "memory", "prompt", "llm"]
exclude = ["event", "agent", "complex_emotion"]
```

### 4.3 字段语义

| 字段 | 语义 |
|------|------|
| **`enabled`** | 是否为该项目启用 Monolith 编译路径（生成 `process_message_monolith.rs` 或等价物，并打开 `monolith` feature 相关 cfg）。 |
| **`weld_modules`** | 参与焊接的模块名列表（与七槽命名对齐：`memory`、`emotion`、`event`、`prompt`、`llm`、`agent`、`complex_emotion`）。**空数组** 表示 **全部可焊接槽位均参与焊接**（再应用 `exclude`）。 |
| **`exclude`** | 从焊接集合中排除的模块；这些模块在编排中 **仍通过** `PluginHost` + trait 调用。 |

**解析优先级（实现时须单义化）**：建议 **`enabled == false`** 时忽略 `weld_modules` / `exclude`；**`enabled == true`** 且 **`weld_modules` 为空** 表示「全焊再减 `exclude`」；非空 `weld_modules` 表示「仅焊列表内模块」（`exclude` 可从列表中再剔除，或禁止与非空列表并用，二选一文档化）。

### 4.4 脚手架与构建行为

- **`oclive init`**：在开发者选择高耦合时 **写入** `monolith.toml`（或合并已有文件）；标准模式可生成 **`enabled = false`** 的占位文件以便发现文档链接。
- **`oclive build`**：读取 `monolith.toml`，决定 feature、生成片段及链接目标名（含 `-monolith`）。

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

## 9. 后续计划

| 阶段 | 内容 |
|------|------|
| **第一阶段** | **已完成（脚手架）**：`oclive-cli init` 支持 **`--monolith`**（非交互）与交互式「开发者编译选项」；生成 **`monolith.toml`**（`weld_modules = []` 表示全槽占位焊接）、**`process_message_monolith.rs`**（七槽 `welded_*` 静态桩）、**双 `[[bin]]`**；`cargo test -p oclive-cli` 含 release 双构建 E2E。占位代码 **尚未** 链接真实 `oclive_*_builtin`。 |
| **第二阶段** | 支持 **`weld_modules` / `exclude`** 自定义与 `oclive build` 代码生成子命令；消费 `monolith.toml` 生成差异化焊接。 |
| **第三阶段** | **`oclive bench`**：同一 fixture 对比标准二进制与 **`-monolith`** 产物（延迟、吞吐、分配）。 |

---

## 10. 验收标准（本 RFC 自身）

- [x] 第 1–9 节齐全：问题、目标、交互、`monolith.toml` 格式与语义、技术概要、目录结构、收益、风险、路线图。
- [x] **与蓝图边界明确**：焊接配置 **`monolith.toml` 专属**；**不**要求向 `pipeline.ocblueprint` 增加 `monolith` JSON 段（若未来合并，须另开修订 RFC 并更新 `PIPELINE_SCHEMA`）。
- [x] 终端流程写清：**默认标准模式**、**`-monolith` 双产物**。
- [x] **第一阶段脚手架**：`oclive-cli` 已实现 **`--monolith`**、交互开发者选项、占位 **`process_message_monolith.rs`** 与 **`cargo test -p oclive-cli`** 中 release 双构建验证。

---

## 11. 参考与索引

- 脚手架：[`creator-docs/cli/OCLIVE_CLI_GUIDE.md`](../cli/OCLIVE_CLI_GUIDE.md)
- 七槽契约：[`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)
- 蓝图 Schema（运行时）：[`creator-docs/kernel/PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md)

讨论请链接本文：`creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md`。
