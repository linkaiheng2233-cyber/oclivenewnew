# Kernel 基础工程 · Memory 第一条上手路径（2026-10-11）

**状态：Locally verified。** controller 直接实施及语义自查，`independent=false`；本片未合入或推送 main，未发正式远端 CI。它接续[基础脚手架片](WAVE-20261011-KERNEL-BASE-SCAFFOLD.md)，不撤销前片本地结论，不借侧对话的只读核对称独立审查。

起点 `bc2b7c64046dd93f0b309d32b19d9c510cc1dd56`，仍使用独立 `codex/kernel-base-scaffold-20261011-r0` 分支和 `E:/OCLive/_worktrees/kernel-base-scaffold-20261011-r0` 工作树；main 为 `2b0390cd`。计划见 [本轮有限范围](../ROUND-02-PLAN.md#kernel-基础脚手架--memory-第一条上手路径2026-10-11)，事件见 [DCL-88](../DEBT_CHANGELOG.md#dcl-20261011-88--memory-第一条开发上手路径)。

## 1. 侧对话建议的处理

基础产物仍是引用既有六槽 Base/types 的工程，五文件、无 Host/runtime/桌面/默认模块/调度器。前片已验证生成、编译、依赖和错误路径，本片补齐作者能直接执行的学习动作。主树仍只有五种模板是尚未合流的事实，不能据此把已验分支标成未实现。

没有另造六套案例或统一测试框架，也没有修改公共契约、权限、模块注册或运行行为。复用已存在的 Memory 外部实现与 `MinimalRoleBaseConsumer`；消费者示例留在 runtime example 中，不进入纯基础工程的依赖。

## 2. 第一条路径与读者入口

生成工程 → 实现/理解 `MemoryBase` → trait 直接调用测试 → 修改并观察结果 → 原消费者绑定案例 → 查目标 Host 接线。

- [生成 README 模板](../../../kernel/crates/oclive-cli/src/templates/kernel_base/README.md.hbs)：先改哪里、运行什么、应看到什么；解释实例协议、异步执行器和四层测试结果。
- [Memory 人类开工包](../../../human-docs/modules/slots/memory.md)与[英文摘要](../../../human-docs-en/modules/slots/memory.md)：从基础工程到已有消费者的动作路由。
- [六槽接入点清单](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)：请求、结果、失败、绑定、参考实现与 Host 接线的唯一表；本片不复制或改写它。
- [CLI 指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)与[英文指南](../../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md)：Rust Base 基础工程与原 minimal 配方、目录/Remote RPC 插件、参考角色包和 UI 插入面分别判读，不更改旧入口。

教程练习只改生成工程的测试内实现：`item.contains(needle)` → `item.ends_with(needle)`。第二条材料以 `report` 而非 `tea` 结尾，实际结果由一条变为空，旧期望应失败。只有作者确认新约定后，才将第二次断言期望 `second` 改为 `Vec::<String>::new()`；若非预期则恢复实现，不能为绿灯削弱要求。原 `Unsupported` 用例在整个练习中仍通过。

真实模块放入作者的源码入口或自建文件，说明功能约定和资源范围。Pending future 需调用方的适用 executor；案例的即时 helper 无法承担任意异步模块。

## 3. 四层结论分开

| 层次 | 本批结论 | 边界 |
|---|---|---|
| 工程可用 | 通过：实际生成、编译与测试 | 当前本地源码依赖、工具链及缓存环境；不是已发布 SDK |
| 契约符合 | 通过（有限）：示例使用原 Memory 请求/结果/完整失败，调用经过作者实现 | 不替代任意模块/其它槽的定向测试 |
| 接入可用 | 通过（已有案例）：外部 Memory 经真实共享消费者调用 | 不是把本工程或任意 Base 自动注册到参考 Host |
| 目标功能可用 | 教程只核已声明的字面匹配行为；其它目标未验 | 检索质量、长期记忆、性能、发行版场景由模块作者及目标发行版验收 |

材料、来源、资源权限和调度由调用方/Host 明确，消费者只使用显式借用的绑定。六个引用不意味着每轮六槽全跑。Host 的有限现有接线入口在原清单中，具体 rich backend 注册仍另行接线。

## 4. 实际验证与证据

本机证据目录：ignored `.cursor/plans/kernel-base-onboarding-20261011-r0/`。初始8份原件和 SHA 保存于 `baseline.json` / `originals/`；所有命令 stdout/stderr/立即退出件分别保存。它们不随 Git 携带，复现用下面的命令与源码。

| 命令 | native 结果 | 证据前缀 |
|---|---|---|
| `cargo test --locked --offline -p oclive-cli --test e2e_kernel_base -- --test-threads=1 --nocapture` | exit 0，4 E2E passed | `T1-onboarding-e2e` |
| `cargo clippy --locked --offline -p oclive-cli --all-targets -- -D warnings` | exit 0，无 Clippy 诊断 | `G1-cli-clippy` |
| `cargo fmt --all -- --check` | exit 0，无差量 | `G2-fmt-check` |
| `cargo run --locked --offline -p oclive_kernel_runtime --example external_memory_base` | exit 0，`selected=2 baseline=3 memory_calls=1 llm_calls=0` | `T2-consumer-run` |
| `cargo test --locked --offline -p oclive_kernel_runtime --example external_memory_base` | exit 0，3 passed | `T3-consumer-test` |

新增 E2E 在实际生成工程执行教程的三个阶段：原始测试通过 → 仅改实现后非零（1 passed/1 failed，实际 `[]` 与原材料期望不等）→ 按新约定改期望后2 passed/0 failed。故意红的原始输出完整进入父测试日志，父 E2E 正确验证该失败后通过；不能把嵌套红写成整批失败或“从未出现 FAILED”。测试临时工程关闭后移除，没有改仓库的 Memory 示例实现。

原三条 E2E 同轮继续验证直接依赖图无 Host/runtime/Tauri/SQLx/Axum、初次五文件、注册表零写入、参数拒绝、只读预览/检查、缺 source和非空目录保持。生成工程内2条测试与4条父 E2E分开计数。旧五模板的113条相关回归与51条 workspace doctest证据继续只绑定前片，不冒称本片新 SHA已跑；没有修改其生产路径或任何公开契约，故不重复整套。

已有消费者的实现、绑定与调用源码逐字未改。本轮实际运行/测试用既有 example，使用当前内存材料，禁止生成的 LLM 计数0；不启动 Host、模型、语音、监听器或访问用户库。

文档链接通过：默认52份，显式8份（7份改文＋实际生成 README）；docs-only路径、镜像、登记、债结构、编码/diff均通过。精确九路径、8份原件 hash 和计划/DCL 原历史字节前缀保持，实际生成 README 的3个仓库链接目标已核存在；镜像检查不替代中英语义自查。只读查看生成结果可用本机 ignored `generated/my-capabilities/README.md`，生成命令日志为 `A1-onboarding-example`，exit 0。

## 5. 有限自查与停止线

生成 README 的教程、实际 Base 实现、E2E 的改动与新期望同一闭环；没有只改文案却宣称动态证明。原公共失败保持，作者更新期望须先确定其新功能语义。教程指向已有消费者，不创建第二份实现、loader或默认注册。原 RPC/角色/UI 入口只澄清用途。

九路径之外不变：Base/templates的实现正文、所有生产 Rust、共享消费者、模块表、锁/依赖、配置、工作流/CI、债务状态与前片报告保持。计划/DCL只追加，既定冻结和未测范围不扩大。

作者能生成、跑检查、修改一个示例看到变化并定位原消费者后即停。本片保存第二个本地提交，不合入或推送 main；无新增需要维护者决定的架构事项。后续 SDK、Host 预设和 CI分层分别按目标推进，不能从本教程自动解冻或扩验证。
