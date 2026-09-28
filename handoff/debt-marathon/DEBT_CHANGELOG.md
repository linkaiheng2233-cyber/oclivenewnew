# 技术债变动记录与 AI 接手协议

**SSOT 范围**：技术债变动的事件、证据关联与安全续跑坐标；不维护第二份当前债务状态表。
**最后更新**：2026-09-28。

当前状态唯一以 [TECHNICAL_DEBT_INVENTORY](../TECHNICAL_DEBT_INVENTORY.md) 为准；执行准入以 [QUEUE](MARATHON_QUEUE.md)、对应 long-plan 和 [GATES](AI_AND_PIPELINE_GATES.md) 为准。通用过程以[仓库流水线](../workflows/dev-pipeline/SKILL.md)为准。本记录中的旧事件是当时快照，后续事件不能改写原始证据。

## 接手顺序

1. 核 `git status --short --branch`、HEAD 与任务授权；用 `rg -n '<DEBT_ID>' handoff/TECHNICAL_DEBT_INVENTORY.md` 找到当前条目。相同 ID 可能有历史、观测或父/子项引用，不能按第一处 `Done` 判整债完成。
2. 在本文件检索该 ID，读最新相关事件及其证据；再读目标计划的当前 Stage。核对 **debt ID + Minimal/Full + 能力范围 + base/head**，不只按标题或时间接手。
3. 对照台账、QUEUE、计划契约、实际 diff 与远端 PR/CI。冲突先记录 `needs-reconcile`；检查器绿只证明其声明的结构，不替代语义对账。
4. 取原始证据；本机忽略目录或用户盘内文件无法取得时标 `needs-evidence-access`，不得把摘要补成原始日志。已消耗业务身份不复用；未知外部进程或 dirty 不擅自清理。
5. 继续一个已授权范围；恢复前确认最后命令、下一步和 `retry_safe`。本轮只有文档治理授权，没有自动开启任何业务债。

## 更新规则

这里的“实时”指**在事实发生的同一个工作检查点写入**，不是后台定时任务。以下事件必须记账：新增/合并/拆分债、状态或优先级变更、Minimal/Full 边界变更、阻断/解冻、新实现或验证证据、复核更正、交接/暂停/收口。

- 先取得证据再改 owner 文档；控制者同一检查点同步受影响的台账、计划、QUEUE 和本事件。无事实变化的轮询不追加事件；Wave 已有细节时只链 Wave，不再抄命令长表。
- 区分四层：**债务状态、计划/Stage 进度、验证结论、PR/CI 状态**。PR merged 不等于父债 Done，计划 closed 不等于 Full 完成，调度 blocked 不表示产品回退。
- 只记录本轮实际核验；旧数据注明日期/SHA，遗漏历史保留 unknown，不回填伪造历史。文案更正或失败归因追加事件并链接旧件，不改冻结日志/DB/二进制。
- 同一债优先复用 ID；新反例先查归属/重叠再决定拆债。Observe/Deferred 没有排期也保留触发条件，不因巡检自动解冻或提升优先级。
- 债 Done 必须满足该范围的全部门槛和 [核实协议](../AI_VERIFICATION_PROTOCOL.md)，含项目要求的目标 SHA 远端 CI及人工/实机证据。缺任一项只记执行进展，保留父债 OPEN/Partial。
- 活跃修改由当前控制者单写；有委派时执行方返回证据和 diff，不能自行变更全局状态。单 Agent 可以完成全部职责，不强制启用子 Agent。
- 事件用稳定编号 `DCL-YYYYMMDD-NN`，按发生顺序追加；更正引用旧编号。不要为补 CI run ID 单独造提交，先记交付报告，在下一次实质变更时入账。
- 本文件不无限堆细节：较长原始记录放 `waves/` 或相应证据目录，事件保留一段摘要和链接；需要分年时保留索引，不删除历史。

## 事件格式

下面只列字段，不是已发生事件：

```text
事件 ID / 日期 / 记录者；类型（新增/进展/复核/阻断/状态迁移/更正/交接）
debt ID 与范围（Minimal/Full、实际能力、owner）
before → after（分别列债务状态与计划/调度/验证，未变写未变）
依据（base/head SHA、实际 diff、原始日志/测试/PR/CI；本机证据可携带性）
关联更新（台账 / 计划 / QUEUE / Wave 的路径；无需改的说明）
未测与残留；最后动作；下一条精确动作；retry_safe 及原因
```

## 变动事件

### DCL-20260928-01 · 初始化审查与调度对账

- **记录者 / 类型**：主控 Codex；复核＋阻断。本轮起点 `6ed1dda1f0ab5317450ce4609a6c29c47d423e50`，开场工作树干净。该 SHA [主 CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36384352172) 17/17 success 是已有工程基线，不是本轮文档提交或全部债务重新验收。
- **范围**：读取台账 §1、前瞻风险、§1.5、冻结/观测/Deferred 与历史；核对计划覆盖、五条 `pr-open`、已验 Host 恢复与当前源码。本轮未运行业务场景、真实模型/语音或硬件矩阵；没有实施任何业务债修复。
- **变动**：五条 QUEUE seq 40/50/60/120/130 的 `pr-open → blocked:needs-reconcile`；对应机器契约 `ready → blocked` 并写明确前提，`currentStage` 原样保留。K-RESILIENCE-01 的本册仍为 Minimal；K-SUPPLY-05-Full、K-CROSS-01、K-DIST-01、V-MARKET-01 的父债均保持原 Full/Partial 边界，未转 Done。计划状态头与主台账对齐；K-SUPPLY-05-Full 撤回“紧急 skip 也可 Full”这一与其既有零 skip 合同矛盾的句子，保留例外只能记 Partial。
- **依据**：[PR #126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 已于 2026-07-16 合并（`23e4e1843ddc2c3ddf3c4cfd727131950fd50c66`），其文件列表包含这五本计划及 Wave；当前 open PR 列表没有对应未合项。已合历史切片不能替代 Full、跨平台实机、签名权限或跨仓工作授权。不能继续沿用“等待同一 open PR”作为下一步。
- **关联**：[覆盖审查](COVERAGE.md) 已撤回全量覆盖声明并列出缺口；[第二轮计划](ROUND-02-PLAN.md) 的历史启动检查继续保留，本事件更新其流水线来源和对账进度。台账中的 D-HOST-RECOVERY-01 只更正当前桌面恢复的描述，未关闭未覆盖的领域/崩溃风险；D-DEBT-LEDGER-01 保持 OPEN，记录本次交接机制切片而不宣称全表规范化完成。
- **下一步**：先选择一项已授权业务债，按其 owner 和完成条件形成现行计划；对五个 blocked 项逐项确认剩余范围和证据后再解除前提。可运行 `npm run check:debt-marathon` 检查结构；这不是开工或 Done 许可。**retry_safe**：只读对账和结构检查可重跑；旧 live 身份不得重用。

### DCL-20260928-02 · 仓库流程成为正式来源

- **记录者 / 类型**：主控 Codex；规则与接手前提变更，关联 D-DEBT-LEDGER-01，业务债状态不变。
- **before → after**：本机缺少 `~/.cursor/skills/dev-pipeline/SKILL.md`，项目入口引用机器私有来源 → [仓库通用 Skill](../workflows/dev-pipeline/SKILL.md) 成为正式来源，本机只安装同字节副本。依据为维护者本轮确认“以仓库的为准，然后放到本机”。
- **关联更新**：AGENTS、AI 阅读索引、handoff 入口、项目 Skill、GATES 与马拉松入口统一引用仓库；本文件承担变动事件与续跑规则。项目 G1–G17 和架构契约没有复制或降级。
- **边界与续跑**：本次安装仅限 `~/.cursor/skills/dev-pipeline/`，不改变其他技能、权限或自动续轮配置。后续改仓库通用 Skill 时，安装副本要重新同步并核 bytes/SHA256；不可只改本机后声称团队规则已更新。本轮命令结果在交付报告登记，文件包含在里程碑提交中，无技术债 Done 迁移。
- **验证出口（Locally verified）**：文档链接默认入口与本轮显式文件、登记、旧路径和 diff 检查；债契约 12 auto plans 结构通过，五个 blocked 项逐项 `--require-ready` 均以 exit 1 拒绝；中文 UTF-8/无 BOM 核验，以及三文件安装副本 bytes/SHA256 相等。官方 `quick_validate.py` 在两个 Python 运行时均因缺 PyYAML 无法启动；改用仓库已安装 `yaml` 解析器核 frontmatter、字段/命名与未完成占位，四份相关 Skill 通过，**不声称官方脚本 exit 0**。未为此安装新依赖。
- **最后动作 / 下一步**：完成上述治理切片与本机镜像；交付给后续 Agent 时先运行 `git log -1 --format=fuller`、`git status --short --branch` 和 `npm run check:debt-marathon`，再按 DCL-20260928-01 确认一个具体业务范围。只读检查可重跑，本次记录不授权业务场景或远端写入。
