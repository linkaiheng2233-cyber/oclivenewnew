# Kernel 基础脚手架 · 首次使用便利性与指令范围（2026-10-11）

**SSOT 范围**：本片实施和验证快照；指令用法仍以 [CLI 指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)、当前帮助和源码为准，不维护第二份契约或能力注册表。

**状态：Locally verified。** controller 直接实施和语义自查，`independent=false`；未合入或推送 main、未发新远端 CI，不称独立使用者验收。

基线 `6752124d670d8b40a6403a85948f8cd9898d149d`；使用原 `codex/kernel-base-scaffold-20261011-r0` 分支及独立工作树，main 为 `2b0390cd`。计划见 [十路径范围](../ROUND-02-PLAN.md#kernel-基础脚手架--首次使用便利性2026-10-11)，事件见 [DCL-89](../DEBT_CHANGELOG.md#dcl-20261011-89--基础脚手架的首次使用便利性)。本片接续[工程实现](WAVE-20261011-KERNEL-BASE-SCAFFOLD.md)与[Memory 第一条路径](WAVE-20261011-KERNEL-BASE-ONBOARDING.md)。

## 1. 三件事及作用

| 用户要求 | 实际处理 | 解决的例子 |
|---|---|---|
| 首页 / 文档入口分流 | 根 README 中英和人类 README 中英分别链接 Base 基础工程、主仓贡献、参考 Host 的 RPC / UI 插件；CLI 顶部分流角色包与原运行配方 | 写 Rust Memory 的人不必先走 RPC 插件或桌面启动路线 |
| 首次准备 | CLI 中英在基础生成命令旁列 Rust/Cargo/平台链接工具、包含模板的源码版本、执行目录、依赖缓存、输出与本地源码路径；生成 README 链接同一入口 | 旧版本没有模板时知道先换源码，而不是误用 `--preset minimal`；`--check` 不是完整构建或缓存验收 |
| 练习到库源码 | 生成 README 指示移动 TestMemory 与 impl、保留测试/helper、公开声明、按 package.name 导入，再跑原测试 | `my-capabilities` 用 `my_capabilities::TestMemory` 导入；测试真实调用库内实现 |

只扩已有文档职责，不复制六槽关系表、公共定义或消费者实现。入口只链接，准备归 CLI 指南，生成工程动作归生成 README。源码消费者、Host接线、权限和资源职责不变。

## 2. 当前指令范围快照

本轮实际 `init --help` / `init --list-templates` 和 `scaffold --help` 均 exit 0。以下是用途判读，不表示执行了所有子命令、运行了所有模板或验证了全部发行版。

| 指令族 | 当前用途与边界 |
|---|---|
| `init --template kernel-base --kernel-source <root>` | contracts/types 基础开发库，五文件；作者实现模块，不提供 Host/runtime/默认六槽模块/调度 |
| `init --list-templates` / `init --help` | 查看六种模板及参数；运行配方参数不因此适用于基础工程 |
| 基础 init 的 `--check --json` / `--dry-run --json` | 工具/源码有限检查或五文件计划；不生成工程、不写用户项目注册表 |
| 基础 init 的 `--project-name` / `-o` / `--non-interactive` / `--quiet` / `--author` / `--license` / `--description` | 工程名称、路径、交互/输出方式和 Cargo 元数据；产物仍是基础工程 |
| 五个原运行模板 | `robot-soul`、`robot-gateway`、`dialogue-only`、`headless-api`、`library-embed` 保持原配方；`minimal/mixed/full` 是原配置预设，不是基础工程的层级 |
| `scaffold list` / `inspect` / `validate` | 本地包发现、来源/权限/声明查看和清单校验，不执行其 entry/hook |
| `scaffold resolve [--write-lock]` | 确定性来源解析；明确 `--write-lock` 才写工程锁 |
| `scaffold generate [--dry-run]` | 有限本地 instruction 生成；新目录、digest、权限、当前锁和不受信任来源确认沿既有规则；官方 builtin 仍委托 init/plugin/pack |
| `plugin create` / `pack create` | 分别是参考 Host RPC 插件与参考角色包格式，不能当作 Rust Base 或通用最小角色定义生成 |

相关规则唯一见 [Scaffold RFC](../../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)及 [CLI 基础工程入口](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start)。TUI/交互已有代码，本片未操作真实 TTY；基础首次路径推荐明确 template 的非交互命令。CLI 本身会编译其现有依赖，不能从生成物只有两个直接依赖推导工具本体也只有两个依赖。

`scaffold` 本地声明与 instruction 机制不等于网络下载 Host/SDK；不执行包 entry、hook 或任意命令，不替代 CI 责任分层。基础工程拒绝 preset/backend/server/角色/插件/聊天存储/非默认pipeline/Monolith/双核/weld/quick/smart 和归档/现有项目替代流程，无 stub 或偷偷启用 Host 的回退。这里记录已有范围，没有解冻其余开发主线。

## 3. 实际验证与获取位置

本机 ignored `.cursor/plans/kernel-base-first-use-20261011-r0/` 保存 baseline、9份原件、逐命令 stdout/stderr/native退出件和实际生成工程，不随 Git 携带。所有 Cargo 命令在已有依赖缓存下离线执行，构建产物在 `E:/OCLive/oclive-dev-artifacts/oclivenewnew-cargo-target`，没有改用户全局环境或仓库锁。

| 本轮命令 / cwd | 结果 | 原始证据前缀 |
|---|---|---|
| 源码根：`cargo run --locked --offline -p oclive-cli -- init --help` | exit 0，包含基础模板和原运行参数说明 | `H1-init-help` |
| 源码根：`cargo run --locked --offline -p oclive-cli -- init --list-templates` | exit 0，六模板，原五顺序不变、kernel-base末尾 | `H2-template-list` |
| 源码根：`cargo run --locked --offline -p oclive-cli -- scaffold --help` | exit 0，list/inspect/validate/resolve/generate | `H3-scaffold-help` |
| 源码根：`cargo test --locked --offline -p oclive-cli --test e2e_kernel_base -- --test-threads=1 --nocapture` | exit 0，4父E2E passed | `T1-foundation-e2e` |
| 源码根：显式 `init --non-interactive --template kernel-base --kernel-source . --project-name my-capabilities -o <隔离新目录>` | exit 0，初始五文件及新README | `A1-generate` |
| 生成工程根：`cargo test --offline --color never` | 移库前、按教程移库后各 exit 0，2 tests passed | `T2-before-move` / `T3-after-move` |

生成工程使用源码锁的副本作为本机离线验证的依赖种子，这不是模板第六文件或首次用户的必需步骤；首次用户须有可用依赖缓存或允许 Cargo 下载。初始五文件 raw bytes/hash 保存于 `generated-originals/` 和 `generated-initial-files.json`。移库仅在 ignored 实际工程内，把 struct/impl与需要的导入放入src、pub声明、测试导入自己的库；没有给仓库新增生产模块。

4条父 E2E 继续覆盖新工程依赖范围、运行参数拒绝、检查/预览、源码缺失和输出目录保持，教程内有意红仍被父测试正确验收；不将嵌套预期失败写成全批失败，也不称“日志没有 FAILED”。已存在消费者的3项测试及 `selected=2 baseline=3 memory_calls=1 llm_calls=0` 仍绑定前片的实际运行，不在本片重跑或换身份冒称新证据。

收尾适用检查均通过：默认链接52份、显式10份（9份改文＋实际生成README），docs-only旧路径、镜像、登记、债结构及diff均 exit 0；`V7-scope-encoding.json` 核精确十路径、9份原件hash、UTF-8无BOM、计划/DCL原字节前缀，以及生成README的4个仓库链接目标。本片新增的准备入口锚点确实存在。原件保全和历史检查为controller同范围核验，不称独立审查。Git对README模板的LF→CRLF提示是换行配置提示，编译stderr含正常Cargo进度，不称所有stderr为空。

本片没有公开 API/运行代码/锁/依赖或CI变更，不重复Clippy/workspace doctest/完整远端矩阵，不把旧SHA的绿灯当本片结果。文档最后状态行更新后再检查本报告链接、编码、精确范围和diff，随后才建立本地提交；实际受检生成模板字节在验证后未再修改。

## 4. 停止线与接手

三项操作可找到、可照做，并在生成工程中移库测试通过后停止。本片只保存本地提交，不合入/推送 main或其它切片，不改技术债状态。没有新架构取舍或需要维护者猜测的问题。

适用于有基本Rust能力的模块作者；不承诺零安装、无网络首次构建、真实新手上手时间、全部平台/模型/功能验收。生成工程不是独立可启动小Kernel或发布SDK。SDK、Host预设下载、CI层次迭代、真实模型/TTS、全部崩溃/浏览器形态均继续按各自范围；已有冻结不变化。
