# 源码 SDK 第一条树外消费路径 · 2026-10-13

**状态**：Locally verified；两个工程的真实 native 检查及适用文档门禁通过。controller 语义自查，不称独立复核；冻结提交 SHA 在交付报告披露。
**基线**：`08348f31b8f3459537c13227302bf4b8e273baa8`；main/实际远端此前一致，正式 [38199557652](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38199557652) 已21/21和ci-gate成功。本片不借该 CI 为新 SHA 盖章。
**责任 / 范围**：controller 实施与语义自查 independent=false，M/常规文档交付；[计划](../ROUND-02-PLAN.md#源码-sdk-首片--复用基础工程与共享消费2026-10-12)、[RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)、[配方](../../../examples/developer-sdk-memory/README.md)。十二个 Markdown 路径；七份既有文原件先保全，计划/事件历史只追加。

## 原因与复用选择

维护者已选择现有 crate + 独立工程 + 依赖/验证说明，不新增 SDK 门面。基础模板足够写 Base；实际消费者支持由作者随后显式添加。把这条组装路径交给作者，使“修改 `src/memory.rs` 后跑 Cargo 检查，看到真实消费者结果”成为第一步，而不是先学参考 Host、角色包、桌面与发布系统。

旧草案 `3884cf19` 只作为提案参照，不整体 cherry-pick：其旧计划/事件不导入，硬件后续建议不视为当前操作授权。RFC 的 V0 是文档标识；现有三个 crate 的 0.2.0 不是新注册表 SDK 版本。SDK 形态归 RFC、操作归配方、架构仍链接 MODULE_MAP、CI 仍链接既有设计；未新增第二套规则。

## 真实试验与原生退出码

试验在 2026-10-12 运行，两个工程均位于主仓 Cargo workspace 外：

`E:/OCLive/_recovery/SDK-SOURCE-KIT-PROBE-20261012-R0/{probe,from-scaffold}/`

| 标签与 cwd | 实际命令 / 原生结果 |
|---|---|
| P1 / probe | `cargo metadata --offline --format-version 1`，exit0；从源码根锁种子为新根包形成独立锁 |
| P2 / probe | `cargo check --locked --offline`，exit0，33.66秒 |
| P3 / probe | `cargo test --locked --offline`，exit0，3 passed，42.60秒 |
| P4 / probe | `cargo run --locked --offline`，exit0，selected=2/baseline=3/memory_calls=1/llm_calls=0 |
| P5 / 主仓根 | `cargo run --locked --offline -p oclive-cli -- init --non-interactive --no-smart --template kernel-base --kernel-source E:/OCLive/oclivenewnew --base-example none --license Apache-2.0 --project-name sdk-memory-caller -o E:/OCLive/_recovery/SDK-SOURCE-KIT-PROBE-20261012-R0/from-scaffold`，exit0；现有 CLI 真实生成四文件 |
| P6 / from-scaffold | `cargo add --offline --path E:/OCLive/oclivenewnew/kernel/crates/oclive_kernel_runtime`，exit0；显式增加共享支持，不改模板默认依赖 |
| P7 / from-scaffold | `cargo metadata --locked --offline --format-version 1`，exit0 |
| P8 / from-scaffold | `cargo check --locked --offline`，exit0 |
| P9 / from-scaffold | `cargo test --locked --offline`，exit0，binary 3 passed；library和doctest各0项 |
| P10 / from-scaffold | `cargo run --locked --offline`，exit0，输出与P4相同 |

本机 `capture.ps1` 每个标签保留独立 stdout/err/exit JSON、cwd、源SHA与时间，立即保存 native exit，拒绝同名覆盖。中断后只复读已有证据与输入，不重跑十条命令。配置中的共享 Cargo target 仅对子进程使用并恢复；stdout/err 中 Cargo 编译信息不被写成“零 stderr”。

每个工程的 workspace 都仅一个作者根包，实际图为87 packages，82第三方锁身份；原锁 `(name,version,source,checksum)` 集合包含全部派生第三方身份。直接依赖为 contracts/types/runtime，本地传递 validation；没有 `oclive_kernel_host`、Tauri、tao或wry。数字是当前输入的观察值，不设硬上限。基础 generator 与 Cargo 均可能写相对 path，搬动工程/源码后须维护 manifest；锁不能替 Git SHA 锁定 path 内容。

## 原源码与有限行为

| 输入 | bytes / SHA256 |
|---|---|
| `external_memory_base/main.rs` | 8375 / `CC819ECEA7056F4809B9957D8DA6E8A644FFCAFDADF225F376D574DC19DCC10E` |
| `external_memory_base/memory.rs` | 1950 / `C07FFC662FA771D947217DEF26EE67B59431BBF9234481C80479F508A1D5F145` |
| 根 Cargo.lock 种子 | 188297 / `FE93421CFD6013B286907A9A5DF8C15CC0DEFAB2E9396FBEA0DDB8D23F3A57A3` |

原例子是 Cargo example target，本身没有独立 Cargo.toml；此片才以独立工程的 metadata/check/test/run 核对树外组装。原例子与两个副本逐字一致，未另写或更改 Rust 实现/测试；Apache-2.0 LICENSE 随配方复制。

调用经 `MinimalRoleBaseConsumer` → 作者 `RecentLiteralMemory`，有显式材料和查询。三项原测试覆盖选择与原文保留、正常空结果/完整 Unsupported、当前调用材料与窗口；消费者不重试、不改错误。只执行 Memory，NoGeneration为0；其它教学引用的绑定不构成固定六槽执行。`literal:` 是作者协议；`immediate` 不是通用异步执行器。

## 交付与适用门禁

RFC中英、双语操作指南、可复制项目 README，加工厂愿景/Memory指南中英入口及handoff一行登记；PLAN/DCL只追加，本Wave记录本次证据。指南用已有CLI/Cargo，不新增生成命令、脚本框架、SDKcrate/re-export、权限、默认模块或执行语义。参考Host可选路径只链接library-embed。

| 本轮门禁 | 实际结果 |
|---|---|
| G1 / G2b | 默认52份、改文12份链接均exit0；G2首轮exit1的三个新错误链接已修为现有CLI的library-embed配方，原失败日志保留 |
| G3 / G4 / G5 | 中英镜像、文档登记、docs-only旧路径均exit0 |
| G6 / G7 | 债结构与diff均exit0；结构成功不改变任何债状态 |
| G8d | 精确12路径、7份原件、2份历史原始字节前缀、编码、源码/副本输入、十个native结果、主树干净均通过 |
| G9 / G10 | 重新读取原metadata/锁/源码输入，两个依赖图均通过；不是重跑Cargo或新平台验收 |
| G11 | 指南4段PowerShell由真实解析器检查，parse_errors=0；这是语法检查，不冒充四段新执行 |

G8/G8b/G8c 为本机核验脚本误用字节相等的失败：Git对象、主checkout与新worktree分别可能是LF/CRLF。实际读取后明确跨checkout仅归一换行对照内容，历史前缀和源/副本证据仍按各自原始字节核；LICENSE副本直接对本次复制来源main核原字节。没有改原件或产品来绕过失败，失败输出分别保留，G8d exit0。DCL追加补丁第一次因旧句末定位错误被工具拒绝，读取精确行后处理，未写入错历史。

生成工程的旧基础README先保全，再复制本路径的项目README，源码/锁及P5–P10原日志保持；只是说明消费者支持已显式加入，未重复编译。原公共源码、CLI模板、manifest/锁、CI控制面、债台账和队列均保持；本片无公开面变更，workspace doctest/完整Rust/完整CI不适用。最终收口文案再做受影响链接、编码/保全与暂存diff核对，冻结后不回写绿灯专用提交。

本机原件和门禁证据位于隔离工作树 ignored `.cursor/plans/sdk-source-kit-20261012-r0/`；十条树外证据位于上述 `_recovery`。这些原生记录不随 Git 携带，仓库内配方与旧原例子可供重新生成，不把该机器私有目录当发布包或跨平台验收产物。

## 冻结与边界

适用门禁通过并只读语义自查后冻结本地开发提交，主树 `08348f31` 与旧SDK支线保留；不合入/推送main，不改技术债状态，也不为了绿灯回写另造提交。状态以本地实际检查为准，新的文档SHA未跑完整远端CI。

完成线是作者能生成、编译、修改原Memory实现并找到实际消费者绑定。Windows离线缓存条件已实跑；其它系统的独立配方未跑，模型/语音/质量/性能/目标Host与发行版/硬件/全部崩溃窗口没有新结论。现有组件足够支撑此片，无新架构问题；后续以最小复现或具体开发动作驱动，不机械补五套案例或重开全仓调查。
