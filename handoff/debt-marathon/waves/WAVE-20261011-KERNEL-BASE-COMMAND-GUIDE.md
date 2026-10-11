# 基础脚手架 · 常用与不常用指令（2026-10-11）

**状态：Locally verified。** controller 直接实施及语义自查，`independent=false`。用法 SSOT 仍为 [CLI 指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#cli-common-commands)及当前帮助；本报告只记录本片范围与实际证据。

基线 `c33e8a0fa6cd72e710a454da2cbe4892600c13bb`，复用 `codex/kernel-base-scaffold-20261011-r0`；main 保持 `2b0390cd`。范围见[七路径计划](../ROUND-02-PLAN.md#基础脚手架--常用与不常用指令2026-10-11)，事件见 [DCL-90](../DEBT_CHANGELOG.md#dcl-20261011-90--基础模块指令的常用与不常用分类)。接续[首次使用便利性](WAVE-20261011-KERNEL-BASE-FIRST-USE.md)。

## 1. 阅读层次及原因

按基础 Rust Base 模块作者的开发路径分类：创建工程时用显式 `init --template kernel-base --kernel-source <源码根>`；日常使用 `cargo check`、`cargo test`、`cargo fmt -- --check`；忘记生成参数时查询 `init --help`。生成/CLI帮助在OCLive源码根执行，后三项在生成工程根执行。

模板浏览、预检/预览、脚手架包、原参考运行配方、RPC插件、角色包和维护/试验/兼容入口归入不常用，按实际任务查阅。频率不表示稳定性或废弃状态，原试验开关与权限规则保持。例如作者修改Memory匹配逻辑后可先检查与测试，不必提前了解角色包发布。

中英文CLI指南拥有完整分类；CLI crate README及生成README给出各自入口，不另建命令体系。原crate README的运行配方边界标明归属，基础工程不被其占位/完整参考运行时说明覆盖。

本片七个文件全部为Markdown或Markdown模板与追加记录；CLI注册、命令名称/默认可见性/路由、公共契约、运行实现、依赖/锁、测试实现、CI及债务状态没有改变。实际 `--help` 继续显示既有命令，分类体现在开发文档。

## 2. 本地验证

本机 ignored `.cursor/plans/kernel-base-command-guide-20261011-r0/` 保存baseline、六份原件、逐命令stdout/stderr/native退出件和实际生成工程；这些材料不随Git自动转让。检查复用原4项基础E2E，并在新生成工程根实际执行文档的check/test/fmt，不新建镜像测试或测试框架。

| 实际命令 / 执行目录 | 结果 | 原始证据前缀 |
|---|---|---|
| 源码根：`cargo test --locked --offline -p oclive-cli --test e2e_kernel_base -- --test-threads=1 --nocapture` | exit 0，4项父E2E passed | `T1-foundation-e2e` |
| 源码根：显式 `init --non-interactive --template kernel-base --kernel-source . --project-name my-capabilities -o <隔离新目录>` | exit 0，初始五文件 | `A1-generate` |
| 生成工程根：`cargo check --offline --color never` | exit 0 | `T2-generated-check` |
| 生成工程根：`cargo test --offline --color never` | exit 0，2项契约测试 passed | `T3-generated-test` |
| 生成工程根：`cargo fmt -- --check` | exit 0，无格式差量 | `T4-generated-fmt` |
| 默认 / 本片显式Markdown链接 | exit 0，分别52 / 7份（后者含实际生成README） | `V1-links-default` / `V2-links-explicit` |
| docs-only旧路径 / 中英镜像 / 文档登记 / 债结构 | 均exit 0 | `V3-stale` 至 `V6-debt` |

Cargo命令在现有缓存下离线执行，使用外置构建目录；实际生成工程拷入源码锁作为本机验证种子，不是模板第六文件或首次用户必需动作。E2E含教程中的有意失败，父测试按预期通过，不称全部日志没有FAILED。Cargo正常编译stderr和Git的LF→CRLF提示保留。

`V7-scope-encoding.json` 核七路径精确写集、六份原件hash、UTF-8无BOM、计划/DCL原字节前缀、实际生成README的五个仓库链接目标及中英分类锚点。HTTP链接仅对照本地目标与锚点，不能据此宣称远端main已含本片内容。本片生成模板字节在测试后未再修改；最终状态行更新后再核报告链接、精确写集/编码与diff，随后才提交。

## 3. 停止线与接手

常用动作可照做、专项入口可查阅即冻结本地提交。不合入或推送main，不借旧SHA正式CI为本片盖章；文档分类无需重复Rust全测、公开面doctest或全量CI。原工程与消费者证据沿各自提交复用，不声称首次新手/所有平台或目标发行版功能已验。

本片没有架构决策或新模块接线。SDK、Host预设下载、CI分层和既定Deferred事项保持各自范围；本片不改变已有H04、S01、真实语音/平台/崩溃/浏览器边界。
