# RFC：oclive 高耦合编译模式（Monolith / 焊接模式）

| 元数据 | 值 |
|--------|-----|
| 状态 | 草案（设计讨论，尚未实现） |
| 入口 | **`oclive init` 脚手架**，终端交互，**不**进入任何 GUI 设置 |
| 结合点 | **`pipeline.ocblueprint`**（蓝图）：开发者标记哪些模块参与「焊接」 |
| 受众 | **仅开发者**；普通用户只使用开发者构建的发行版，不接触终端 |
| 本质 | **编译期**优化路径：以模块可替换性换取极限性能（可选、默认关闭） |

**相关文档**：[`PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md)（蓝图 Schema）、[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)（七槽与 `PluginHost`）、[`OCLIVE_CLI_GUIDE.md`](../cli/OCLIVE_CLI_GUIDE.md)（脚手架现状）。

---

## 1. 问题陈述

oclive 的 **七槽插件化架构**（`plugin_backends` + `PluginHost` + trait 门面）在提供 **生态灵活性**（builtin / remote / directory、侧车、目录插件）的同时，引入 **架构性开销**：

- **trait 虚调用**：`process_message` 编排链上多次动态分发；
- **模块间 DTO**：边界处为通用性往往走堆分配与拷贝路径；
- **锁与运行时结构**：各子系统独立持有资源时，锁粒度偏细，高频路径上竞争成本可见。

对 **不需要模块可替换性** 的高频场景（例如：游戏 NPC、实时语音管线、嵌入式玩偶内核），上述开销构成 **oclive 架构本身的天花板**：不是单点算法慢，而是 **抽象边界** 的累积成本。

本 RFC 提出面向开发者的 **高耦合编译模式（Monolith / 焊接模式）**：在 **`oclive init`** 脚手架中可选启用，将 **选定模块** 在编译期 **焊接** 为单一路径，**消除被焊接模块的 trait 虚调用**，使 LLVM 能对静态调用做激进内联；未焊接模块仍走现有 `PluginHost` 路径，保持渐进迁移与混合部署。

---

## 2. 设计目标

| 目标 | 说明 |
|------|------|
| **入口隔离** | 仅在 **`oclive init`**（及后续 **`oclive build`** 类命令）终端流程中出现；**不**写入桌面应用「设置」页，不打扰终端用户以外的受众。 |
| **与蓝图结合** | 焊接范围由 **`pipeline.ocblueprint`**（或等价蓝图文件）中的 **`monolith`** 段声明；脚手架可读取、校验、引导编辑。 |
| **编译期优化** | 生成（或条件编译）**第二套**编排实现：被焊接模块改为 **直接调用具体 crate / 函数**，不再经 `dyn Trait`。 |
| **发行版语义** | 普通用户只安装/运行开发者提供的 **已优化二进制**；是否启用焊接、焊接哪些模块，全是 **开发者构建时** 的选择。 |
| **默认安全** | **默认始终为标准模式**（低耦合、可替换）；高耦合为 **显式二次确认**，不默认勾选。 |

---

## 3. 脚手架交互设计

以下为 **`oclive init`** 在实现本 RFC 后的 **目标交互**（文案可随产品 tone 调整，行为不变）。

```text
$ oclive init my-fast-npc
  欢迎使用 oclive 脚手架！
  ? 项目名称: my-fast-npc
  ? 选择角色包: (从本地或市场选择)
  ? 选择后端配置: (7 槽逐个确认)
  ? 是否启用开发者编译选项? [y/N]: n
  …（标准流程结束）
```

当开发者选择 **`y`** 进入开发者编译选项：

```text
  ── 开发者编译选项 ──
  ? 编译模式:
    ● 标准模式（低耦合，保留模块可替换性，推荐）
    ○ 高耦合模式（焊接选定模块，消除虚调用，极限性能）
```

选择 **高耦合模式** 后：

```text
  ? 焊接范围:
    ● 全部模块（推荐：最大性能提升；与当前预设后端一致的全部槽位）
    ○ 自定义（通过蓝图指定焊接范围）
```

选择 **自定义** 后：

```text
  ? 蓝图路径: ./my_fast_npc.ocblueprint
  （脚手架校验 meta / pipeline.monolith，必要时打开编辑器或打印模板片段）
```

生成结束时的 **提示目标**（两产物并存、可对比）：

```text
  项目已创建！
    标准模式二进制:     target/release/my-fast-npc
    高耦合模式二进制:   target/release/my-fast-npc-monolith
```

### 3.1 关键设计约束

1. **默认选项始终是「标准模式」**：首屏与开发者选项第一层均 **不** 预选高耦合，避免误触。
2. **高耦合产物命名**：Release 二进制名加 **`-monolith` 后缀**（或与 `[[bin]]` name 区分），**禁止覆盖**标准 `target/release/<name>`，便于 A/B 与回归。
3. **两版本可并存**：同一 `Cargo.toml` 下通过 **`feature = "monolith"`**（名称可最终裁定）与/或 **第二 bin target** 生成两条链接单元；CI 与本地均可分别 `cargo build` / `cargo build --features monolith`。
4. **与现有 `oclive-cli` 对齐**：本 RFC 不强制立即改名二进制；实现阶段以 **`cargo run -p oclive-cli -- init`** 为入口即可。

---

## 4. 与蓝图的结合

蓝图（示例文件名 **`oclive.ocblueprint`** 或仓库既有 **`pipeline.ocblueprint`** 约定）在 **`pipeline`** 下新增 **`monolith`** 段，用于声明焊接策略。

### 4.1 建议 JSON 形状

```json
{
  "meta": {
    "name": "my-fast-npc",
    "oclive_version": "0.2.0"
  },
  "pipeline": {
    "monolith": {
      "enabled": true,
      "weld_modules": ["emotion", "memory", "prompt", "llm"],
      "exclude": ["event", "agent", "complex_emotion"]
    }
  }
}
```

### 4.2 字段语义

| 字段 | 语义 |
|------|------|
| **`monolith.enabled`** | 是否为该项目启用高耦合编译路径（生成第二套源码或 cfg 分支）。 |
| **`weld_modules`** | 参与焊接的 **模块名列表**（与七槽 / 编排子系统命名对齐，如 `memory`、`emotion`、`event`、`prompt`、`llm`、`agent`、`complex_emotion`）。**空数组** 表示 **全部可焊接槽位均焊接**（仍尊重 `exclude`）。 |
| **`exclude`** | 从焊接集合中 **排除** 的模块；这些模块在 `process_message` 中 **仍经** `PluginHost` + trait。 |

**解析优先级（建议）**：

- 若 **`enabled == false`**：脚手架只生成标准路径，忽略 `weld_modules` / `exclude`。
- 若 **`enabled == true`** 且 **`weld_modules` 非空**：仅焊接列表内模块；`exclude` 用于从「默认全焊」中扣减（实现时可二选一语义，**必须在 SCHEMA 中单义化**，避免「列表 + exclude」双源冲突）。
- 若 **`enabled == true`** 且 **`weld_modules` 为空**：表示 **全模块焊接**，再应用 **`exclude`** 扣减。

### 4.3 脚手架行为（相对本 RFC）

| 场景 | 行为 |
|------|------|
| 蓝图已含 **`pipeline.monolith`** | `oclive init` 在「开发者编译选项」中 **检测并展示摘要**（已启用、焊接列表、排除列表），询问是否与交互选择对齐。 |
| 用户选择「自定义焊接」 | 引导输入/确认蓝图路径；若文件不存在，可生成 **带 `monolith` 段的模板** 并提示下一步编辑。 |
| 蓝图与 CLI 选择冲突 | **以蓝图为真** 或 **以 CLI 覆盖蓝图** 须在实现时二选一并写进 SCHEMA；本 RFC 推荐 **蓝图为真**、CLI 仅作首次生成的默认值填充。 |

**Schema 演进**：正式字段名、默认值与校验规则应合并进 [`PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md) 及对应 Rust 校验器（实现阶段任务）。

---

## 5. 技术方案概要

### 5.1 编译期：`Cargo` feature

- 新增 **`monolith`**（或 `oclive_monolith`，名称待定）**feature**。
- **关闭**：`process_message`（及共景/异地分支）保持现有 **`pl.<module>.…`** trait 动态分发。
- **启用**：对被焊接模块，生成或 `cfg` 切换为 **直接调用具体实现** 的静态路径，例如：

```rust
// 低耦合（现状）：trait 虚调用
pl.emotion.analyze(&input);
pl.memory.rank_memories(&input);

// 高耦合（目标形态示意；crate/路径为示例）
oclive_emotion_builtin::analyze(&input);
oclive_memory_builtin::rank_memories(&input);
```

LLVM 可对上述 **静态调用** 做内联、去虚拟化；未焊接模块仍通过 **`PluginHost`** 解析。

### 5.2 蓝图驱动的代码生成

- **`oclive init`**（或子命令 **`oclive codegen monolith`**）根据蓝图 **`weld_modules` / `exclude`** 生成 **`src/process_message_monolith.rs`**（或等价模块），其中：
  - **仅**被焊接子系统使用直接调用；
  - 其余子系统 **保留** `pl.*` trait 调用；
  - 与 **`process_message`** 主入口通过 **`cfg(feature = "monolith")`** 或 **`include!`** 组合，避免维护两套完全分叉的巨型文件（实现细节待定）。

### 5.3 与内核仓库的关系

- **第一阶段**可在 **脚手架生成的独立 crate** 内用 **桩实现** 验证链接与 feature 矩阵；
- **第二阶段**需 **`oclivenewnew` 主仓** 暴露稳定的 **「具体实现」符号边界**（或 `pub fn` 门面），否则焊接只能指向 `pub(crate)` 内部符号，难以跨 crate 稳定链接。

---

## 6. 脚手架生成的目录结构（目标）

```text
my-fast-npc/
├── oclive.ocblueprint          # 蓝图（含 pipeline.monolith；文件名可与现有 pipeline 约定对齐）
├── Cargo.toml                  # 含 [features] monolith = [...] 与可选第二 bin
├── src/
│   ├── main.rs                 # 入口
│   └── process_message_monolith.rs   # 由蓝图生成或模板渲染（实现阶段）
├── target/release/             # 本地构建产物（示意）
│   ├── my-fast-npc             # 标准模式
│   └── my-fast-npc-monolith    # 高耦合模式（或同名不同 output 目录，实现时统一）
└── README.md                   # 说明两构建目标、feature 与性能对比指引
```

**说明**：`target/` 通常不提交版本库；上表强调 **开发者工作区** 的预期布局。

---

## 7. 预期收益

| 方向 | 预期（需基准验证） |
|------|---------------------|
| **动态分发** | 被焊接模块在热路径上 **消除** `dyn` 虚调用（每轮 `process_message` 可减少数次至十余次量级，视编排深度而定）。 |
| **DTO 与分配** | 焊接边界内可改为 **栈传递 / 专用结构体**，减少堆分配与序列化式拷贝（具体取决于各模块 API 改造幅度）。 |
| **锁** | 有机会将多锁合并为 **单锁** 或缩小临界区（需逐模块审计，非自动保证）。 |
| **二进制体积** | 未参与焊接的后端实现可通过 **dead code elimination** 与 feature 裁剪减小体积（收益与 LTO、泛型单态化相关）。 |

所有数字目标 **不作为本 RFC 承诺**；以 **`oclive bench`**（见第 9 节）实测为准。

---

## 8. 风险与局限

| 风险 | 说明 |
|------|------|
| **可替换性丧失** | 焊接模块 **不能** 再通过插件市场 / `settings.json` 切换为 remote/directory；变更需 **重新编译**。 |
| **双路径维护** | `monolith` feature 下与默认路径 **分支增多**，测试矩阵膨胀；需严格 **cfg 测试** 与共享逻辑抽取，避免漂移。 |
| **API 稳定性** | 直接调用具体实现会 **绑定** 内部符号；需定义 **稳定的「焊接 API 层」**，否则升级内核时开发者项目大面积破损。 |
| **收益不确定** | 虚调用成本占比随场景变化大；嵌入式 I/O 或 LLM 网络延迟主导时，焊接收益可能 **不明显**。 |
| **安全与审计** | 高耦合路径若绕过现有权限/目录插件门禁，需 **显式文档** 与 **构建时警告**（实现阶段要求）。 |

---

## 9. 后续计划（分阶段）

| 阶段 | 内容 |
|------|------|
| **第一阶段** | 在 **`oclive init`** 中增加「开发者编译选项」骨架：**全模块焊接** 单一预设；生成 `Cargo.toml` feature、`README` 与 **占位** `process_message_monolith.rs`；主仓可先不接真实符号。 |
| **第二阶段** | 支持 **蓝图驱动** 的 `weld_modules` / `exclude`；与 [`PIPELINE_SCHEMA.md`](../kernel/PIPELINE_SCHEMA.md) 对齐校验；`init` 检测蓝图并合并提示。 |
| **第三阶段** | 提供 **`oclive bench`**（或 `cargo xtask bench-monolith`）：同一 fixture 下对比 **标准二进制** 与 **`-monolith`** 的延迟/吞吐/分配计数；输出可上传 CI artifact。 |

---

## 10. 验收标准（本 RFC 自身）

- [x] 文档覆盖 **第 1–9 节** 全部主题：问题、目标、交互、蓝图字段、技术概要、目录结构、收益、风险、路线图。
- [x] **终端交互流程** 以「默认标准模式 + 显式进入高耦合 + 双产物命名」写清。
- [x] **与蓝图的结合点** 以 JSON 示例 + 字段语义表 + 脚手架行为表 **明确定义**。
- [x] 与现有 **`plugin_backends` 七槽**、**`PluginHost`**、**蓝图 pipeline** 文档 **交叉引用**，避免与 GUI 设置混淆。

---

## 11. 参考与索引

- 脚手架：`creator-docs/cli/OCLIVE_CLI_GUIDE.md`
- 插件后端契约：`creator-docs/plugin-and-architecture/PLUGIN_V1.md`
- 蓝图 Schema：`creator-docs/kernel/PIPELINE_SCHEMA.md`

如需讨论本 RFC，请在 PR 或 issue 中链接本文路径：`creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md`。
