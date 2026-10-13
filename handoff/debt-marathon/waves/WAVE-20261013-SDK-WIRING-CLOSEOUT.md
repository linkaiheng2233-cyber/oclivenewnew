# 源码 SDK 与终端接线视图 · 冻结交接快照

**日期 / 状态**：2026-10-13；**Locally verified**。适用文档、示例与范围检查通过；最终本地提交、tag与工作树状态在交付记录中核对，不宣称新SHA远端CI通过。
**基线**：`abf2f8a5074e863289af94cb2993d3733cfda0f7`；controller实施与语义自查 independent=false。六个Markdown路径，五份原件先保全；[有限计划](../ROUND-02-PLAN.md#源码-sdk-与接线视图--文档及工程收尾2026-10-13)。本报告拥有本阶段交接、复用证据与停止线，不重新定义SDK、元数据或架构。

## 接手人从哪里开始

1. [源码 SDK Memory 指南](../../../examples/developer-sdk-memory/README.md)：从基础工程到作者实现、三项测试与共享消费者调用；先走主开发路径，再按需查看声明。
2. [CLI接线说明](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring)：命令前提、唯一字段格式、来源/未知标签和常见错误；[English](../../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring)。
3. [Base接入点清单](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)：公共入口与目标Host接线位置；不是本报告重新维护的接口表。

只需先掌握生成、Cargo check/test/fmt、帮助与可选doctor wiring。基础工程依赖contracts/types，SDK消费者支持由作者显式加入runtime；完整checkout/path和派生锁要求沿原指南。公共Kernel、共享支持、特化Host/发行版责任继续查MODULE_MAP，CI归属继续查既有设计，不在这里重复长表。

## 冻结成果与证据分层

| 切片 | 已交付及实际证据 | 未由其证明 |
|---|---|---|
| SDK `9c0ce96f` | 源码工具包配方、原Memory复用和两个独立工程；十个native命令exit0，原三项测试与消费者实际调用，零生成、无Host/Tauri依赖 | 注册表SDK发布、所有平台/模块功能或完整发行版 |
| 接线 `abf2f8a5` | 现有doctor域新只读命令、脚手架空元数据；bin100+wiring8+基础E2E7、Clippy/fmt及实际声明重读/不改源 | 运行绑定、调用追踪、目标功能质量；仅改Rust不会自动同步声明 |
| 本次文档收尾 | SDK先实现再可选查看、CLI中英版本前提/清单选择/去敏排查、携带此交接快照与本地冻结锚点 | 没有新模块运行或远端正式CI结果 |

原生证据分别见 [SDK Wave](WAVE-20261013-SOURCE-SDK-FIRST-PATH.md) 和 [接线 Wave](WAVE-20261013-WIRING-DECLARATION-VIEW.md)。本机原日志在对应ignored目录/_recovery，不随Git携带；配方和新8项CLI集成等测试可在源码中复现。旧证明只沿用源码/模板/测试逐字不变的范围，不把文档新SHA借旧CI盖章。

## 本轮文档修补与检查

原SDK第5节把可选查看放在“改哪里”的表和实现说明前面；本轮将实现、输入/断言和异步限制留在主路径，可选查看放在后面。CLI指南中英增加帮助探针和五种常见现象定位，明确旧CLI缺命令只是入口版本不匹配、workspace-only清单不能代表package、声明名称不等于核实。格式仍由CLI指南拥有，不复制第二套schema或增新义务。

实际验证记录在本机 `.cursor/plans/wiring-closeout-20261013-r0/`，各原生命令保留stdout、stderr、退出码与执行参数：

- C1：实际CLI的 `doctor wiring --help` exit0，二进制hash与原接线验收相同。
- C2：SDK五段PowerShell示例解析错误0；CLI中英两组TOML示例相同，两组均经实际CLI读取、exit0，查看前后示例清单hash不变。这只证明语法与声明视图，不证明实际绑定或模块执行。
- C3–C9：默认与本轮六文链接、镜像、登记、docs-only旧路径、债结构、`git diff --check`，全部exit0。
- C10：严格UTF-8无BOM/无替换符、中文正常、精确六文写集、五份原件及两份历史正文前缀保全；原接线清单中本轮未改的十份文件逐字相符（其中七份为Rust/模板/测试），main与独立SDK工作树干净且HEAD不变。范围核对exit0。

最终报告修改后仅复查受影响链接、最终字节范围与暂存diff，再提交冻结；不为写入检查结果重复运行无变化的命令。Rust、模板、测试和执行配置保持，沿用原接线115项、Clippy/fmt与真实重读示范的限定证据，本轮未重跑这些测试、完整本地链或远端CI。

## 工程冻结与后续交接

当前分支 `codex/wiring-view-20261013-r0` 承载 SDK → 接线 → 文档收尾的线性提交；旧完整SHA仍可追溯。收尾验收通过后本地提交并创建注释tag `checkpoint/source-sdk-wiring-20261013-r0`，不覆盖旧锚点。最终SHA/tag/clean状态留交付报告，不为回写绿灯另造提交。tag为本地开发checkpoint，不是SemVer发版、远端正式验收或发布触发，本轮不推送。

main `08348f31b8f3459537c13227302bf4b8e273baa8`及独立SDK冻结工作树保持；main此前正式21/21仅属于其旧SHA。此基线可供接手，合入/发布仍按后续明确的集成范围与相应门禁处理。无需为文档收尾全面重设计CI或新建框架。

第一阶段在作者能生成、编写/验证Memory、找到消费者接线、查看有限声明时结束。反馈附实际源码SHA、CLI帮助、目录、命令、预期/实际输出及去敏最小复现；工具便利性、模块约定和公共契约表达问题分别定位。新需求优先修真实阻碍，不机械补五槽示例、穷尽异常或凭“工程质量”解冻旧架构项。

没有GUI、监控、模块管理、发现/注册、权限/执行策略或Host强制报告；实际绑定/调用/功能保持未核实。原S01语义、真实音频/模型、平台实机、浏览器形态与未覆盖崩溃窗口等各自边界不变。当前无新增需要维护者裁定的问题。
