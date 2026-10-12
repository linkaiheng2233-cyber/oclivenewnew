# 默认参考 Host 配置的 CI 所有者 · 2026-10-12

**状态**：Locally verified，定向、适用治理及完整本地链通过；冻结后的目标SHA正式CI另判，不先宣称新配置已远端通过。
**基线**：本地 `f4f0ecdbf4165c1f5fa7ad87235919217b5922b8`，main `12bc7dc8243a2d3c3bf17f98306da1cf4cb8ca92`，父main正式 [38156014581](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38156014581) 已19/19和ci-gate成功。controller实施与语义自查 independent=false。
**范围**：[计划](../ROUND-02-PLAN.md#参考-host-默认配置的正式执行所有者2026-10-12)、[默认Host所有权说明](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#reference-host-default-owner)、[三层架构SSOT](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)。本报告不另定义架构或技术债状态。

## 原因与停止线

实际Cargo tree显示：独立Host为 `default`，旧workspace默认为 `default,tauri-commands`。前者是已支持的无桌面使用配置，原CI默认被桌面feature合并；dual_core又是另一已有配置。为Host开发者把它自己的默认配置接入两平台CI，原桌面组合检查原样保留。这里是两种实际feature条件的验收，不是同条件同命令的双重执行，也不通过移走Host减少覆盖。

直接manifest/命令和两次feature观察已足以行动，停止逐cfg调查。新增 `reference-host-default` 调用原有固定入口，保留default=[]、Host doctest=false、不启用可选feature；原公共组、remaining workspace、dual及组合继续负责各自条件。不给Kernel增公共义务，不开发新Host框架或重新定义7B策略。

## 关联闭环与保全

workflow执行既有入口 → catalog有限validator/command及五个旧profile → 受信comparison-base拓扑探针 → gate真实needs → 既有规划器与21影子样本 → 文档/回归。探针检查两个所有者的唯一正确数组映射；旧基线缺组/坏种类/混合所有权/坏JSON均走既有full bootstrap，不使用PR自己的catalog批准少跑。执行策略、影响图和ready选择类别逐字不变。

本片12路径，前片f4与之形成16路径同主题批次。11份原件及14项不变源/控制/manifest保存在ignored `.cursor/plans/ci-reference-host-owner-20261012-r0/`，PLAN/DCL历史前缀保留。Host源码/API、manifest、锁、架构/债台账、旧入口/helper、旧Wave及其它工作流不改。原workspace的步骤、资源和参数按原字节核对；格式仅规范本次Rust测试。

## 实际定向结果与失败登记

| 证据 | 原生结果与用途 |
|---|---|
| `G0b/G0c` | Cargo tree默认/旧workspace条件观察均exit0，原始输出保留。 |
| `R1-owner-contract-red` | 新仓库回归在旧配置exit101，首因“default Host configuration must have a distinct owner”；证明配置缺口，不是产品回合失败。 |
| `G1-node-regressions` | 25/25，默认/两公共入口/native helper；拓扑旧目录、混合/错种类/坏JSON和真实gate两组失败/缺失/取消/skip负控通过，docs合法skip保持。 |
| `G2-ci-plan-contracts` | exit101，11 lib通过；仓库8项中3失败：旧validator计数、旧bootstrap提示、我新增的raw换行匹配不兼容CRLF。原日志保留；前两项同步明确的新配置，后一项改为既有lines匹配，不改变判据。 |
| `G3/G4` | scoped fmt exit0；修正后规划器11 lib＋8仓库回归共19 passed，native exit0；21影子数据经真实Planner对照。 |
| `G5-reference-host-default` | 原样Host入口真实Clippy/tests，native exit0，51.48秒；lib637＋集成119＝756 passed，19 ignored，0 failed，未执行ignored场景。 |
| `G6/G15-scope` | 11原件/14不变摘要、历史前缀、12路径/合批16路径相符；原14个验证job的YAML执行配置一致，catalog/影子除新组坐标外保持，影响图与执行策略不变。 |
| `G7–G12` | 默认及7份改文链接、镜像、登记、docs-only旧路径、债结构均exit0，编码由scope逐文件严格UTF-8核验。 |
| `G13/G14-diff` | 首次exit2，新增workflow行的CRLF被Git按trailing whitespace拒绝；先保全当前修改件，仅将31条报错新行改为LF，原行未整体改写；后续exit0，YAML语义与scope仍通过。 |
| `G16-full-local-chain` | 原 `npm run check:ci-local` 完整链native exit0，700.89秒；Dimension5、前端/Rust检查及workspace集成/CLI串行E2E实际执行。不以局部通过代替完整链，未额外运行ignored业务场景。 |

三项断言补丁首尝试因原行定位不相符被apply_patch拒绝，未写入；读取精确原行后处理。Node回归未引入第二份gate实现，仍跑已冻结实际policy CLI；Rust回归不依赖ignored本机证据，因此能在新clone运行。stdout/stderr/native exit分别保存，不把exit0称为零stderr。

换行规范化脚本首尝试把总新增34行误当全部带CR的新行，安全断言在写入前拒绝；实际31条与G13逐行报错一致，原修改件hash复核后处理。该工具误计不影响产品运行或原件保全。

## 验收出口与未测范围

本地配置为15验证job、四组两平台矩阵、规划/gate共21实际job；定义/数量是结构，性能比例未知。将与f4的既有默认入口合批，稳定后完成适用完整本地链、冻结提交并一次push；新目标SHA正式CI在最终交付记录，不回写本报告另造HEAD。

完整本地链复用已读取的ignored原生npm wrapper，Python覆盖只在验证子进程作用；当前源码、锁、命令及stdout/stderr/立即保存的exit构成证据。wrapper并非新的工程入口，也不借父SHA正式CI。与前次34分44秒不能直接推导提速比例，资源/缓存与改动范围不同。

K-CI-IMPACT-01仍In progress。shared/runtime及部分治理仍在mixed组，旧桌面feature组合不声称已拆净；未来ready选择扩大、required减少或公共语义变化另议。真实模型/TTS、平台实机、所有崩溃窗口及独立浏览器的历史边界不扩展。
