# Kernel 基础脚手架：合批集成里程碑

**性质 / 日期**：2026-10-11的集成交接快照，不是新的能力或门禁SSOT；controller实施与语义自查，`independent=false`。本报告冻结前描述已有交付和验收安排，实际本地退出码及最终SHA正式CI终态以原始回执和交付报告为准，不预写通过。

## 1. 交付和基线

维护者已确认脚手架足以支撑现阶段第一步，并授权集成验收和推送。起点为干净分支 `codex/kernel-base-scaffold-20261011-r0` 的 `46318810aaa8eda7f6f77b434a9bd3565942d354`，本地/实际远端main为 `2b0390cdecaeed428f9ea1f753098f4aa69f3ebd`。这次集成六个既有切片和本交接记录，不混入SDK草案或其它债分支。

开发者从[CLI基础工程入口](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start)生成现有contracts/types工程，经[Memory开工路径](../../../human-docs/modules/slots/memory.md)修改和验证示例，再按已有消费者案例理解实际绑定。问答可选Memory教学或空白内容；[本地作者模板](../../../examples/scaffold-kernel-base-custom/README.md)允许维护文件布局和字符串参数，沿用现有来源锁及逐次确认。常用/专项命令已分开，使用问题按现有仓库反馈入口给最小复现。

工程可编译、有限契约符合、已有案例接入及作者目标功能分别判断；空白工程零项测试不是功能验收。产物没有参考Host/runtime/Tauri、默认六槽实现或回合调度；六Base保留，none只省教学内容。不是已发布SDK，也未承诺独立可启动小Kernel或全部平台/发行版。范围及旧原始证据见[最终文档收尾](WAVE-20261011-KERNEL-BASE-CLOSEOUT.md)，使用和格式SSOT继续由原指南及[RFC](../../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)拥有。

## 2. 合批审查与验收

按[本批计划](../ROUND-02-PLAN.md#基础脚手架--合批集成验收与推送2026-10-11)和[变动事件](../DEBT_CHANGELOG.md#dcl-20261011-93--基础脚手架合批集成验收)核整个差量。配置/问答/参数→共享生成及预览→来源/文件边界→默认和空白工程→原模板与错误回归→文档是本能力闭环；不因CLI旧结构仍包含Host链接功能就给新基础模板增加Host。公共traits/DTO、现有运行接口、默认feature、锁、权限、模块调度及CI策略保持。

里程碑只运行一次适用完整本地组合，不为六个子提交重复矩阵。使用仓库已有debug资源前置和 `npm run check:ci-local`；CLI嵌套Cargo集成测试由原串行阶段拥有。验证子进程不继承外层CARGO_TARGET_DIR，辅助Voice显式使用已核本机Python，不把本机绝对路径写成跨机器要求。有公开模板枚举增量，按公共面口径显式执行 `cargo test --workspace --doc`，不以workspace lib测试冒充覆盖。适用文档、原件/历史、编码、精确写集和diff另核。

所有实际进程结束后确认同一冻结内容及main/远端身份，才快进main并一次推送；正式出口为目标完整SHA的 `ci.yml` 和成功 `ci-gate`。远端观察故障与job失败分别处理；真正失败只按首因修复，不无变化rerun。远端绿灯后保持HEAD，不为回写数字或run编号制造新提交。

## 3. 证据位置与接手

本机ignored `.cursor/plans/kernel-base-integration-20261011-r0/` 保存基线、两旧原件、合批受检文件身份、stdout/stderr/native退出码、清理和环境、最终远端快照。这些原件不随Git携带；跨机器以提交内代码/测试/指南及目标Actions为入口，不能补造本地历史。

```sh
git rev-parse HEAD
gh run list --workflow ci.yml --commit <完整SHA> --json databaseId,headSha,status,conclusion,url
```

达到本次验收后停止主动扩展脚手架，后续由具体使用反馈启动有限修改。维护者授权的下一条线为CI责任分层准备，设计SSOT见[现有CI路线](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#24-主工作流的执行所有权)：从Kernel公共契约向Host接入、具体Host/发行版、发布组合划分；不默认取消required gate、改变Push/ready策略或把Host业务反向定义为Kernel承诺。签名/TLS、双核、Full韧性、多Agent、高级情绪记忆等既定冻结及真实模型/音频/平台/崩溃/独立浏览器证据边界均不扩大。
