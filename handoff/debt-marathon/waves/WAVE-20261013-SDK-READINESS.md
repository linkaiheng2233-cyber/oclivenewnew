# 当前批收尾与源码 SDK 开发准备

**SSOT范围 / 更新**：本批收尾交接与下一片开工前提；2026-10-13。SDK形态仍以[RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)为准，实施范围由[计划](../ROUND-02-PLAN.md#审查批完整交接与源码-sdk-开发准备2026-10-13)拥有，不复制六槽或CI的责任表。
**状态 / 身份**：收尾与开发准备 **Locally verified**；不是发布或远端正式验收。起点 `6947c48a2afa4dafc7adcd72c4183f71348fc64f`，工作树干净，controller实施与语义自查 independent=false。最终SHA/tag/clean状态在原生交付记录中核实，不为回写绿灯另造提交。

## 已完成的基线

| 成果 | 冻结入口与证据 | 使用含义 |
|---|---|---|
| 源码SDK第一条路径 | `9c0ce96f`；[Memory指南](../../../examples/developer-sdk-memory/README.md)、[原Wave](WAVE-20261013-SOURCE-SDK-FIRST-PATH.md) | 作者独立工程生成/编译/测试/真实消费者调用已有限本地验证；原三项binary测试与零生成事实不外推所有模块质量 |
| 终端接线声明 | `abf2f8a5`；[CLI指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring)、[原Wave](WAVE-20261013-WIRING-DECLARATION-VIEW.md) | 每次读取作者声明；Base与实现独立显示，绑定/调用/功能仍未核实；原CLI 115项及Clippy/fmt限于原受测范围 |
| 上手整理与审查修补 | `2d36bb01` → `9c2f142a` → `072fa31b` → `6947c48a`；[冻结交接](WAVE-20261013-SDK-WIRING-CLOSEOUT.md)、[审查及修补](WAVE-20261013-RECENT-QUALITY-REVIEW.md) | T1入口重复实际收敛，T2–T5文案与误报已明确；原入口回归28/28与Dimension5 31项通过，不把不同feature的Host检查删成“重复” |

截至本片起点，main与origin/main均为 `08348f31b8f3459537c13227302bf4b8e273baa8`、主工作树干净；SDK独立工作树为 `9c0ce96f13b7804c288783537143b760c89a35b5`、干净。它们未被本次准备改写。main原正式CI只证明原SHA；本地开发分支 `codex/wiring-view-20261013-r0` 承载当前成果，未宣称已合入远端。旧两个checkpoint保持原指向，新ready checkpoint只新增、不移动旧锚点。

## 给接手开发者的顺序

1. 从[RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)确认源码工具包范围，再按[Memory指南](../../../examples/developer-sdk-memory/README.md)生成、修改和运行自己的工程。
2. 接线查看是可选诊断；实现源码与Cargo行为测试仍负责实际调用验证。需要具体Host时查[Base接入点](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，本阶段不启动Host。
3. 下一实施片按计划补此配方的自动回归：复用真实CLI与原例子，不能仅mock命令或复制第二份模块。当前只准备范围，仓库内新 `e2e_source_sdk.rs` 尚未实现。

例如升级共享消费者后，作者目前需要手工再执行生成、加依赖、派生锁与check/test/run。下一片将这一条现有路径纳入可重复回归，及早发现“库内例子正常、作者工程组装失败”的问题；不以此新增SDK API或反向规定Host的调度。

## 可转让与本机记录

Git交接带走源码、测试、指南、报告与完整提交历史；ignored日志、缓存、树外试验和未推送tag不会因复制文档自动转让。SDK原试验在本机 `_recovery/SDK-SOURCE-KIT-PROBE-20261012-R0/`，当前视图试验在 `_recovery/WIRING-VIEW-20261013-R0/`，审查与修补在对应工作树 `.cursor/plans/`。接手人可以按原指南重做，不应把这些本机绝对目录当作可移植依赖。

新机器需准备与该checkout匹配的Rust/Cargo、链接工具与依赖；path依赖读取当前磁盘，派生Cargo.lock不固定源码Git内容。保存自己的源码SHA/dirty、工具与锁；不要用源码种子覆盖既有作者工程锁。当前证据限Windows/已有缓存，不宣称其它平台、新机器或注册表发布通过。

## 本轮检查与收口

本轮五Markdown路径，四原件先保全，计划/DCL历史正文保持。G1–G8实际exit0：默认52文与本轮五文链接、镜像、登记、docs-only旧路径、债结构、严格UTF-8/无BOM、精确写集、原证明/源码不变及diff。范围核对实读原入口测试和Dimension5退出件/stdout，核五源摘要和原Memory两源身份；沿用而未重跑28/28与31项，下一片集成测试尚不存在。原生结果保存在 `.cursor/plans/sdk-readiness-closeout-20261013-r0/`。最终报告文字只复查受影响链接、范围和暂存diff，再提交/核clean；实际最终SHA/tag以交付记录为准。

本片没有源码、测试、模板、依赖或CI控制面变化，复用原受测输入下的结果，不再跑不变的Rust/Dimension5/真实业务或完整远端矩阵。通过后新增本地 `checkpoint/source-sdk-ready-20261013-r0`，完成本地可接手收口；正式集成/发布仍须按实际范围取得目标SHA的相应门禁，不用本地tag替代。

本轮未发现必须改公共契约或新增Host义务的阻碍。SDK自动回归尚未实施，目标功能仍由作者与目标Host验收。签名、TLS、双核、Full韧性、多Agent、高级情绪记忆等已定冻结不变；S01、真实模型/TTS、平台/浏览器/未覆盖崩溃窗口不因本批收尾转绿。
