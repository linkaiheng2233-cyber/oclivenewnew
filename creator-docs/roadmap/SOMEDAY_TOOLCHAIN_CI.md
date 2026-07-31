# OCLive 领域感知 CI · 分阶段实施基线

> **状态（2026-08-01）**：Stage 1 已实现并进入 Shadow 证据积累期；规划结果仍不控制或跳过现有 job。本文是 CI 影响规划的设计 SSOT；模块之间的职责边界只在 [`MODULE_MAP_AND_HANDOFF.md` §12.7](../../handoff/MODULE_MAP_AND_HANDOFF.md#127-ci-影响元数据与脚手架边界) 登记，执行证据仍以工作流和 [`AI_VERIFICATION_PROTOCOL.md`](../../handoff/AI_VERIFICATION_PROTOCOL.md) 为准。

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

模块只能引用验证器坐标。`command_id` 是有限、可终止的本地复现入口，`workflow_jobs` 才是当前远端 CI 的实际编排映射；Stage 1 规划器只报告两者，不执行命令，也不调度 job。命令、工作目录、secret、runner、缓存、并发和超时始终归主仓工作流/验证目录所有。第三方提交到主仓时，其自测只能通过已审核坐标受限执行；插件自带 `.github/workflows/*` 不参与主仓编排。Fork/二次发行可自行维护 CI，主仓不对其流水线负责。

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
| **Stage 1 · Shadow** | 计算建议范围、输出报告；原 CI 全部照常运行 | 只观察，禁止据此跳 job |
| **Stage 2 · Compare** | 对比“规划器本会跳过的验证”和全量结果，积累漏选/过选数据 | 全量仍权威 |
| **Stage 3 · PR selective** | 只对低风险且有足够证据的 PR 启用选择性验证 | 高风险规则与未知路径仍全量 |
| **Stage 4 · Merge/Nightly split** | 合并门禁保留跨模块/高风险全量；长时 soak、GPU、性能移至 Nightly/Release | Nightly 不替代合并前硬门禁 |
| **Stage 5 · Ecosystem** | 脚手架生成/校验模块描述，外部模块复用规划与契约检查 | 外部流水线自行负责 |

Stage 1 的成功条件不是“CI 变快”，而是规划结果确定、可解释、fail-safe，且能用全量 CI 的事实验证没有漏选。只有积累过多个真实改动类别后，才能讨论 Stage 3 的跳过策略。

## 5. 脚手架的辅助边界

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

- 本地开发跑受影响窄测；逻辑里程碑再跑一次适用的完整本地门禁并推送一次远端 CI。
- Linux + Windows 继续覆盖当前主力平台；正式发布 Mac 包前不因本设计自动扩张 macOS 矩阵。
- 更重的真窗口 E2E、GPU soak、内存泄漏和性能基准归 Nightly/Release 或显式手动任务，不进入每次小提交。
- 包契约或校验 crate 改动仍须联动 `oclive-pack-editor` 的 `npm run contract:json-keys` 与 `HOST_RUNTIME_VERSION`；影响规划不能替代跨仓契约验证。

**相关**：产品体验向 backlog 见 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](./BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)。
