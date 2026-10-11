# 公共契约与兼容检查执行所有权首片 · 2026-10-11

**状态**：Locally verified；新workflow尚未合入/推送main或取得目标SHA正式CI，不是K-CI-IMPACT-01 Done。
**身份**：base `5e483aa613265427c246ae23c916a9c5da7ccd07`；分支 `codex/ci-layering-preparation-20261011-r0`；controller实施与语义自查，`independent=false`。
**规则归属**：[架构责任SSOT](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)、[CI责任表](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-responsibility-map)、[有限计划](../ROUND-02-PLAN.md#ci-分层--公共契约与兼容检查执行所有权2026-10-11)，本报告只记录实施与证据。

## 为什么做及实际收益

原 `rust` 两平台job先构建前端、准备Tauri及kernel资源，再覆盖公共合同、共享运行、参考Host和桌面等多个责任。模块作者需要检查公共合同与旧兼容面时，这个前置范围过大。既有 `check:kernel-base` 是有限反馈，不含所有集成测试，不能直接替代两crate完整检查。

本片给完整contracts/types检查独立出口，不再先准备Host资源；例如修改 `MemoryBase` 公共面时能运行 `npm run check:kernel-contracts`，其相关共享消费者/Host回归仍按原影响关系执行。不承诺时间比例、独立部署的小Kernel或其它Host已验收，也不为所有槽补同形框架。

## 调整前后与覆盖集合

| 集合 | 原所有者 | 本地首片所有者与保留条件 |
|---|---|---|
| contracts/types完整Clippy和默认tests | `rust`，Linux/Windows | 新 `kernel-contracts`，同两平台；all-targets/all-features Clippy后完整默认测试，无lib/test过滤。 |
| 剩余workspace | `rust`，Linux/Windows | 只排除原CLI及迁移两crate根包；共享runtime、Host、server、桌面、fuzz和原资源准备保留。 |
| CLI和既有组合/治理 | 原各job | 所有命令、平台和失败出口保持；workspace fmt仍在 `rust`。 |
| 受信目录与gate | trusted-base规划/catalog/原gate | 5个原选workspace的profile加入新owner；`ci-gate`聚合新job，required名称及原执行策略保持。 |

Cargo metadata的原14个workspace成员（不是default-members）中，CLI已有独立测试所有者；原非CLI根包为13。首片公共2＋剩余11互斥且并集保持，包括workspace fuzz。两crate无自有feature，默认测试条件保持；它们有rich/Stream等兼容面，不整体归为小Kernel。

完整目标为：contracts lib **3**、Base fixture **16**、types lib **46**、最小角色合同 **2**、runtime Stream合同 **2**、OneBot Stream review **8**，共 **77** 单测/集成测试；contracts/types doctest分别 **11 / 17**，共 **28**。这些数字是本地真实完整入口输出，不是只统计窄入口，也不证明所有Kernel硬边界。依赖包仍会编译；两组Clippy用 `--no-deps` 将lint归给所选根包，避免依赖lint把已拆的所有权重新混起来。

主workflow新增一个两平台责任组：验证job定义 **13→14**，配置实际job（含矩阵、planner/gate）**17→19**。未改其它job或为减少数量取消检查；本片不做性能比较。

## 迁移的失败关闭与原策略保持

工作流读取受信base的catalog核新validator及job映射。旧base缺新owner、错误映射或解析失败时，走既有 `trusted_policy_bootstrap` 全量回退；PR自己的catalog不能批准少跑。否则沿用既有safe-draft及ready-docs-only规则，其它ready/Push仍全量。

Node回归运行工作流实际inline探针，覆盖新目录/旧目录/错误映射/坏JSON；原 `ci-execution-policy.mjs` 一字未改，真实CLI负控验证新job选中时失败、取消、skipped或未接gate均拒绝，全量失败也拒绝，原docs合法跳过仍通过。仓库合同核两个job、五profile、两平台、资源边界、剩余命令及聚合出口；既有影响图与ready晋升类别未改。

影子语料保留 **21** 场景（18 targeted / 3 fail-safe）的路径、模块传播及回退条件，仅同步新增validator/job坐标。模拟不执行validator，不能当真实远端Compare或删下游边的依据。

## 本地验证与原始失败

stdout、stderr和立即保存的native exit分别保存在本工作树ignored `.cursor/plans/ci-contract-owner-20261011-r0/`；不覆写旧日志。本片真实命令及有限结构核对如下：

| 证据 | 结果与含义 |
|---|---|
| `R0-node-owner-missing` / `R1-owner-contract-missing` | 新入口缺失exit1 / 原catalog缺owner的合同exit101，先红后绿；不是业务场景。 |
| `G0b-node-owner-regressions` | 新入口6项＋原窄入口4项＋native helper10项，**20/20**；含native失败停止、缺工具、实际trusted探针及原gate负控。 |
| `G1b-planner-regressions` | ci_plan lib **11/11**、repository **7/7**；无修改规划器生产算法。 |
| `G2-public-contracts-full` | 真正执行新完整入口，native exit0；**77** 单测/集成＋**28** doctest、Clippy通过。 |
| `G3-planner-clippy` / `G4-format-check` | 受改Rust测试all-targets/all-features Clippy与workspace fmt exit0。 |
| `G5-execution-policy` / `G6-compare-evidence` | 原策略/Compare自测exit0；不冒充真实远端Compare。 |
| `G7-shadow-corpus` | **21/21**模拟合同一致，18 targeted / 3 fail-safe。 |
| `G8-module-compat` / `G9-plugins-index` | **10 slots / 9 manifests / 7 UI**及**6 plugins**，exit0。 |
| `G10d-dimension5-python-override` | `--ci` **31项PASS**，sample workspace lib按该模式明确跳过；voice纯registry/回归不播真实语音。 |
| `G24c-ci-local-with-resources` | 一次成功的完整 `npm run check:ci-local`，native **exit0**，耗时 **2084.39秒（约34分44秒）**；Dimension5、前端检查/构建、原workspace检查及串行CLI集成全部完成。 |
| 适用文档与有限保全 | 默认及本轮7个Markdown链接、镜像/登记/旧路径/债结构/编码；11份原件、12个控制/实现摘要、历史字节前缀、14路径精确diff、YAML及原/新根包集合核对。 |

`G1-planner-regressions`曾exit101：gate已接新组，但断言要求LF，而新增行是CRLF；修改为逐行比较，不放宽缺job拒绝，也未重写全workflow换行。`G10`及`G10b`曾exit1：本机Windows默认 `py` launcher不可用，第一次只给Python目录PATH未解决launcher选择；核现有源码后以既有 `OCLIVE_VOICE_PYTHON` 在验证子进程指向现存Python，先 `G10c` 单项通过，再完整复跑。原失败与环境处置均保留，不改生产门禁、不安装依赖、不改变用户环境。

`G18-encoding`曾exit1：controller误写不存在的 `check-encoding.mjs` 命令名；核实际入口后执行 `check-doc-encoding.mjs`，不创建第二个编码工具。该失败也保留原日志，源文件编码另由逐写集严格解码及汉字检查核对。

`G19-diff-check`曾exit2：apply_patch在原混合换行的workflow新增了CRLF行，Git把新增CR判为行尾空白。仅按本次diff新增/替换行恢复LF，既有行的换行保留；随后重跑实际workflow探针、仓库合同、YAML/集合和diff检查。没有为修换行重写全文件或弱化Git检查。

`G24-ci-local-milestone`曾exit101：新工作树没有Tauri构建所需的 `resources/oclive-kernel-server`；公共独立入口此前已真实通过，不依赖该资源。核既有构建脚本和输出边界后，`G24b-tauri-resources` 使用仓库原 `bundle-kernel-for-tauri.mjs --profile debug` 准备本工作树的生成资源（native exit0），再执行 `G24c` 完整链通过。两次完整链尝试及资源步骤各有独立日志，未覆盖失败、安装依赖或启动服务。完整链的stdout/stderr分别为294723/35084字节，不将exit0写成“没有stderr”。

完整链会经原 `build:plugin-bridge` / `copy-plugin-bridge.mjs` 写入tracked bridge生成物。收尾先保全当前3342字节及摘要（`G25-bridge-generated.bin` / `G25-bridge-provenance.json`），核它与base Git blob、本次build输出及干净main逐字相同；没有内容差量，无需替换源码。只刷新Git文件状态后该路径不再显示修改，最终写集仍为14路径；不把刷新返回的其它真实待提交路径误当内容恢复失败。这里不声称曾预先保存该生成物的原磁盘换行身份。

## 保全、语义自查与后续出口

精确14路径为workflow、catalog/影子坐标、package npm入口、两个新检查脚本、repository合同、CI设计中英、人类门禁矩阵中英、PLAN/DCL及本报告。磁盘原件先保全，历史PLAN/DCL逐字节只追加；公共Rust正文/API、锁文件、原窄入口/helper、影响图/策略、共享消费者、架构SSOT和原报告未改。

controller语义自查确认：公共完整目标保留；新group不用Host资源；旧选择范围同步两owner；旧trusted-base全量；gate不可静默漏组；其它job原命令与矩阵保持。YAML/集合/摘要核对只是有限实现检查，不构成独立审查或全仓架构合规证明。

本轮没有公共API或运行语义变化，不新增workspace doctest全测、真实模型/服务/业务身份、发布组合或完整远端矩阵。门禁脚本/CI适用Dimension5、受影响定向检查及一次成功的完整本地集成链已经执行；两crate完整入口自身保留28项doctest。只在新变更或失败需要时重跑相关检查，不为记录绿灯再跑全链。后续集成以冻结目标SHA取得正式CI出口，main `24f62303` 及其旧17/17证据不用于为新SHA盖章。

此片停在本地可解释的公共检查执行所有权。K-CI-IMPACT-01继续In progress，其它债务与签名/TLS/双核/Full韧性/多Agent/高级情绪记忆等冻结不变；S01、真实TTS、平台实机、全部崩溃窗口和独立浏览器结论不扩大。下一步优先集成这组关联切片并取得目标SHA正式出口；再根据实际混合检查造成的开发阻碍决定是否进一步拆分，不机械迁移所有job。
