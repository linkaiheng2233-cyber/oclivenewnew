# OCLive 领域感知 CI · 分阶段实施基线

> **状态（2026-08-18）**：Stage 3.1 以 `domain-aware-pr-v2` 把领域规划真正用于开发期 PR：无 warning、无 full fallback、非 shadow 且范围/job 非空的**草稿 PR**按计划执行，并只发布不能满足分支保护的 `ci-draft-gate`；转为 ready 时同一提交由 `ready_for_review` 重新触发，除已有真实 Compare 支撑的纯文档 Canary 外一律全量并发布正式 `ci-gate`。Push、高风险/未知路径和规划器异常仍全量。每次全量 run 会自动生成同一 workflow SHA 的 Compare JSON/Markdown，保留 90 天；选择性 run 只记执行证据，永不冒充漏选对照。影子语料已扩为 **20** 场景（**17 targeted / 3 fail-safe**），并覆盖中文/NUL 路径、混合模块和 merge 策略。五个 `nightly` 责任组继续留在独立定时/手动工作流。本文是 CI 影响规划的设计 SSOT；模块边界只在 [`MODULE_MAP_AND_HANDOFF.md` §12.7](../../handoff/MODULE_MAP_AND_HANDOFF.md#127-ci-影响元数据与脚手架边界) 登记，执行证据见 [`TECHNICAL_DEBT_INVENTORY.md` K-CI-IMPACT-01](../../handoff/TECHNICAL_DEBT_INVENTORY.md)，核实口径以 [`AI_VERIFICATION_PROTOCOL.md`](../../handoff/AI_VERIFICATION_PROTOCOL.md) 为准。

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

### 2.5 执行通道约定

- `.github/workflows/ci.yml`：`ci-impact-plan` 产出执行范围；安全草稿 PR 与 ready 的纯文档 Canary 可选择性执行，其他事件 fail-safe 全量；草稿发布 `ci-draft-gate`，ready/Push 发布受保护的 `ci-gate`，两者都核对 selected/success 与 unselected/skipped；
- `.github/workflows/nightly-advisory.yml`：目录中 `tier=nightly` 的 Loom、fuzz、原生窗口、视觉冒烟和无阈值性能证据；支持每日全跑与按 validator 手动复现；
- Nightly 失败不会阻塞 main，但在 Nightly 内不得 `continue-on-error`；失败日志或 artifact 是待处理证据，不能粉饰为绿；
- 同一 `workflow_jobs` 坐标按 validator 的 `tier` 落入对应受信通道，仓库契约测试防止 job 漂回错误工作流。

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
- **当前模拟基线（2026-08-18）**：**20** 场景全部契约一致，其中 **17** 个靶向、**3** 个 fail-safe；除既有 docs、shared、角色包、目录插件、内核、脚手架、examples、Nightly、控制面、锁文件与未知路径外，新增中文角色路径的 NUL 文件输入、docs+scaffold、docs+examples、CLI、Desktop、Theater、跨域 reply-mode 及 docs/scaffold merge。shared / 角色包 / 目录插件仍因当前前端影响环选中 8 个 validator（含 Rust），作为过选候选保留，不能仅凭模拟擅自删边。
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
- Linux + Windows 继续覆盖当前主力平台；正式发布 Mac 包前不因本设计自动扩张 macOS 矩阵。
- 更重的真窗口 E2E、GPU soak、内存泄漏和性能基准归 Nightly/Release 或显式手动任务，不进入每次小提交。
- 包契约或校验 crate 改动仍须联动 `oclive-pack-editor` 的 `npm run contract:json-keys` 与 `HOST_RUNTIME_VERSION`；影响规划不能替代跨仓契约验证。

**相关**：产品体验向 backlog 见 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](./BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)。
