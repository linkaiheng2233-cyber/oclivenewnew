# 源码 SDK 最终范围审查、集成冻结与远端交付

**SSOT范围 / 日期**：本次里程碑的分类交接、实际验收和推送边界；2026-10-13。操作仍由[Memory指南](../../../examples/developer-sdk-memory/README.md)拥有，形态由[SDK RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)拥有，不复制公共契约或开发规则。
**身份**：controller实施与语义自查，independent=false。整批起点main `08348f31b8f3459537c13227302bf4b8e273baa8`，本轮接续干净 `89590c0d1eb745deaa012080b2aa9c72a38fd1db`；按[最终计划](../ROUND-02-PLAN.md#源码-sdk-最终范围审查集成冻结与远端交付2026-10-13)收口。最终提交、tag、原生退出件和远端状态在本次交付报告核对，不以旧main正式CI证明本批。
**冻结时状态**：Locally verified，正式远端验收随后按目标SHA另核。本报告保存冻结时本地事实，不为追写CI状态移动基线。

## 可转让的内容与提交分类

保留以下原始提交，不压平历史或移动旧checkpoint。本次最终交接只新增本报告与PLAN/DCL追加，不再扩充SDK功能。

| 分类 | 提交 | 接手者得到什么 |
|---|---|---|
| 源码SDK开发入口 | `9c0ce96f` | 从纯基础工程显式添加共享runtime，复制canonical Memory；独立工程、派生锁和真实消费路径 |
| CLI只读接线声明 | `abf2f8a5` | `doctor wiring`读取可选Cargo元数据，Base/实现独立标示，未核实状态明确；模板与真实CLI回归 |
| 初始冻结及有限审查 | `2d36bb01`、`9c2f142a`、`072fa31b` | 上手交接、近期审查与二次意见的裁定，保持原件和误报更正 |
| 工具入口整理 | `6947c48a` | 三个固定检查脚本复用同一入口守卫，固定命令、超时和原生错误语义保持 |
| 配方准备与真实回归 | `aebe7c46`、`4107d52e` | 使用当前CLI，在临时树外工程真实生成、metadata/check/test/run；坏Memory副本必须出现具名行为失败 |
| 作者教学及文档导航 | `b7581292`、`89590c0d` | 实现/绑定/输入/断言定位，窗口2→1练习；人类与AI复用同一权威资料和三种验证分责 |

源码、测试、配方、指南和报告随Git转让。本机缓存、临时作者工程与 `.cursor/plans/` 原生记录不会自动转让；接手者按现有命令复现，不把目录存在或文档状态当作执行证据。

## 最后一次有限语义检查

本轮核八条直接声明及其实现/直接消费者/回归：工具包形态、真实Memory消费、具名负控、Base与实现分离、只读/去敏失败处理、作者教学、规范/案例/历史导航、固定脚本入口。均未发现新的阻断问题；未为证明不存在而扩大搜索。这是controller自查，不称独立审查、全仓质量证明或AI理解测评。

SDK仍只复用既有contracts/types/runtime，没有新增公开门面、模块发现/注册或统一验收器。基础模板只带contracts/types，runtime由作者显式添加；共享消费者借用调用者绑定，不获得Host资源或调度权。Memory示例的literal规则、材料顺序和窗口不是六槽Base强制义务。

`doctor wiring`每次读取作者声明；替换实现后作者同步声明才改变视图。它不扫描Rust、不绑定/调用模块，不证明目标功能；初始空元数据保持未知。新增回归实际覆盖元数据更改、未知信息、错误/私有内容、参数冲突和无工程执行。

教学把官方配方回归、作者自己的模块测试和目标Host场景验收分开。作者练习的三阶段红/绿和真实selected2→1输出由[教学记录](WAVE-20261013-SDK-MEMORY-TEACHING.md)拥有，本轮按相同源码与步骤复用，不冒称重新执行。AI入口只导航，不增加运行Agent、授权或阅读全仓义务。

公开Rust契约、依赖锁、Host/发行版生产实现、workflow/catalog/选择策略、required gate和技术债状态均无本批变化。先前架构冻结与独立未覆盖项保持；不因SDK可靠性检查自动变更其状态。

## 本地里程碑验收

本机原生stdout/stderr/退出件与八项语义核对在 `.cursor/plans/sdk-final-closeout-20261013-r0/`。使用Node 22.23.2、Cargo 1.97.1、既有Cargo缓存/target、主树已有node_modules的Junction与显式portable Python；未安装或升级依赖。

- G2：固定入口/工具链与三类检查脚本回归28/28，exit0。
- G3–G9：默认及本批改文链接、镜像、登记、docs-only旧路径、债结构和编码，均exit0。
- G1：`npm run check:ci-local`首跑native101，停在Tauri build的缺失资源路径。Dimension5已PASS31（sample lib按--ci明确SKIP），前端lint/typecheck/build与分层/镜像/fmt均已通过；不把整条首跑写成exit0。
- G10：核实隔离资源目录后使用现有 `bundle-kernel-for-tauri --profile debug`，exit0；没有启动模型、服务或产品回合。这是完整工程编译准备，不是SDK依赖要求。
- G11：续跑 `npm run check:rust`，exit0；分层、CHANGELOG镜像、文档镜像、workspace fmt、全目标/全feature Clippy及workspace library测试通过。
- G12：`check:rust:integration`的workspace段通过，CLI段native101；旧Monolith三项在临时项目自己的target查找产物，而本机私有capture全局设置的 `CARGO_TARGET_DIR` 将实际产物移到了共享target。正常build已成功，故不修改产品、旧断言或降低测试。
- G15：首版移除target覆盖的私有执行器仍将null写成空环境值，CLI依赖树单测因此native101（99过/1败）。实测Windows PowerShell下 `SetEnvironmentVariable(...,$null)`留空条目，改用Env provider删除并断言条目不存在。原执行器两版及失败输出保全；最终仅外层Cargo显式指定target，子工程保留自己的target。
- G13：module-compat exit0；10 slots、9 manifests、7 UI contributions是当前输出，不是SDK功能或六槽逐轮执行数量。
- G16：CLI默认完整tests串行续跑native0；先前Monolith8/8通过，接线8/8通过，新 `source_sdk_memory_recipe_runs_outside_the_workspace` 实际通过。该目标仍包含临时作者工程metadata/check/test/run、三项binary行为与确切失败负控，不用“未执行的文件存在”替代运行证据。

完整本地链的所有适用阶段现已完成：G1已过的Dimension5/前端部分、G11 Rust和G12已过的workspace集成，加G16 CLI续跑。此为分段通过，不是 `check:ci-local`一次exit0；不因环境修正再重复已通过的整段。最终三文适用门禁及只读diff总审之后提交/冻结，停止新增测试。首跑与续跑使用独立日志，不覆盖失败。三次非零分别是隔离资源缺失、执行器target作用域和空环境值问题；不得合写成SDK产品缺陷。构建复制的plugin bridge与原Git blob相同，仅刷新索引stat，无该资产提交。

## 冻结和远端验收边界

计划新增注释tag `checkpoint/source-sdk-final-20261013-r0`，精确推送当前分支和本轮tag。它是开发里程碑，不是SemVer发布；保留十个分类提交与六个本地checkpoint，不使用全量tags快捷脚本。历史片“不push”是当时授权，本次用户已明确授权远端交付。

main启用 `ci-gate` 保护，先建立ready PR接受目标提交的正式 `ci.yml`。本地检查与冻结不冒充远端通过；待正式CI成功后，在交付报告记录精确head、run与门禁，不为回写编号另造提交。本次不绕过保护或自动合入main；远端冻结分支/tag是可转让的SDK基线。

达到作者能按文档生成、实现、真实调用、建立有效断言并理解接入边界的完成线即停止。后续收实际使用反馈；只有出现另一个槽的具体不同障碍才补案例。没有统一任意模块验收、目标Host全流程、真实模型/TTS、平台实机、全部崩溃窗口、浏览器形态或性能矩阵的承诺；这些各按自己的目标与证据处理。
