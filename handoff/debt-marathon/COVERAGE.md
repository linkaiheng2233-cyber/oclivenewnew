# 计划书覆盖审计（对照 TECHNICAL_DEBT）

**SSOT 范围**：台账与计划书的覆盖关系及缺口；不决定债务状态或自动开工权限。
**更新日期：** 2026-09-29
**当前结论：** 不能再声称所有活跃债都有 `long-plans/`。后续新增事项未全部进入旧马拉松队列；专题 RFC/本机计划也不等于 Ready 机器契约。本轮变动见 [DCL-20260928-01](DEBT_CHANGELOG.md#dcl-20260928-01--初始化审查与调度对账)。`V-MODULE-QUALITY-01` 已有的 Done 证据不变。
**详略：** `auto` 书 = Minimal 可执行分阶 + `oclive-marathon-contract`；`skip`/`human` = stub + 触发条件。`npm run check:debt-marathon` 强制校验所有 auto 书的文件范围、验收理由、产出、回退与父债处置。**深度细则以 [`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md) + 七阶段 Skill 为准**（不在每本 stub 重复）。

**开工依赖**：见 [DEBT_CHANGELOG · 依赖登记](DEBT_CHANGELOG.md#依赖登记与判读)；区分硬前置、决策/外部/证据门、共因与建议顺序。下表“缺计划”不表示缺少全部已有契约，也不意味着上游 Full 必须先 Done；实施具体切片时核对实际使用的能力、owner 与解除条件。

## 2026-09-28 审查与接手缺口

受检起点 `6ed1dda1`；审查台账各现行区段、历史引用、QUEUE 和计划文件。以下是**接手/覆盖问题**，不是重新实测全部债务：

| 问题 | 核对结果 | 接手处置 |
|------|----------|----------|
| 标题与日期误导 | §1 标为“OPEN”但也保留 Done；首部最后更新停在 8 月而已有 9 月增补；同一 ID 在供应链、观测及历史重现 | 台账日期和范围头已纠正；按条目状态与证据范围读，不统计章节行数作为“未偿还债总数”，不机械删除历史引用 |
| 五个旧 PR 等待态 | seq 40/50/60/120/130 的计划/Wave 均在已合 #126 中，当前没有对应 open PR；Full/人工/跨仓条件仍未满足 | QUEUE 与五本契约同步 `blocked:needs-reconcile`，保留 currentStage 与父债状态；不直接转 runnable/Done |
| 模板 Ready 与实际能力 | K-PLUGIN-SEC-01 虽有 ready Stage，台账仍缺 Windows 原生隔离与可信签名证据；human/skip 亦不代表产品债消失 | 结构可运行不是能力或任务授权；实际实施前核本轮环境、未决语义与目标范围 |
| 旧 Host 恢复描述 | 台账 D-HOST-RECOVERY-01 仍描述旧 `/chat` 整轮重发；当前 `chatStoreSend` 在失败分支使用同对象 `recoverMessage(request)`，已有桌面限定验收 | 关联 [Host 检查单](../CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md) 更正当前行为，保留未覆盖领域/崩溃风险与延期条件；不升整债 Done |
| 状态词未完全规范 | K-EMO-02 使用“已验收”；部分条目同时含 Minimal Done、Full Partial 或 Implemented/Observe | 本轮保留历史裁定，不靠字符串命中统一转 Done；下一次触及条目时核 owner 和缺失证据 |
| 证据携带边界 | 多项工具链、模型/语音与接入证据位于 `E:\Env`、`.cursor/plans` 或隔离运行树，不随 Git 转移 | 接手先取原始证据；取得不了就登记 needs-evidence-access，不把摘要视为重跑结果 |

下列现行事项没有同名 long-plan（仅文件覆盖核对；已有专题或本机计划另查 owner），尚不能由旧自动队列接手：

| 分组 | ID | 开工前补什么 |
|------|----|--------------|
| 工具链/构建 | K-TOOLS-01、K-VERIFY-01、K-BUILD-06、K-BUILD-07 | 门禁真实调用链、受控缺依赖负例、内存/缓存预算和安全维护范围；未取得等价证据不移除 `-j 1` |
| 情绪/编码/语音 | K-EMO-01/03/05/06/07、K-ENCODING-01、K-VOICE-09 | 分清算法、标签/来源、持久化与真实硬件验收；不将词表输出或短测当作 Full 质量 |
| Kernel/Host/可移植性 | K-CORE-BOUNDARY-01、D-CLI-BLUEPRINT-05、K-PROACTIVE-01、K-EVENT-STREAM-01、V-PORTABLE-01、V-EMBED-01 | 从现行 SSOT 选定一个边界；不自动启动物理拆分、Production Stream 或发行版全合规 |
| 待决语义与实验 | K-UID-DEFAULT-02、K-DUAL-ROLLBACK-02、K-AGENT-MERGE-01 | 产品/架构取舍及确切冻结/解冻条件；不根据“代码已存在”默选方案 |
| 资源/CI/开发工具 | K-RESOURCE-COORD-01、D-SCAFFOLD-RESOURCE-01、K-CI-IMPACT-01、D-CI-AI-REVIEW-03、D-SCAFFOLD-EVOLUTION-04 | 当前专题计划、测量/数据收集范围与执行权限；通用 CI 绿不替代资源 soak/选择性门禁证明 |
| 台账/资产 | D-DEBT-LEDGER-01、D-ASSET-FOOTPRINT-01 | 已有 [重复 ID 引用化、七月 Verification 与八月工程快照迁移、持续结构门禁](waves/WAVE-20260929-DEBT-REFERENCES.md)；§5 等其它长快照、全表状态词、自由文本语义冲突及资产取舍仍需具体范围；结构治理不代替产品验收 |
| 连续性/编写器/LoRA | K-CONTINUITY-01、PE-CONTINUITY-01、V-LORA-FORGE-02、V-LORA-PACK-03、V-LORA-PEFT-04 | 人工观感、跨仓范围或适配契约；不能把运行时局部完成扩成整个创作产品 Done |
| Deferred Host 风险 | D-HOST-RECOVERY-01、D-HOST-ERROR-CONTEXT-01 | 新范围、当前源码复核和未测窗口；历史静态推断不能直接重演为当前故障 |

缺计划不意味着应给每债批量造 stub；普通单项任务可先用现行专题计划，只有要进入自动马拉松才补 Ready 契约、QUEUE 和当轮授权。下方是 **2026-08-14 历史覆盖矩阵与模板说明**，不作为当前全量覆盖声明。

## 历史覆盖矩阵（2026-08-14）

| 台账状态类 | 覆盖 | 注 |
|------------|------|-----|
| OPEN / Partial 主仓可施工 | ✓ auto 书 | K-PLUGIN-SEC-01 · RESILIENCE · CROSS/DIST/MARKET · VOICE-07 · SUPPLY-05-Full |
| OPEN 跨仓/人工 | ✓ human stub | VSCODE · PE-* · P0-STRANGER-EXT |
| Partial | ✓ | K-PERF-10 skip |
| Observe | ✓ skip stub | SLOT/TRAIT/PORT/POLICY/ORPHAN · F4 · SUPPLY-04/08 · VOICE-05 等 |
| Deferred / 冻结 | ✓ skip stub | VOICE-01/08 · SUPPLY-06/07 · MEGA · MODE3 · §4 项 |
| 已 Done | 不建施工书 | PLATFORM/LLM/VOICE-04 Minimal 等；仅 FOLLOWUP 跟随项 |

## 历史 stub 补齐（2026-08-14 · §4）

| ID | 文件 |
|----|------|
| K-PERF-15 | long-plans/K-PERF-15.md |
| V-FUSED-01 | long-plans/V-FUSED-01.md |
| V-LORA-WORKSHOP-01 | long-plans/V-LORA-WORKSHOP-01.md |
| D-OPUS-05-P2 | long-plans/D-OPUS-05-P2.md |
| K-UID-POST-01 | long-plans/K-UID-POST-01.md |
| ROADMAP-MODAL-EDGE | long-plans/ROADMAP-MODAL-EDGE.md（§3.5–3.7 + §5.3 UGC 合并 stub） |
| DUAL-CORE-FREEZE | long-plans/DUAL-CORE-FREEZE.md |
| D-READ-03 | long-plans/D-READ-03.md |

## 计划详略与准入

| 类型 | 详细度 | 是否够隔夜 |
|------|--------|------------|
| auto Minimal | Stage 表 + 机器契约 + 非目标 + 停条件 | 具备模板；须完成结构、语义、环境与权限对账才可派工 |
| auto Full 战役 | 机器契约 + 有限 Stage + 停条件，禁止假 Done | 只执行获准 Stage；前置条件不足即 blocked |
| skip/human stub | 薄 · 故意 | 够「跳过不犯错」 |

若要加强某本 auto 的行级文件清单：优先加厚该 ID 的 long-plan，而不是膨胀 stub。
