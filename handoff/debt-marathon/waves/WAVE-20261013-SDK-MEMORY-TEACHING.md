# 源码 SDK 单个 Memory 案例教学收束

**SSOT范围 / 更新**：本片教学改动、练习证据与交接；2026-10-13。配方与教学步骤由[Memory指南](../../../examples/developer-sdk-memory/README.md)拥有，SDK形态仍查[RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)，不新增模块验收规则。
**身份 / 状态**：**Locally verified**。起点干净 `4107d52e5a50c9a7eabb8f0c86f15576b8c9bff5`；controller实施与语义自查 independent=false。按[五文计划](../ROUND-02-PLAN.md#源码-sdk-单个-memory-案例教学收束2026-10-13)完成实际练习及适用门禁后本地冻结，最终SHA/tag/clean以原生交付记录为准。

## 教学变化

现有指南的生成、依赖、复制与运行步骤复用，不新编一套工具。第5节把实现文件、`retrieve`的消费者绑定、`main`的输入与`tests`的断言分别定位；加入一条作者窗口2改1的练习。第6节明确官方配方回归维护官方案例、作者Cargo测试检查自己的承诺、目标Host另验真实接线与场景。复制README只指向这套说明，初始示例输出与作者修改后的预期分开。

练习修改的是已有Memory实现的作者构造配置，算法本身未改；更换实现类型/算法时，作者同步自己的import、构造、绑定和对应断言。literal协议及calls观测不升级成Base通用义务。官方 `e2e_source_sdk.rs` 不发现或验收任意模块；doctor声明视图不作为调用/功能证据。

## 作者工程真实练习

新工程在本机 `E:/OCLive/_recovery/SDK-MEMORY-TEACHING-20261013-R0/author-memory/`。原始、旧期望及新期望三个源码阶段保留在同级stage目录；本机原生stdout/stderr/退出件在工作树 `.cursor/plans/sdk-memory-teaching-20261013-r0/`，不随Git自动转让。使用已有缓存/offline、现有Cargo target，未启模型或Host。

| 原生命令记录 | 真实结果 |
|---|---|
| P1–P5 | 原配方生成/add/check/test/run均exit0；作者binary三测通过，输出selected=2/baseline=3/memory_calls=1/llm_calls=0 |
| P6 | 只改main与首个选取测试的窗口2→1，保留旧预期；指定测试exit101、确切测试名FAILED、0 passed/1 failed，属于预期行为断言失败而非构建失败 |
| P7/P8 | 两处预期材料区间2..→3..，其它测试与断言保留；三测通过、run exit0，selected=1且只输出Dave原文，baseline/调用计数/零生成不变 |
| P9 | 官方配方回归exit0、外层1 passed；仍验证官方原例子，未因作者实验改变官方Rust/测试 |
| P10/P11 | 修改后的作者工程fmt与all-targets Clippy -D warnings均exit0 |

原Memory副本、派生锁与官方两份canonical源码保持；本轮仅作者main的两处窗口及两处对应预期变化。G1–G8均exit0：默认及五文链接、登记、docs-only旧路径、债结构、严格编码/五写集/四原件/历史前缀/三阶段精确变化/官方原件与旧tag保全、diff及指南五段PowerShell解析。最终文字只复查受影响链接、范围及暂存diff，不重跑已通过的练习。

**执行者命令错误**：首次从作者工程调用私有编辑脚本时误用源码工作树相对路径，脚本未执行，红测试也未启动。工具原始输出在会话中，`X0-command-path-failure.json`是归因摘要而非原始stderr；改用已核实绝对路径，X1/X2才执行作者编辑。该错误不作为P6行为失败或产品缺陷，不补造失败日志。

## 收口与停止线

本片五份Markdown，官方Rust、测试、模板/公共面、manifest/锁、CI/required和债状态不改，旧报告与checkpoint保留。只有project README文本作为复制输入更新，因此实际复跑官方SDK目标；不重复原115项、官方Clippy/fmt、Dimension5、全量CI或发行版验收。适用检查通过后本地提交并新增 `checkpoint/source-sdk-memory-teaching-20261013-r0`，main保持，不push或宣称远端新SHA通过。

完成线是作者能找到实现/绑定/输入/断言，修改一个行为，先看到测试发现变化，再按自己明确承诺更新预期并看到真实输出。当前练习已实现该线；以此冻结并收实际使用反馈，不机械补其余五槽、任意模块验收器、注册/发现、SDK门面或Host强制接口。Windows/已有缓存的有限证据不代表其它平台、新机器、模块质量或目标Host主流程通过；原架构冻结和历史未覆盖边界保持，当前没有新架构问题需维护者裁定。
