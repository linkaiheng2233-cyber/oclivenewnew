# CI 分层前的责任边界与现有检查归属 · 2026-10-11

**性质**：维护者确认方向后的有限源码/执行入口对照；不是第二份架构或CI规则。
**状态**：Locally verified（本轮文档/对照）；首片调整为建议，未实施。
**身份**：base `c166af791eba58a8649514bcbe837f031394881f`，分支 `codex/ci-layering-preparation-20261011-r0`；controller实施及语义自查，`independent=false`。

## 四项交付物及归属

| 交付 | 唯一维护位置 | 本轮解决的问题 |
|---|---|---|
| 责任表、双Host依赖图、共享支持的位置 | [MODULE_MAP §0.3.1](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries) | 让参考Host的模型/资源策略与公共承诺分开；允许发行版走自有Host。 |
| 当前实现入口与Memory定点对照 | 同一架构节 | 不凭contracts/runtime/shared等名字分类；区分合同、共享调用、Host材料绑定与ChatPro会话规则。 |
| 13个主CI验证job及其它四workflow归属 | [CI设计](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-responsibility-map)，[英文摘要](../../../creator-docs-en/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-responsibility-map) | 明标混合出口，Nightly节奏与职责分开，release checksum不冒充全部发行版发布。 |
| 一片调整前/后所有者及覆盖保持条件 | 同一CI节 | 首片只建议独立公共契约/兼容检查；保留Host组合回归、原平台/feature及完整测试集合，选择策略另议。 |

## 当前事实与停止线

公共Base端口和最小角色校验已有明确文件；共享消费者借用Host绑定的能力；参考Host `process_minimal_local_conversation` 的生产代码准备会话材料并调用Memory，不强制经过六Base消费者；ChatPro最小会话store在切换绑定时清理会话。`minimal_llm.rs` / `minimal_event.rs` 当前已存在；不能把更早切片的“未找到”带到此基线。旧架构 §0.3 的历史SHA/行号与限定结论保留，不冒充重新全仓核验。

主CI有13个验证job，Rust与前端各两平台，加规划/gate共17个实际job；`rust` 混合公共/共享/Host/桌面，CLI和Dimension5也有混合职责。当前catalog的公共profile仍包含下游影响，这不是可删边的依据。其它workflow分别拥有Nightly条件检查、锁文件审计、限定人工重跑和server checksum产物。

只读前轮五workflow调查可复用；本轮核上述直接入口、生产/测试区分和现有命令即可决定建议形状。未调查每个测试的所有异常、全仓物理拆分或性能矩阵。达成四项交付物就停止；首片实施前仅对两crate迁移目标做有限原/新集合与编译条件核对，若无法保持覆盖则不移除原执行。

## 基线与证据口径

已集成main `24f62303e9bdea37061fe44acc7a0830dc519f98` 的正式[CI 38111785703](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38111785703)证据沿用[上一快照](WAVE-20261011-CI-BASELINE-HANDOFF.md)，不是本轮重跑或新SHA结论。本轮再次只读确认主树干净、HEAD与实际远端main均为该SHA。开发分支base `c166af79` 有前面的分类文档和有限本地入口；本轮只加六个Markdown路径，不改变任何主CI控制面。

`check:kernel-base` 只覆盖原四步的有限公共基础/编译文档检查；两crate本身仍有rich/外围兼容面，包含未进入窄入口的现有测试，不能直接按crate名等同小Kernel，也不能从workspace排除它们后用窄入口宣称覆盖保持。所有权建议不新增权限/调度/公共强制义务或执行策略，不关闭K-CI-IMPACT-01或其它债。

## 文档验收与保全

精确六路径：MODULE_MAP、CI设计中英、ROUND-02-PLAN、DEBT_CHANGELOG、本报告。五份磁盘原件和15项控制面/实现摘要先保全于本机ignored `.cursor/plans/ci-boundary-map-20261011-r0/`；PLAN/DCL历史正文只追加。原件与命令stdout/stderr/native exit分件保留，旧报告/业务身份/日志不改。

适用八组命令全部native exit0：默认链接52份、显式六路径链接、docs-only旧路径、镜像、登记、债结构、编码，以及严格UTF-8无BOM/汉字、两份历史字节前缀、五原件、15项控制面/实现摘要与精确六路径diff。命令stdout/stderr/退出码分件保留；收口文字更新后复核受改链接、编码与精确保全。controller语义自查确认四项交付物、历史证据限定及MIXED标签，不称独立复核。纯文档没有Rust/API/脚本/control-plane变化，不重复Rust、模块功能、Dimension5或远端CI；文档结构通过不代替实现验收。

本轮仅保存本地文档提交，不合入/推送main、不建PR、不启动新的CI。用户确认的是责任方向；下一片执行迁移仍待对接。签名/TLS/双核/Full韧性/多Agent/高级情绪记忆/Production Stream等冻结，以及S01、真实模型/TTS、平台实机、全部崩溃窗口和独立浏览器形态的结论均不扩大。
