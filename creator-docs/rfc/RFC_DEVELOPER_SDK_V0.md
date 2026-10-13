# 源码接入工具包 · 第一条模块开发路径

**状态**：维护者已选择源码接入工具包；第一片提供现有组件的组装指南与树外消费案例，不新增公开 SDK 门面。
**更新**：2026-10-13。`V0` 是本文档标识，不是已发布 SDK 的版本或新 API 稳定性承诺。
**SSOT 范围**：SDK 的交付形态与扩展停止线；公共职责仍由 [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries) 拥有。

[English](../../creator-docs-en/rfc/RFC_DEVELOPER_SDK_V0.md)

## 开始使用

模块作者先读 [源码 SDK Memory 路径](../../examples/developer-sdk-memory/README.md)：使用现有 CLI 生成基础工程，显式添加共享运行库，复制已有 Memory 示例，在仓库外编译、测试并运行。指南回答“改哪里、运行什么、应该看到什么”，也说明源版本、锁文件和移动目录的要求。

现有 [基础工程](../cli/OCLIVE_CLI_GUIDE.md) 继续保持契约与公共类型依赖；SDK 路径是作者随后选择添加消费者支持的动作，不改变基础模板的默认内容。生成物不携带参考 Host、桌面壳或回合调度。示例中的教学绑定由调用方明确提供，不注册为默认模块。

## 第一片交付什么

| 组成 | 用途与当前形态 |
|---|---|
| `oclive_kernel_contracts`、`oclive_kernel_types` | 使用已有六槽 Base、请求、返回与完整失败；按现有兼容规则开发 |
| `oclive_kernel_runtime` | 作者显式选择的共享接入支持；本路径使用既有 `MinimalRoleBaseConsumer` |
| 独立消费工程 | 由现有 `init --template kernel-base --base-example none` 生成，再由 Cargo 添加 runtime；自己的 workspace、manifest 与锁文件 |
| 现有 Memory 源码 | 从原例子复制 `main.rs`、`memory.rs` 与仓库许可证；只维护一份教学实现，作者在自己的工程修改 |
| 使用与验证说明 | [操作指南](../../examples/developer-sdk-memory/README.md)及可复制的项目 README；指向原测试、接入点和目标 Host 的后续责任 |

三个 crate 当前版本均为 `0.2.0`，本片使用源码 path 依赖，未发布新的 registry SDK 包。整个 contracts/types crate 还含兼容面，runtime 也有其它支持，不能把这三个 crate 整体等同于纯净小 Kernel。实际职责和可用接线查 [源码对照](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map) 与 [Base 接入点清单](../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。

## 如何判断通过

| 层次 | 第一片实际核对 | 不能据此推出 |
|---|---|---|
| 工程可用 | 独立工程 metadata/check/test/run；锁与源码输入可追踪 | 全平台打包或 SDK 注册表发布已完成 |
| 有限契约行为 | 选择与原文保留、正常空结果、完整 Unsupported 失败、调用材料范围 | 所有 Memory 实现的检索质量、持久化或性能合格 |
| 消费者接入 | 作者实现真实经过现有消费者调用；生成调用为零 | 每轮必须执行六槽或已注册到参考 Host 的丰富后端 |
| 目标功能 | 由模块作者及目标 Host 补实际场景验收 | 本例已经替所有发行版验收 |

本片 Windows PowerShell 树外试验的源码输入为 `08348f31b8f3459537c13227302bf4b8e273baa8`；使用已准备的依赖缓存离线执行。操作与结果见 [本轮记录](../../handoff/debt-marathon/waves/WAVE-20261013-SOURCE-SDK-FIRST-PATH.md)。这是有限本地证据，不借该输入 SHA 的正式 CI 为新的文档提交盖章；其它平台的独立工程配方本片未执行。

## 接入前继续由作者决定的事项

- 材料、资源、状态、权限和异步执行器的来源。源码依赖或成功调用不授予网络、工具或外部硬件权限。
- 自己的实现接受什么要求；示例的 `literal:` 语法归作者，不是 Base 的新增通用义务。
- 用什么 Host 接线。参考 Host 路径可按 [CLI 指南中的 library-embed](../cli/OCLIVE_CLI_GUIDE.md) 配方继续；自有 Host 依照现有公共契约实现。SDK 本片不启动两者。
- 如何交付及升级依赖。保存实际源码 SHA、dirty 状态、工具链和派生锁；更新源码时按影响核对自己的调用和下游兼容。

## 扩展条件与停止线

### 下一片开发准备：配方自动回归（未实现）

第一条源码接入路径已具备实际消费者案例；下一片优先降低维护这条路径的重复操作成本。准备将当前指南的生成、显式添加runtime、复制canonical示例、派生锁和check/test/run接入既有CLI集成测试设施。具体写集、验收与停止条件见[开发计划](../../handoff/debt-marathon/ROUND-02-PLAN.md#审查批完整交接与源码-sdk-开发准备2026-10-13)，当前冻结与证据边界见[收尾记录](../../handoff/debt-marathon/waves/WAVE-20261013-SDK-READINESS.md)。这是一项尚未实施的回归计划，不是已经存在的新SDK命令。

| 准备项 | 下一片要求 |
|---|---|
| 交付对象 | 现有Memory源码工具包；独立作者工程真实编译、测试并经过现有消费者调用 |
| 复用入口 | CLI原集成测试辅助设施及原Memory源码；不复制维护另一份模块、不新增公开SDK门面 |
| 判断通过 | 独立workspace/派生锁与依赖边界成立，真实子命令成功且行为断言成立；声明视图与空测试不代替调用证据 |
| 结束条件 | 一条上手路径可重复回归后冻结；Host接线、其它槽、平台、发布按实际需求另片 |

源码工具包依旧使用现有crate与path依赖，不因增加回归而成为脱离完整checkout的可发布SDK包。新命令、额外包装与跨语言绑定不在这项准备的授权范围内。

开发者能按指南生成、跑通检查、定位自己的实现与消费者绑定，即冻结第一条路径。反馈优先提供失败命令、源码 SHA、工具版本和最小复现，沿 [CLI 反馈入口](../cli/OCLIVE_CLI_GUIDE.md) 处理。

后续只有具体开发动作被现有交付方式阻碍时，再讨论包装、其它槽案例、Host 预设、跨语言绑定或发布方式；不机械增加六套同形案例，不引入发现系统、通用执行器或新的 SDK trait。公共契约表达障碍须提供新旧用例；权限、调度权或新增强制义务仍由维护者确认。硬件目标、真实模型/TTS、独立浏览器形态与未覆盖崩溃窗口不因 SDK 指南完成而转绿。
