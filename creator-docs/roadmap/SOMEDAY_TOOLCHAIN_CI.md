# OCLive 领域感知 CI · 分阶段实施基线

> **状态（2026-08-21）**：Stage 3.1 以 `domain-aware-pr-v2` 把领域规划真正用于开发期 PR：无 warning、无 full fallback、非 shadow 且范围/job 非空的**草稿 PR**按计划执行，并只发布不能满足分支保护的 `ci-draft-gate`；转为 ready 时同一提交由 `ready_for_review` 重新触发，除已有真实 Compare 支撑的纯文档 Canary 外一律全量并发布正式 `ci-gate`。Push、高风险/未知路径和规划器异常仍全量。每次全量 run 会自动生成同一 workflow SHA 的 Compare JSON/Markdown，保留 90 天；选择性 run 只记执行证据，永不冒充漏选对照。影子语料已扩为 **21** 场景（**18 targeted / 3 fail-safe**），并覆盖中文/NUL 路径、混合模块、merge 策略与 Dependabot 节奏。五个 `nightly` 责任组继续留在独立定时/手动工作流。本文是 CI 影响规划的设计 SSOT；模块边界只在 [`MODULE_MAP_AND_HANDOFF.md` §12.7](../../handoff/MODULE_MAP_AND_HANDOFF.md#127-ci-影响元数据与脚手架边界) 登记，执行证据见 [`TECHNICAL_DEBT_INVENTORY.md` K-CI-IMPACT-01](../../handoff/TECHNICAL_DEBT_INVENTORY.md)，核实口径以 [`AI_VERIFICATION_PROTOCOL.md`](../../handoff/AI_VERIFICATION_PROTOCOL.md) 为准。

OCLive 采用成熟 CI 的分层、测试金字塔和合并门禁，并增加一层领域感知规划器。目标不是让模型猜测该跑什么，也不是立刻删除全量检查，而是先用确定性元数据回答：一次改动直接落在哪些模块、经哪些契约传播、需要哪些受信验证。

## 1. 总体结构

```text
git changed paths
        ↓ 只做直接归属定位
central path bindings + oclive.module.json
        ↓
central impact policy + declared_affects
        ↓
affected modules + validation_profiles
        ↓
trusted validation catalog
        ↓
plan.json + Job Summary
```

这是**路径定位 + 语义传播**的混合模型：完全不看路径无法知道本次改了哪个模块；只看路径又无法表达协议、宿主、发行版和插件之间的波及关系。分析器必须确定性运行，不调用 LLM；无法解释的路径、未知必需扩展或损坏的元数据一律 fail-safe 到当前策略下的全量验证。

## 2. 三类受版本控制的输入

### 2.1 模块描述 `oclive.module.json`

模块描述只陈述事实和引用坐标，不包含 shell、工作流或触发器：

| 字段 | 责任 | 不负责 |
|------|------|--------|
| `runtime_requires` | 运行时必需的逻辑能力或服务 | GPU/RAM 数值与 CI 环境安装 |
| `resource_claims` | 对共享 GPU、RAM、CPU、渲染或受管进程的资源需求 | 直接支配 Resource Coordinator |
| `declared_affects` | 模块维护者声明的潜在下游影响 | 覆盖或删除中央强制影响边 |
| `validation_profiles` | 引用官方验证配置 ID | 自定义命令、secret、runner 或并行度 |
| `extensions` | 命名空间扩展；声明 required/optional | 向根 schema 无限加字段 |

Stage 1 先为仓内领域模块建立描述；既有模块允许渐进迁移，新模块进入主仓后必须有描述或被中央映射明确归入已有模块。未知 required 扩展使计划失败并转全量；未知 optional 扩展保留并告警。

### 2.2 中央影响图

中央影响图由 OCLive 维护者拥有，包含：

- 路径到直接模块的精确/前缀绑定；
- 维护者强制的影响边和高风险覆盖规则；
- 未知路径时的全量回退配置；
- 当前受支持的扩展命名空间。

最终影响图是“中央强制边 ∪ 合法的 `declared_affects`”。第三方声明只能增加审查范围，不能缩小中央范围。图允许环，规划器必须用 visited 集合计算稳定闭包。

### 2.3 受信验证目录

验证目录将 `validation_profile` 展开为验证器，并区分：

- `tier`：`fast`、`pr`、`merge`、`nightly`、`release`；
- `gate`：`required`、`advisory`、`quarantined`；
- 平台与信任级别；
- 由主仓维护的本地复现 `command_id`；
- 对应现有远端编排的 `workflow_jobs`。

模块只能引用验证器坐标。`command_id` 是有限、可终止的本地复现入口，`workflow_jobs` 是当前远端 CI 的实际编排映射；规划器不执行命令，Stage 3 也只允许主仓工作流按这些受信坐标选择既有 job。命令、工作目录、secret、runner、缓存、并发和超时始终归主仓工作流/验证目录所有。第三方提交到主仓时，其自测只能通过已审核坐标受限执行；插件自带 `.github/workflows/*` 不参与主仓编排。Fork/二次发行可自行维护 CI，主仓不对其流水线负责。

### 2.4 主工作流的执行所有权

影响规划回答“应验证什么”，但不能用重复执行换取表面安全。主工作流为高成本验证分配唯一所有者：通用 Rust job 覆盖非 CLI workspace，CLI job 独占需要嵌套 Cargo build 的串行 E2E，Dimension 5 独占 `cargo audit`，前端 job 显式持有 lint、Vue/TypeScript 类型检查、单测与构建。验证目录可以让多个验证器坐标映射到同一个受信 job，但不得为了坐标一一对应而重复运行同一命令。

这项去重与选择性执行仍是两层控制：Stage 3.1 允许安全的草稿 PR 按计划选择责任组，用于开发反馈；ready PR 只有纯文档 Canary 可以继续选择性执行，其他 ready PR 与所有 Push 都运行全部主 CI 责任组。规划器 warning/full fallback/异常在任何状态都强制全量。目录中 `tier=nightly` 的责任组继续进入独立 Nightly/手动通道。每次调整执行所有权都必须同时更新工作流、验证目录、本地复现命令和仓库契约测试；在远端证据确认前，不以本地冷/热缓存耗时推断最终收益。

#### 后续责任分层的调查输入（未实施）

当前的 job 所有权去重已生效；它尚未证明每个验证器都按 Kernel 与可变化 Host 的责任边界归属。下一轮评估以 **Kernel 自身公共契约 → 通用 Host 接入契约 → 具体 Host / 发行版行为 → 发布组合** 四层逐项标注现有 validator、job 与真实调用者。Kernel 层只承诺其公共能力，不把某个 7B 或 Chat Pro Host 的业务路径当作所有 Host 的代表；通用接入层检验合法 Host 能否满足接口；具体 Host 维护自己的扩展和业务测试；组合测试在目标发行/集成里验收。这是待核的归属模型，不预判现有 CI 已错误归类，也不改变当前 `ci-gate`、Stage 3.1 选择范围或 Push 全量策略。

调查先覆盖现有主 CI 责任组的拥有者、跨层依赖与已有 Compare 的漏选/过选候选，再挑能改变路由决策的少数边界追证；记录未判定项即可停止，不为消除低风险不确定性穷尽调用链。只有在同一目标 SHA 的真实 Compare、契约测试及稳定门禁能证明没有漏掉应测范围后，才提出逐类迁移。日常全量运行的节奏控制按 [`AI_VERIFICATION_PROTOCOL.md` §2.4](../../handoff/AI_VERIFICATION_PROTOCOL.md#ci-推送节奏与证据绑定) 执行，不以重构 CI 为当前债务收敛的前置条件。

**2026-10-04 维护者确认的渐进方向**：上述分责按公共承诺与实际依赖划分，模型大小本身不是模块边界。面向特定 7B 模型的提示词、回复修复、情绪表现及资源调优归具体实现或 Host；其质量/性能证据绑定模型版本、运行配置与适用条件，不代替 Kernel 公共契约测试，也不由公共契约通过推导模型质量达标。同一个 LLM 适配器可以同时拥有模型无关的接口/错误测试与模型条件下的质量评测，分别报告承诺、执行条件和结果。

**模块化服务于可验证的归属**：先明确接口、责任者、依赖及可独立执行的测试，实际代码耦合阻碍这些出口时再提出最小拆分，不要求先完成物理拆库。不能只给现有 profile 改名就宣称验证范围已分离，也不能因某测试归属 Host 就删除公共接口变动对它的影响。具体职责仍以 [MODULE_MAP §0.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) 为准，本节不另定 Kernel/Host 架构。

**逐类迁移出口**：每批只选择一个已有实际重复执行或过选依据的范围，核直接归属、公共依赖及跨边界组合，形成可审查的最小变更。仅改模型特化策略时选择对应实现/Host 验证；公共契约变化继续沿依赖传播；发布时验证指定组合。精确选路仍须受前述真实 Compare、契约回归及稳定门禁约束；未知采用既有保守策略，不凭模拟、命名或缺少反例缩减验证。一个范围足以决定路由便结束调查，其余未知另记，不展开全仓测试重分类。这里是后续设计约束，当前工作流、元数据、影响图和执行策略均未改变。

<a id="kernel-host-ci-first-slice"></a>

#### 2026-10-11 首片：公共基础契约的独立检查入口

**性质与止点**：分责准备基于 `24f62303e9bdea37061fe44acc7a0830dc519f98`，本片只实现下述本地入口，未改变CI选路。按现有目录、workflow及直接测试入口，将本次输入收束为下表8组职责；不逐个重分类全仓测试。职责层由[MODULE_MAP §0.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)拥有，本表只判验证出口。跨层治理不是额外的Kernel能力。

| 入选职责组 / 现行入口 | 当前验证归属与直接依据 | 本片结论 / 不外推的范围 |
|---|---|---|
| 公共基础契约；目录中的 `kernel-contracts` / `rust-workspace` | 六Base及最小角色数据是公共面；但[目录](../../data/ci/validation-catalog.v1.json)的profile同时引用Host/组合job，实际 `rust` job先构建前端与Tauri资源，再跑非CLI workspace | 第一片可新增不依赖Host的本地检查入口；整个profile或 `contracts/types` 的全部旧rich导出不能直接等同纯Kernel |
| 共享消费者；runtime测试和外部Memory案例 | [最小消费者测试](../../kernel/crates/oclive_kernel_runtime/tests/minimal_role_consumer.rs)与[外部Memory案例](../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs)验证调用方绑定与真实调用；消费层在小Kernel之外 | 属通用接入的有限案例，单独保留；不把runtime内全部实现移入公共基础检查，也不新增固定六槽回合顺序 |
| 参考Host条件运行；`kernel-host-dual-core` / `rust-arm64-cross` | 前者明确测试Host的可选feature；后者检查server/Host/runtime的目标编译 | 属具体实现和平台条件；现有回归保留，不构成双核解冻或ARM实机验收 |
| 既有Host组合；`cross-host-e2e` / `oocp-test-suite` | [主workflow](../../.github/workflows/ci.yml)实际运行发行版、profile、bundled资源等脚本；cross-host脚本使用参考角色目录和Host进程 | 为现有Host接入/组合提供有限证据；不是任意合法Host的完整合规证明，也不反向定义所有Kernel承诺 |
| 发行版呈现与插件；`frontend` / `plugins-index` / `remote-plugin-demo` | 前端job覆盖shared、ChatPro/Theater与浏览器预览；插件索引和remote示例拥有各自格式/接线检查 | 归具体Host/发行版及适配案例；令牌、IPC、真实音频和外部插件功能按各自目标验证，不能由命名或编译外推 |
| 开发工具；`cli` 与scaffold路径 | CLI默认检查与 `diagnostics-host` 测试在同job分步执行；基础工程E2E验证生成物，Host案例通过显式feature参与 | 工具生成与Host接入分别报告；模板生成成功不是模块功能或发行版验收 |
| 跨层治理；layering / Dimension 5 / cargo、npm audit / stale-paths | 治理检查覆盖仓库分层、依赖、文档、控制面和其它已登记约束；多个validator可由同job执行 | 不塞入某个Host业务层，也不为四层整齐机械拆job；供应链和控制面安全下限保持 |
| 重型条件证据；Nightly的Loom / fuzz / CLI bench / visual / native Tauri | [Nightly](../../.github/workflows/nightly-advisory.yml)有明确目标与独立执行通道，部分覆盖公共数据解析，部分覆盖具体桌面/工具 | tier说明运行节奏，不能代替语义归属；逐项物理拆分与全量性能收益本轮未判定 |

**第一片本地入口：`npm run check:kernel-base`，不改变远端责任组。** 在OCLive源码根目录运行；只需Node（版本按package.json）、Cargo和Clippy，无需先准备前端、桌面资源、模型或数据库。实现见 [`check-kernel-base.mjs`](../../scripts/check-kernel-base.mjs)，复用既有native命令helper，按顺序执行以下四条已有检查；非零、工具缺失或超时即停止并让入口非零，每步上限20分钟。先显示命令，Cargo输出在该步结束后显示。它不是脚手架生成工程的自动验收器，作者仍在自己的工程运行 `cargo check` / `cargo test`。检查链仅为：

```sh
cargo test --locked -p oclive_kernel_contracts --test base_only_fixture
cargo test --locked -p oclive_kernel_types --test minimal_role_contract
cargo clippy --locked -p oclive_kernel_contracts -p oclive_kernel_types --all-targets -- -D warnings
cargo test --locked -p oclive_kernel_contracts -p oclive_kernel_types --doc
```

contracts/types自身没有Host/runtime/Tauri依赖，但仍携带兼容导出；上述Clippy/doctest按当前crate编译面覆盖，不宣称所有内容均属小Kernel。该入口只验证已有Base/最小角色相关契约及编译文档；不覆盖Kernel所有硬边界、模块目标质量、共享消费者运行、Host业务或所有发行版。真实公开面变更仍按[核实协议](../../handoff/AI_VERIFICATION_PROTOCOL.md)显式跑workspace doctest，不能以这个窄入口替代。

**有限实施与后续出口**：同一命令链的成功、失败非零传播及目标crate实际依赖按[本片报告](../../handoff/debt-marathon/waves/WAVE-20261011-KERNEL-BASE-LOCAL-CHECK.md)记录；使用指南见[本地门禁矩阵](../../human-docs/08_PR_GATE_MATRIX.md)。不复制Base测试、生成新DTO/trait或机械补六套包装。达到本地入口可用即停；只有下一片真正需要远端接入时，才同步受信catalog、本地命令、workflow所有权与仓库合同测试。保留旧组合回归；要从原 `rust` job中迁走测试时先证明覆盖与唯一所有权，不能先新增一份重复高成本测试再称分层完成。ready选择类别扩展仍受真实Compare与正式gate约束。未知的更深测试归属、缓存/并发收益和发布矩阵另排，不成为本地入口的前置调查。

### 2.5 执行通道约定

- `.github/workflows/ci.yml`：`ci-impact-plan` 产出执行范围；安全草稿 PR 与 ready 的纯文档 Canary 可选择性执行，其他事件 fail-safe 全量；草稿发布 `ci-draft-gate`，ready/Push 发布受保护的 `ci-gate`，两者都核对 selected/success 与 unselected/skipped；
- `.github/workflows/nightly-advisory.yml`：目录中 `tier=nightly` 的 Loom、fuzz、原生窗口、视觉冒烟和无阈值性能证据；支持每日全跑与按 validator 手动复现；
- Nightly 失败不会阻塞 main，但在 Nightly 内不得 `continue-on-error`；失败日志或 artifact 是待处理证据，不能粉饰为绿；
- 同一 `workflow_jobs` 坐标按 validator 的 `tier` 落入对应受信通道，仓库契约测试防止 job 漂回错误工作流。

<a id="ci-layering-development-handoff"></a>

#### 分层对接：执行归属与选择范围分开迁移

当前已经具备影响规划、受信目录、执行策略和最终gate；下一片优先复用这些设施。需要分清两个问题：**执行归属**回答某项检查由哪个责任组运行；**选择范围**回答某次改动能跳过哪些责任组。把公共基础检查从需要桌面资源的job中分出来，不自动授予PR跳过Host回归的权限。

以下是[主workflow](../../.github/workflows/ci.yml)与[执行策略](../../scripts/ci-execution-policy.mjs)的现行行为，不是新策略：

| 触发 / 条件 | 当前执行范围 | 最终出口 |
|---|---|---|
| 安全草稿PR：非shadow、无warning/fallback且范围与job非空 | 受信计划选中的责任组 | `ci-draft-gate`，不能满足main的required check |
| ready PR，直接模块仅为已晋升的 `oclive.docs` 且满足上述安全条件 | 文档Canary选择范围 | `ci-gate` |
| 其它ready PR | 全部主CI责任组 | `ci-gate` |
| 推送main/master，或手动运行主workflow | 全部主CI责任组 | `ci-gate`；普通备份分支push本身不触发此workflow |
| 规划warning、未知/高风险回退或规划失败 | 全量；规划失败仍使最终gate失败 | 不允许全量检查的通过掩盖规划失败 |

上述表不代替Nightly通道，也不把所有仓库workflow归为同一required gate。本地所有权首片的主workflow有14个验证责任job定义；公共契约、剩余Rust和前端各展开两平台，加上规划与gate，全量配置为19个实际job。已推送main `24f62303` 仍是原17-job配置；新配置尚待集成后的目标SHA正式CI。数量描述结构，不能作为质量或性能目标。

**本地首片：公共契约与兼容检查执行所有权。** 已同步catalog、完整本地命令、workflow和[仓库合同](../../kernel/crates/oclive_ci_plan/tests/repository_contract.rs)，保留当前Push/ready策略与required context，具体集合与迁移规则见[下方责任对照](#ci-responsibility-map)。共有依赖可能仍需编译，编译复用与重复运行同一测试分别判读；新增责任组不等于全部CI分层完成。

**后续单独评估：选择范围。** 执行归属稳定后，才依据目标SHA的真实Compare与契约回归讨论某类ready PR能否选择性执行。不因本地窄测更快、一次主CI通过或四层分责表就移除影响边。缓存、性能矩阵和未判定的细粒度归属不是首片前置调查；开发切片定向验证，同主题里程碑才做适用完整验收。

分类提交与main正式基线见[2026-10-11交接快照](../../handoff/debt-marathon/waves/WAVE-20261011-CI-BASELINE-HANDOFF.md)；本地所有权实施、适用验证与未执行项见[本片报告](../../handoff/debt-marathon/waves/WAVE-20261011-CI-CONTRACT-OWNER.md)。当前改动仅在本地开发分支，未合入/推送main，不借旧SHA的远端绿灯；K-CI-IMPACT-01仍为In progress。

<a id="ci-responsibility-map"></a>

#### 维护者对接：现有检查的责任归属（2026-10-11，本地首片）

架构承诺与实际入口统一见[MODULE_MAP §0.3.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)。原13-job对照冻结于 `5e483aa6`，其依据的workflow与正式main `24f62303` 相同；下表更新为本地首片的[主workflow](../../.github/workflows/ci.yml)14个验证job。crate名、目录名、profile名或执行平台不直接等于架构层。**MIXED** 表示一个执行出口覆盖多个责任，不表示违规或可删。

| 现有验证job | 责任归属 | 实际检查与不能外推的范围 |
|---|---|---|
| `kernel-contracts`（Linux/Windows） | 公共契约与既有兼容面 | 两crate的all-targets/all-features Clippy（`--no-deps`）及全部默认测试/适用doctest；包含rich/Stream等兼容面，不称两crate整体都是小Kernel。无需前端/Tauri资源准备。 |
| `rust`（Linux/Windows） | **MIXED：共享接入、参考Host、桌面及治理** | 前端构建/桌面资源准备、workspace fmt保留；Clippy/tests只排除CLI及迁移的contracts/types根包，剩余检查及平台不变。Clippy `--no-deps` 将lint归给被选择的根包；依赖仍需编译。公共变更仍会选中相关下游回归。 |
| `layering-ratchet` | 跨层治理 | domain分层棘轮；不是完整架构合规证明。 |
| `dimension5-acceptance` | **MIXED：跨层治理＋已有工具/Host/产品约束** | `dimension5 --ci` 包含依赖/文档/分层等与已有定向检查，sample lib按此模式跳过；不是单一Kernel能力。 |
| `npm-audit` | 跨层供应链治理 | npm生产与全依赖审计；不能代替Cargo或发布者身份验证。 |
| `plugins-index` | 插件目录格式/仓库治理 | 插件索引一致性，不是全部模块功能或信任根验收。 |
| `stale-paths` | 跨层路径治理 | 登记的旧路径规则，非全仓语义审查。 |
| `kernel-host-dual-core` | 参考Host条件回归 | Host的可选dual_core feature；已有回归不解冻双核方向。 |
| `cli` | **MIXED：开发工具＋显式参考Host接入** | CLI默认测试与 `diagnostics-host` 的最小角色/资源诊断回归分步运行；基础工程生成与Host功能分开解释。 |
| `rust-arm64-cross` | 参考Host/共享运行代码的目标编译 | server/Host/runtime交叉 `cargo check`；无ARM实机执行证据。 |
| `frontend`（Linux/Windows） | 发行版与其共享前端组件 | shared/ChatPro/Theater lint、类型、单测/构建；Linux另跑浏览器预览，不等于独立浏览器发行版或真实语音验收。 |
| `cross-host-e2e` | 既有参考Host/发行版组合 | 当前跨Host记忆、发行版、profile、bundled资源及VSCode配置对照；不是所有合法自有Host的统一业务要求。 |
| `oocp-test-suite` | 既有桌面/参考Host协议组合 | mock LLM下OOCP和重启检查，不证明真实模型语义质量或所有崩溃窗口。 |
| `remote-plugin-demo` | 参考插件/传输适配案例 | loopback Python `memory.rank` RPC案例，不等同Rust `MemoryBase` 或任意插件/网络验收。 |

公共契约检查已有独立执行出口，共享消费者仍由剩余workspace及既有案例覆盖，尚未另拆job。`ci-impact-plan` 属受信选择控制面，`ci-gate` 属最终聚合，不与14个验证job混计；三组两平台矩阵展开后配置全量19个job。目录里的 `kernel-contracts` / `kernel-runtime` profile用于影响传播，不能因为下游测试归Host就删掉公共变更到它的边。

| 其它现有workflow | 执行责任与证据边界 |
|---|---|
| [Nightly](../../.github/workflows/nightly-advisory.yml) | Loom为具体并发路径；fuzz有公共校验和兼容解析；CLI bench属工具；visual/native Tauri属发行版/桌面。五类按各自责任读，nightly是节奏而非架构层。 |
| [Cargo lockfile audit](../../.github/workflows/cargo-audit-lockfile.yml) | 锁文件PR供应链检查；触发与audit参数不同于Dimension5，不能凭名称当同一重复检查。 |
| [CI rerun](../../.github/workflows/ci-rerun-flake.yml) | 限定失败集合的一次人工重跑设施，非通用自动重试或验证责任。 |
| [Kernel checksums](../../.github/workflows/release-kernel-checksums.yml) | server发布构建、版本manifest、校验和及Actions artifact；不是全部发行版发布、签名或公开发布验收。 |

**第一批已在本地实施：公共契约与兼容检查执行所有权。** 范围只含现有 `oclive_kernel_contracts` / `oclive_kernel_types` 的完整检查，不称两个crate都是小Kernel。已有 `check:kernel-base` 四步保持为有限开发反馈；新入口 **`npm run check:kernel-contracts`** 在源码根目录依次运行：

```sh
cargo clippy --locked -p oclive_kernel_contracts -p oclive_kernel_types --all-targets --all-features --no-deps -- -D warnings
cargo test --locked -p oclive_kernel_contracts -p oclive_kernel_types
```

新入口与窄入口复用同一个native命令辅助函数；工具缺失、失败或超时均非零并停止后续步骤，每步上限20分钟。它需要Node/Cargo/Clippy，不安装依赖，不准备前端或Host，不自动检查外部作者模块。未加 `--lib` / `--test` 过滤：保留两lib单测、`base_only_fixture`、`minimal_role_contract`、`runtime_event_stream_contract`、`runtime_event_stream_onebot_review`及两lib适用doctest。两个manifest当前没有自有feature；默认测试条件保持，Clippy保留all-targets/all-features。rich/Stream兼容测试未删减，也未新增公共接口义务。

| 调整点 | 原所有者 | 本地首片所有者 / 保持覆盖的方法 |
|---|---|---|
| 两crate实际被执行的测试及Clippy目标 | `rust` 混合job，两平台 | `kernel-contracts`独立组，两平台、完整两crate目标；Clippy `--no-deps` 防止经其它根包又lint同一依赖。其余workspace的根包并集与原非CLI根包集合相同。 |
| 共享消费者、参考Host、server、桌面等剩余回归 | `rust` 及各现有组合job | 保持原所有者及适用资源；剩余workspace仅排除两迁移根包，CLI原独立所有权保持。workspace fmt仍是治理，不归某个Host。 |
| 门禁与受信选路 | 现有catalog/本地入口/workflow/repository contracts | 原会选中 `rust-workspace` 的5个profile同步加入新组；影子21场景只更新相应validator/job坐标，影响图不变。`ci-gate`聚合新组，选中/全量时不得跳过或失败；既有docs选择路径仍可合法跳过。 |
| 旧受信基线尚不认识新组 | 原trusted-base机制 | 在受信目录核新validator及job映射；缺失/错误/解析失败走既有 `trusted_policy_bootstrap` 全量回退，不使用PR自己的目录批准少跑。正式main的required名称和ready/Push策略保持。 |

第一批的收益是公共检查不必先准备前端/Tauri资源，并有可解释的执行所有者；**不是节省时间比例承诺**。检查集合已对齐，不长期保留重复两crate测试；各job仍可能编译共有依赖。到本地首片即可停止，不以全仓完全分离、全部混合job拆开或job数减少为完成条件。共享Memory调用案例继续作为小Kernel外的接入证据；细粒度性能、real模型/TTS、实机及全部崩溃窗口另有目标才验。

实施与适用本地验证见[所有权首片报告](../../handoff/debt-marathon/waves/WAVE-20261011-CI-CONTRACT-OWNER.md)。未改影响图/ready晋升类别/公共API/运行语义，不关闭K-CI-IMPACT-01；未合入推送或跑目标SHA正式CI，Linux及新远端配置仍待该出口。原责任调查的证据边界保留于[前轮报告](../../handoff/debt-marathon/waves/WAVE-20261011-CI-RESPONSIBILITY-MAP.md)。

## 3. 规划输出与可解释性

规划器输出稳定排序的 `plan.json`，至少包含：

- base/head SHA、策略和 changed files；
- 直接模块、受影响模块及逐项原因；
- 选中/跳过的验证器及原因；
- 是否进入全量回退及触发原因；
- 输入契约摘要，便于复现同一计划。

`explain` 只把同一份 JSON 渲染成人类摘要，不重新计算。初期通过 GitHub Job Summary 和 artifact 暴露证据，不自动写 PR 评论，避免噪音与额外写权限。

## 4. 分阶段启用

| 阶段 | 行为 | 权威性 |
|------|------|--------|
| **Stage 1 · Shadow** | 计算建议范围、输出报告；主 CI 硬门禁全部照常运行 | 只观察，禁止据此跳 job |
| **Stage 2 · Compare** | 对比“规划器本会跳过的验证”和全量结果，积累漏选/过选数据；模拟语料只做路由回归 | 实际远端结果仍权威 |
| **Stage 3 · PR selective** | `domain-aware-pr-v2`：安全草稿 PR 按计划执行；ready 阶段仅纯文档 Canary 继续选择性执行 | 草稿门禁不能满足分支保护；非文档 ready、高风险、未知路径、warning、规划器异常与 Push 全量 |
| **Stage 4 · Merge/Nightly split** | 合并门禁保留跨模块/高风险全量；长时 soak、GPU、性能移至 Nightly/Release | Nightly 不替代合并前硬门禁 |
| **Stage 5 · Ecosystem** | 脚手架生成/校验模块描述，外部模块复用规划与契约检查 | 外部流水线自行负责 |

Stage 1/2 的成功条件不是“CI 变快”，而是规划结果确定、可解释、fail-safe，且能用全量 CI 的事实验证没有漏选。Stage 3 只按已验证类别逐类开放；纯文档 Canary 不构成前端、Rust、插件或跨宿主类别已经可跳过的证据。

### 4.1 Shadow 证据分级

- **规划模拟**：`data/ci/shadow-scenarios.v1.json` 固定代表性 changed paths、期望模块闭包、validator/job 坐标和 fail-safe 原因；`npm run ci:shadow-samples` 生成 JSON + Markdown 到 `target/oclive-ci/shadow-samples/`，但不执行任何 validator。
- **真实 Compare**：把某次实际 diff 的 `plan.json`、`execution.json` 与同一 workflow SHA 的全部远端 job 终态绑定，才可记录漏选/过选；全量 run 自动上传 `oclive-ci-compare-*` JSON/Markdown（90 天），为“本会跳过但实际失败”的 job 标记 false-negative candidate，仍须维护者裁决。快照缺失、job 未终态、SHA 不一致或结果不完整时只能记为 observational；失败后修复重跑不能被合并成“从未失败”。
- **当前模拟集合**：沿用2026-08-21的 **21** 场景（**18** 个靶向、**3** 个 fail-safe），2026-10-11首片仅更新完整公共检查的执行坐标；除既有 docs、shared、角色包、目录插件、内核、脚手架、examples、Nightly、控制面、锁文件与未知路径外，覆盖中文角色路径的 NUL 文件输入、docs+scaffold、docs+examples、CLI、Desktop、Theater、跨域 reply-mode、docs/scaffold merge 与 Dependabot 节奏。shared / 角色包 / 目录插件仍因当前前端影响环选中9个validator（原8个加新公共所有者，含Rust），仍保留过选候选，不能仅凭模拟擅自删边。
- **纯文档真实对照（2026-08-15）**：PR #159 只改 `handoff/TECHNICAL_DEBT_INVENTORY.md`；计划为 targeted docs、2 个 validator，旧主 CI [`31828405121`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/31828405121) 全量 **16/16** 成功、墙钟约 **48 分钟**，未观察到漏选。该证据只授权纯文档 Canary。

模拟通过只能证明规划器按**当前规则**稳定工作；它不能证明规则本身没有遗漏，也不能替代真实进程、硬件或远端平台证据。

### 4.2 Stage 3 执行安全下限

- `plan.shadow=true`、任何 warning、`fallback.full=true`、非 PR、空直接/受影响范围或空 job 集合，一律 `run_full=true`；
- 草稿 PR 只有在上述安全条件全部满足时才可选择性执行，并使用 `ci-draft-gate`；该名称不得配置成 main 的 required context；
- `ready_for_review` 必须重新触发当前提交：直接模块只有 `oclive.docs` 时可沿用纯文档 Canary，其他 ready PR 一律 `run_full=true` 并生成正式 `ci-gate`；
- PR 的规划器、中央影响图、验证目录、执行策略和 `ci-gate` 校验逻辑均从 comparison base 的受信提交运行；当前 PR 代码只提供 changed paths 与被验证内容，不能改写自己的选择结果；
- 每个主 CI job 必须依赖 `ci-impact-plan`；规划器失败时通过 `always()` 运行全量，同时让最终门禁失败，禁止静默降级为绿；
- 对应 gate 始终运行，并验证全量模式全部成功，或选择模式中 selected job 成功且 unselected job 确为 skipped；
- GitHub 分支保护只绑定稳定 `ci-gate`，不把可能按计划 skipped 的单个 job 设为 required context；
- `scripts/ci-execution-policy.mjs` 与 `scripts/collect-ci-compare-evidence.mjs` 属 CI 控制面高风险路径，修改它们会触发规划器 full fallback；
- 失败自动重跑已取消；`.github/workflows/ci-rerun-flake.yml` 仅接受人工指定的失败 CI run，且只在失败集合为 Rust 矩阵加聚合 `ci-gate`、未重试过时定向重跑一次。

## 5. 脚手架的辅助边界

Scaffold Package 的发现、来源锁定、命令命名空间与兼容规则由 [`RFC_SCAFFOLD_PACKAGE_V1.md`](../rfc/RFC_SCAFFOLD_PACKAGE_V1.md) 单独维护；本文只规定它与 CI 的交界，避免脚手架契约反向扩张 CI 控制面。

脚手架以后可以：

- 询问模块类型、能力、依赖和资源需求；
- 生成符合 schema 的 `oclive.module.json`；
- 只从验证目录列出可选 profile；
- 在本地调用同一个规划器执行 `validate` / `plan` / `explain`；
- 提示未映射路径、未知坐标、兼容范围和建议补测。

脚手架不可以：

- 生成或修改主仓工作流编排权；
- 把任意 shell 塞进模块描述；
- 自动批准 secret、自托管 GPU runner 或高危权限；
- 另造一套影响算法、验证目录或资源 schema；
- 将“生成成功”当作模块已通过 CI。

因此二者的关系是：**脚手架生产和预检 CI 能理解的知识，CI 在受信环境里消费同一知识并生成证据。** 描述契约稳定前不改脚手架，避免模板先于解析器成为第二套事实源。

## 6. 保留的性价比原则

- 本地开发跑受影响窄测；需要远端协作时保持 PR 为 draft 以获得领域选择反馈；逻辑里程碑跑完适用的完整本地门禁、冻结 HEAD 后再转为 ready，让当前提交只触发一次正式门禁。
- Dependabot 的常规 version updates 按 npm / Cargo 生态分别合并兼容范围更新、错峰并设置冷却；主版本迁移由维护者单独规划，不自动制造多条 ready PR。Security updates 继续独立处理，不能为了节省 CI 被常规版本节奏吞并。
- Linux + Windows 继续覆盖当前主力平台；正式发布 Mac 包前不因本设计自动扩张 macOS 矩阵。
- 更重的真窗口 E2E、GPU soak、内存泄漏和性能基准归 Nightly/Release 或显式手动任务，不进入每次小提交。
- 包契约或校验 crate 改动仍须联动 `oclive-pack-editor` 的 `npm run contract:json-keys` 与 `HOST_RUNTIME_VERSION`；影响规划不能替代跨仓契约验证。

**相关**：产品体验向 backlog 见 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](./BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)。
