# 源码 SDK：从基础工程到 Memory 消费者 / Source SDK: foundation to Memory consumer

本路径让模块作者在 **OCLive 仓库外** 拥有自己的 Rust 工程、Memory 实现和测试，并通过现有共享消费者真实调用。它复用原 [Memory 源码](../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs) 与 [调用/测试](../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs)，没有第二份维护中的实现。

This path creates an author-owned Rust project outside OCLive, reuses the canonical Memory example and calls it through the existing shared consumer. Follow the same PowerShell blocks below. No new SDK crate or generator is required. Delivery scope: [中文 RFC](../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md) / [English RFC](../../creator-docs-en/rfc/RFC_DEVELOPER_SDK_V0.md).

## 1. 准备 / Prepare

- 使用包含 `kernel-base` 的**完整 OCLive 源码 checkout**，从它的根目录打开 PowerShell。同一会话执行下面步骤；不能只下载本 examples 目录。
- 准备能构建该 checkout 的 Rust/Cargo 与本机链接工具。本路径不需要 Node、前端资源、参考 Host、Ollama 或真实模型。首次构建可能下载 Cargo 依赖；本轮试验使用已备缓存及 `--offline`，不代表新机器无须准备依赖。
- 输出使用仓库外的**新目录**。已有非空目录会被 CLI 拒绝；不要为重试而覆盖作者已有工程或锁文件。

Use a full checkout, a compatible Rust/Cargo toolchain and local linker. Run from the checkout root in one PowerShell session. Choose a new directory outside the repository. Prepared caches allow `--offline`; otherwise Cargo may need its normal dependency preparation. Node, desktop resources, models and a running Host are unnecessary.

记录自己的输入，不把示例的测试版本当成自动锁定：

```powershell
git rev-parse HEAD
git status --short
rustc --version
cargo --version
```

本轮 Windows 配方在源码 `08348f31b8f3459537c13227302bf4b8e273baa8` 实跑。后续使用者记录自己的 SHA、工作状态和工具版本；path 依赖会读取当前磁盘，Cargo.lock 不锁定它们的 Git 内容。This recipe was tested on Windows at that source SHA; record your own inputs. A Cargo lock does not pin the contents of path dependencies.

## 2. 创建基础工程 / Generate the foundation

```powershell
$ErrorActionPreference = 'Stop'
$sourceRoot = (Resolve-Path -LiteralPath '.').Path
$projectRoot = Join-Path (Split-Path $sourceRoot -Parent) 'my-sdk-memory'

cargo run --locked -p oclive-cli -- init --non-interactive --no-smart --template kernel-base --kernel-source "$sourceRoot" --base-example none --license Apache-2.0 --project-name sdk-memory-caller -o "$projectRoot"
if ($LASTEXITCODE -ne 0) { throw 'Foundation generation failed.' }
```

这里选择 `none`，先生成四文件基础工程；仍可使用六槽公共契约，没有关闭运行时 Memory。`--no-smart` 保持明确的生成动作；其它选项和冲突规则见 [CLI 指南](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)。

`none` omits the teaching test, not the six-slot contracts. At this point the project depends only on contracts/types. The next step explicitly adds consumer support. See the [English CLI guide](../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md) for other options.

## 3. 添加消费者案例 / Add the consumer example

在**刚生成的项目**中复制原例子和许可证。下面对新增文件采用拒绝覆盖；只有生成的 README 被有意替换为本路径的项目说明。保留生成的 `src/lib.rs`。

Copy the canonical files into the newly generated project. New files are not overwritten; replacing the generated README is intentional because this project now adds runtime. Retain `src/lib.rs`.

```powershell
$exampleRoot = Join-Path $sourceRoot 'kernel/crates/oclive_kernel_runtime/examples/external_memory_base'
[System.IO.File]::Copy((Join-Path $exampleRoot 'main.rs'), (Join-Path $projectRoot 'src/main.rs'), $false)
[System.IO.File]::Copy((Join-Path $exampleRoot 'memory.rs'), (Join-Path $projectRoot 'src/memory.rs'), $false)
[System.IO.File]::Copy((Join-Path $sourceRoot 'LICENSE'), (Join-Path $projectRoot 'LICENSE'), $false)
[System.IO.File]::Copy((Join-Path $sourceRoot 'examples/developer-sdk-memory/project-README.md'), (Join-Path $projectRoot 'README.md'), $true)

# Seed only this new project; Cargo derives a lock for its different root package.
[System.IO.File]::Copy((Join-Path $sourceRoot 'Cargo.lock'), (Join-Path $projectRoot 'Cargo.lock'), $false)

Push-Location -LiteralPath $projectRoot
try {
    cargo add --path "$sourceRoot/kernel/crates/oclive_kernel_runtime"
    if ($LASTEXITCODE -ne 0) { throw 'Adding runtime failed.' }
} finally {
    Pop-Location
}
```

这不是新的 SDK 命令：`cargo add` 是现有 Cargo 操作。种子锁来自同一源码 checkout，Cargo 会为新根包与依赖集合派生锁，**不要求整份锁与 OCLive 根锁逐字相同**。保存派生后的 Cargo.lock；以后用 `--locked` 拒绝意外解析变化。已有项目不要套用这段复制，更不要覆盖其锁。

This uses ordinary Cargo. The seed lock is derived for the new root package; byte equality with OCLive's workspace lock is not expected. Retain the derived lock and use `--locked` for subsequent checks. Never apply the copy block over an existing project or replace its lock.

基础模板只带 contracts/types，runtime 由这一步显式加入。实测依赖图为一个 workspace 成员、87 个 package、82 个第三方锁身份，第三方身份均来自原锁；没有 `oclive_kernel_host`、Tauri、tao 或 wry。本地支持还会传递引入 `oclive_validation`。这些是该输入版本的观察值，不是未来版本的数量上限。

The tested graph had one workspace member, 87 packages and 82 third-party locked identities, all from the source lock; no reference Host or Tauri. `oclive_validation` is a transitive local dependency. Counts describe that source input, not a future limit. Do not equate the whole runtime crate with the small Kernel.

## 4. 检查并运行 / Check and run

```powershell
Push-Location -LiteralPath $projectRoot
try {
    cargo check --locked
    if ($LASTEXITCODE -ne 0) { throw 'Project check failed.' }
    cargo test --locked
    if ($LASTEXITCODE -ne 0) { throw 'Project tests failed.' }
    cargo run --locked
    if ($LASTEXITCODE -ne 0) { throw 'Consumer example failed.' }
} finally {
    Pop-Location
}
```

缓存已准备时，可分别给 Cargo run/add/check/test 添加 `--offline`；不要给 `init` 子命令添加该 Cargo 参数。本轮原生退出码均为 0，测试为 **binary 3 passed**；保留的空白 library 和 doctest 各 0 项不能作为模块功能证据。

With prepared caches, add `--offline` to the Cargo command, before `--` where applicable, not to `init`. The recorded run passed three binary tests; the blank library and doctest each ran zero tests and are not functional proof.

运行的第一行应是：

```text
external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0
```

随后输出选中的两条原始材料，包括否定与引述。`main.rs` 通过 `MinimalRoleBaseBindings.memory` 借用作者实现，再调用 `MemoryBase::retrieve(&consumer, request)`。绑定六个引用不要求六槽一轮全跑；这里只调用 Memory，`NoGeneration` 计数为零。

Expect the first line above followed by the two unchanged selected materials, including negation/quotation. The actual call goes through the shared consumer. Six borrowed bindings do not prescribe a six-slot turn sequence; only Memory is called and generation remains zero.

## 5. 我改哪里 / What to edit

### 可选：看工程接线声明 / Optional: view project wiring declarations

基础脚手架现在初始提供空的 `package.metadata.oclive.wiring`。按 [CLI 接线字段与 Memory 示例](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring) 填写自己的声明：例如本路径的 Memory 实现为 `RecentLiteralMemory`、接入为 `MinimalRoleBaseConsumer`、用途为 `project`，调用范围声明只含 `memory`。已生成空块时补字段或替换它，不重复同名表；旧工程没有该块时可自行添加。

The foundation now initializes optional empty wiring metadata. Follow the [CLI format and Memory example](../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring), adding fields or replacing the empty block once. In this case the author declares the Memory implementation/consumer and a Memory-only scope; an old project can add the block explicitly.

从本配方保留的源码根执行 / From this recipe's source root:

```powershell
cargo run --locked -p oclive-cli -- doctor wiring --manifest-path "$projectRoot/Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw 'Wiring declaration view failed.' }
```

应看到 `MemoryBase -- declared --> RecentLiteralMemory`，其余未填写的实现保持未知；绑定、调用与功能均未由视图核实。更换 Rust 实现时同步作者声明，再运行命令查看变化。它不修改项目，也不运行本例；实际调用和3项测试仍用前面的 Cargo 命令验证。

Expect the declared Memory edge and unknown unspecified implementations. The view does not verify binding, calls or functionality. Synchronize metadata when editing an implementation; the command rereads it without changing or running the project. The real consumer call and three tests remain separate checks.

| 文件 / File | 开发动作 / Action |
|---|---|
| `src/memory.rs` | 修改自己的 `MemoryBase` 实现及接受要求 / Edit your implementation and accepted requirements |
| `src/main.rs` | 明确材料、查询、绑定与调用，调整对应测试期望 / Supply materials, query and bindings; update tests for the chosen behavior |
| `src/lib.rs` | 基础模板保留的作者库入口；本例 binary 不调用它 / Author library entry retained from the foundation; unused by this binary |
| `Cargo.toml`、`Cargo.lock` | 维护源码 path 与实际依赖输入 / Maintain source paths and dependency inputs |

本例按调用方给出的从旧到新材料，以大小写敏感的 `literal:<exact substring>` 选择最近最多两条，原文不改。`literal:` 是作者协议，不是 Base 新要求。三个测试分别核对消费者选择/原文、正常空结果与完整 `Unsupported` 失败、当前调用的材料范围与窗口；改变实现后用这些明确期望观察差异，不通过删断言或吞失败来让绿灯保持。

The author's agreement selects at most two latest literal matches from caller-ordered materials, preserving text. Three tests check consumer selection/text, empty success versus complete Unsupported failure, and caller scope/window. Update explicit expectations when intentionally changing behavior; do not remove assertions or hide failures to preserve a green result.

`main.rs` 的文件头保留原仓库 example 命令，说明源码来源；这个独立工程执行本页的本地 `cargo run` / `cargo test`。其中的 `immediate` helper 仅能驱动立即就绪的内存实现；真实异步实现由作者提供执行器。源码接入不提供隐式资源、存储、权限、重试、发现或丰富角色默认值。

The copied file header retains the repository example command as provenance; this independent project uses local `cargo run`/`cargo test`. Its `immediate` helper only handles immediately ready in-memory implementations. Real async providers need the caller's executor. No implicit storage, permissions, retry, discovery or rich-role defaults are added.

## 6. 保存与继续接入 / Preserve and continue

- 保存自己的源码 SHA/dirty 状态、工具版本、manifest、派生锁与测试结果。原例子仍由 OCLive 维护；复制后作者改动归自己的工程。
- 移动源码 checkout 或生成工程后，核对 Cargo.toml 的所有 path；生成器与 Cargo 可能写相对路径，目录移动不会自动修复。源码 checkout 中的 workspace 依赖仍需要整棵源码树。
- 可用 `cargo metadata --locked --format-version 1` 检查实际 workspace 根与依赖图；“不链接 Host”不等于模块的目标功能或所有平台已验收。
- 工程与有限契约测试通过、消费者实际调用成立，是这条路径的完成线。检索质量、性能、持久化、目标 Host 生命周期与发行版产品行为仍另验。

Record source/tool/lock inputs, recheck path dependencies after moving either directory, and use Cargo metadata to inspect the actual graph. The first path establishes compilation, limited contract behavior and a real consumer call. Retrieval quality, persistence, performance, Host lifecycle and distribution behavior remain target-specific work.

后续入口 / Continue with:

- [六槽 Base 接入点 / Base entry points](../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)：请求、返回、失败、绑定和 Host 接线位置。
- [Memory 开工包](../../human-docs/modules/slots/memory.md) / [English](../../human-docs-en/modules/slots/memory.md)：基础实现与丰富后端的不同路径。
- [CLI 的 library-embed 配方](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)：需要参考 Host 时再选择的路径，不是本配方的依赖。
- [CLI 指南](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)：生成参数、作者模板及既有反馈入口。反馈附失败命令、版本与最小复现即可；本片不另建反馈系统。
