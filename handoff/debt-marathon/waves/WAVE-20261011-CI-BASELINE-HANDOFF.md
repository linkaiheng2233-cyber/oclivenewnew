# CI 分层 · 分类开发基线与维护者对接 · 2026-10-11

**性质**：分类提交、既有证据核对与有限对接快照；不是第二份CI规则。执行归属与选择范围由[CI设计](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-layering-development-handoff)拥有，架构分责仍由[MODULE_MAP](../../MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)拥有。
**状态**：Locally verified；基线核对与本轮适用文档门禁通过，按文档切片保存本地提交，不启动新的远端CI。
**身份**：本轮base `2b8d03ae5a2305dc4de081f705c1e0c67ba6fd06`，分支 `codex/ci-layering-preparation-20261011-r0`；controller实施及语义自查，`independent=false`。

## 分类保留的成果

| 分类 / 提交 | 具体内容 | 证据归属与开发状态 |
|---|---|---|
| 已集成脚手架里程碑 `24f62303e9bdea37061fe44acc7a0830dc519f98` | 基础工程、Memory第一条开发路径、常用命令、问答/作者模板及最终文档收尾合批 | main与实际origin/main相同且工作树干净；[集成快照](WAVE-20261011-KERNEL-BASE-INTEGRATION.md)记本地验收边界，正式CI见下文 |
| CI分责设计 `a74e249c8c444525073eecf3705e587f0df84089` | 四个Markdown路径，按8组现有入口有限归属，提出公共基础契约出口 | 文档准备提交；不改变工作流、选择权限或公共契约 |
| 本地入口实施 `2b8d03ae5a2305dc4de081f705c1e0c67ba6fd06` | 十路径关联闭环：固定四步 `check:kernel-base`、同链失败停止回归与中英使用说明 | [本地报告](WAVE-20261011-KERNEL-BASE-LOCAL-CHECK.md)拥有原始验收及失败更正；Locally verified，尚未合入/推送main |
| 本轮对接文档 | 五个Markdown路径，说明现行触发策略与下一片迁移顺序 | 只做适用文档验证，不重写上述提交，不冒充新SHA正式验收 |

分类依据是可审查的职责与变更，不按文件类型机械拆开一个关联闭环。此前入口实施已保存为完整提交，本轮没有遗留业务差量需要再切分。

## 实际主基线与证据边界

本轮读取实际远端 `refs/heads/main` 与正式[CI 38111785703](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38111785703)：`headSha=24f62303e9bdea37061fe44acc7a0830dc519f98`，workflow为completed/success，17个job全部success，含 `ci-gate`。这是该主基线的正式证据，不能给本地 `2b8d03ae` 或本轮新文档SHA盖章。

单次run的job时间观察：规划开始04:30:18 UTC，最后gate结束05:04:09 UTC，跨度33分51秒；Rust Windows为31分06秒，Rust Linux为28分51秒。它描述这一次run的关键路径，不是缓存/平台性能矩阵，不能据此预报分层后的节省比例。未新发run、rerun或workflow dispatch。

原本地入口报告记录真实四步exit0：Base fixture 16 passed、最小角色2 passed、两crate Clippy通过、显式doctest 11+17 passed；入口4项失败关闭回归及复用helper 17项回归通过。本轮沿用该提交的原证据，没有重跑或改写失败日志。公共基线入口不检查作者模块实际检索质量，也不等于全部Kernel硬边界、共享消费者或Host/发行版验收。

## 本轮有限调查与下一片

只核现有主/Nightly workflow、catalog和模块元数据、执行策略及仓库合同的直接入口，维持[本计划](../ROUND-02-PLAN.md#ci-分层--分类基线与维护者对接2026-10-11)的8组/D1止点。例如 `rust` 先准备前端与Tauri资源再测非CLI workspace，因此本地公共窄测有直接用途；这不能推导所有下游回归都可省掉。源文件核对确认13个验证job定义与两组双平台矩阵，当前全量展开17个实际job。

下一片优先迁移公共基础检查的远端执行归属，保留Host/组合回归及现行Push/ready策略；迁移前明确测试集合、原所有者、剩余测试与受信入口同步，不因窄入口可用直接排除整个crate。选择范围扩大另行按真实Compare评估，具体规则链接CI设计，不复制第二张权力表。本轮达到足以对接的结论即停止，不进入逐测试全仓调查或缓存研究。

## 本轮验证与保全

精确五路径：CI设计中英、本计划、DEBT_CHANGELOG及本报告；四份磁盘原件先保全，计划/事件历史只追加。原件、摘要和本轮stdout/stderr/native exit分件位于本机ignored `.cursor/plans/ci-baseline-handoff-20261011-r0/`，不是跨机器阅读本报告的必备材料。

适用默认及五路径链接、docs-only旧路径、镜像、登记、债结构、编码共七组命令，均native exit0。另核五路径严格UTF-8无BOM、两份历史正文原字节前缀、九个控制面/本地入口文件摘要与精确diff，均通过；这些结构结果不代替语义自查或债状态判断。收口文字调整后显式链接、编码与精确diff再核。controller语义自查确认本轮仅增加对接说明，没有扩大选择权限。

本轮没有公开API、Rust、脚本或门禁组合变化，不重复Rust/业务、Dimension5或远端全矩阵。主树、两旧提交及旧Wave保持，K-CI-IMPACT-01仍In progress；其它债状态与冻结项不变。本轮只保存本地文档提交，没有合入/推送main或创建PR；接续者使用该干净分支即可，正式main仍按其独立SHA证据读取。
