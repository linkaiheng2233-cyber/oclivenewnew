# 源码 SDK：从基础工程到 Memory 消费者 / Source SDK: foundation to Memory consumer

本路径让模块作者在 **OCLive 仓库外** 拥有自己的 Rust 工程、Memory 实现和测试，并通过现有共享消费者真实调用。它复用原 [Memory 源码](../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs) 与 [调用/测试](../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs)，没有第二份维护中的实现。

This path creates an author-owned Rust project outside OCLive, reuses the canonical Memory example and calls it through the existing shared consumer. Follow the same PowerShell blocks below. No new SDK crate or generator is required. Delivery scope: [中文 RFC](../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md) / [English RFC](../../creator-docs-en/rfc/RFC_DEVELOPER_SDK_V0.md).

**资料导航 / References**：本页拥有开发操作与案例；接口承诺、责任与兼容分别查[Base接入点](../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)、[Kernel/共享支持/Host边界](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)及[已采纳兼容审阅](../../creator-docs/COMPATIBILITY.md#six-slot-minimal-compatibility-draft)（[English](../../creator-docs-en/COMPATIBILITY.md#six-slot-minimal-compatibility-draft)）。选择模块/Host/发行版路径见[人类入口](../../human-docs/README.md#按角色选择路径)与[AI任务导航](../../handoff/AI_READING_INDEX.md#sdk-task-reading)；AI修改与核实继续沿原规则，不因本案例获得运行权限。The guide owns practical steps, while linked authorities own contracts/boundaries/compatibility; the human and AI entries select task-specific reading.

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

未修改的案例运行第一行应是：

```text
external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0
```

随后输出选中的两条原始材料，包括否定与引述。`main.rs` 通过 `MinimalRoleBaseBindings.memory` 借用作者实现，再调用 `MemoryBase::retrieve(&consumer, request)`。绑定六个引用不要求六槽一轮全跑；这里只调用 Memory，`NoGeneration` 计数为零。

Expect the first line above followed by the two unchanged selected materials, including negation/quotation. The actual call goes through the shared consumer. Six borrowed bindings do not prescribe a six-slot turn sequence; only Memory is called and generation remains zero.

## 5. 我改哪里 / What to edit

| 文件 / File | 开发动作 / Action |
|---|---|
| `src/memory.rs` | 修改自己的 `MemoryBase` 实现及接受要求 / Edit your implementation and accepted requirements |
| `src/main.rs` 的 `retrieve` | `MinimalRoleBaseBindings.memory` 借用传入的 `&dyn MemoryBase`，然后调用消费者 / Bind the supplied implementation and call the consumer |
| `src/main.rs` 的 `main` | 作者实现的构造、材料、查询及运行结果检查 / Construct your implementation, supply inputs and check the run result |
| `src/main.rs` 的 `tests` | 按自己的功能承诺编写正常结果、空结果和失败断言 / Assert normal, empty and failure behavior promised by your module |
| `src/lib.rs` | 基础模板保留的作者库入口；本例 binary 不调用它 / Author library entry retained from the foundation; unused by this binary |
| `Cargo.toml`、`Cargo.lock` | 维护源码 path 与实际依赖输入 / Maintain source paths and dependency inputs |

本例按调用方给出的从旧到新材料，以大小写敏感的 `literal:<exact substring>` 选择最近最多两条，原文不改。`literal:` 是作者协议，不是 Base 新要求。三个测试分别核对消费者选择/原文、正常空结果与完整 `Unsupported` 失败、当前调用的材料范围与窗口；改变实现后用这些明确期望观察差异，不通过删断言或吞失败来让绿灯保持。

The author's agreement selects at most two latest literal matches from caller-ordered materials, preserving text. Three tests check consumer selection/text, empty success versus complete Unsupported failure, and caller scope/window. Update explicit expectations when intentionally changing behavior; do not remove assertions or hide failures to preserve a green result.

`main.rs` 的文件头保留原仓库 example 命令，说明源码来源；这个独立工程执行本页的本地 `cargo run` / `cargo test`。其中的 `immediate` helper 仅能驱动立即就绪的内存实现；真实异步实现由作者提供执行器。源码接入不提供隐式资源、存储、权限、重试、发现或丰富角色默认值。

The copied file header retains the repository example command as provenance; this independent project uses local `cargo run`/`cargo test`. Its `immediate` helper only handles immediately ready in-memory implementations. Real async providers need the caller's executor. No implicit storage, permissions, retry, discovery or rich-role defaults are added.

### 练习：作者窗口从两条改为一条 / Exercise: change the author's window

只编辑按第2–3节生成的**作者工程**，不修改OCLive官方原件。本例的上限已参数化，练习修改作者的构造配置；更换算法或类型时，改自己的 `src/memory.rs`，同步 `main.rs` 的import/构造和 `retrieve` 所借用的实现。`calls()` 是本例观察调用的方法，不要求所有Memory实现提供它。

1. 先按第4节运行，确认三项binary测试通过、输出 `selected=2`。
2. 在 `main` 和 `tests::public_consumer_calls_the_external_selection_without_rewriting_material` **这两个位置**，将 `RecentLiteralMemory::new(2)` 改为 `RecentLiteralMemory::new(1)`。此时保留旧期望；另外两项测试不改。
3. 在作者工程运行下面命令，应该看到这个行为测试 **FAILED**；若是编译或依赖错误，先解决那个错误，不能算作断言发现了行为变化。`sdk-memory-caller` 是本配方的binary名称，自己的工程名称不同则替换它。

```sh
cargo test --locked --bin sdk-memory-caller tests::public_consumer_calls_the_external_selection_without_rewriting_material -- --exact
```

4. 按新承诺“保留最新一条原文”调整这两个位置的期望：`main` 的 `selected != materials[2..]` 改成 `selected != materials[3..]`；该测试的 `assert_eq!(selected, materials[2..]);` 改成 `assert_eq!(selected, materials[3..]);`。保留baseline、调用计数、零生成、其它测试及原错误处理，再运行自己的 `cargo test --locked` 和 `cargo run --locked`。应三测通过，运行第一行变为：

```text
external memory: selected=1 baseline=3 memory_calls=1 llm_calls=0
```

随后只输出Dave那条原文。这表示通过现有消费者调用的选取行为已改变，不代表目标Host已经采用它。期望由作者事先决定；不能根据实际输出自动改期望或删除失败断言来保绿。

Edit only your generated project. First confirm the original three tests and `selected=2`. In `main` and the named selection test, change the constructor's limit from 2 to 1 and leave expectations unchanged; the command above must fail that test, not compilation. Then change those two expected slices from `materials[2..]` to `materials[3..]`, preserving the other assertions and tests. Your own test/run commands should pass and print `selected=1` plus only Dave's unchanged material. This exercises an existing author configuration; a different implementation needs its own promised behavior and meaningful assertions. The example's literal syntax and call counter are not universal Base requirements.

### 可选：看工程接线声明 / Optional: view project wiring declarations

先按上表实现、测试模块，再按需查看声明。此可选步骤要求使用含 `doctor wiring` 的CLI源码/二进制；旧版本不认识命令时先核对 [CLI版本前提与排查](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring)，不因此改动Base接口或否定前面已经通过的消费者测试。

Implement and test first, then optionally view declarations. This step needs a CLI containing `doctor wiring`; see [version prerequisites and troubleshooting](../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring). An older CLI does not invalidate a passing consumer test.

基础脚手架现在初始提供空的 `package.metadata.oclive.wiring`。按 [CLI 接线字段与 Memory 示例](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring) 填写自己的声明：例如本路径的 Memory 实现为 `RecentLiteralMemory`、接入为 `MinimalRoleBaseConsumer`、用途为 `project`，调用范围声明只含 `memory`。已生成空块时补字段或替换它，不重复同名表；旧工程没有该块时可自行添加。

The foundation now initializes optional empty wiring metadata. Follow the [CLI format and Memory example](../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring), adding fields or replacing the empty block once. In this case the author declares the Memory implementation/consumer and a Memory-only scope; an old project can add the block explicitly.

从本配方保留的源码根执行 / From this recipe's source root:

```powershell
cargo run --locked -p oclive-cli -- doctor wiring --manifest-path "$projectRoot/Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw 'Wiring declaration view failed.' }
```

应看到 `MemoryBase -- declared --> RecentLiteralMemory`，其余未填写的实现保持未知；绑定、调用与功能均未由视图核实。更换 Rust 实现时同步作者声明，再运行命令查看变化。它不修改项目，也不运行本例；实际调用和3项测试仍用前面的 Cargo 命令验证。

Expect the declared Memory edge and unknown unspecified implementations. The view does not verify binding, calls or functionality. Synchronize metadata when editing an implementation; the command rereads it without changing or running the project. The real consumer call and three tests remain separate checks.

## 6. 保存与继续接入 / Preserve and continue

- 保存自己的源码 SHA/dirty 状态、工具版本、manifest、派生锁与测试结果。原例子仍由 OCLive 维护；复制后作者改动归自己的工程。
- 移动源码 checkout 或生成工程后，核对 Cargo.toml 的所有 path；生成器与 Cargo 可能写相对路径，目录移动不会自动修复。源码 checkout 中的 workspace 依赖仍需要整棵源码树。
- 可用 `cargo metadata --locked --format-version 1` 检查实际 workspace 根与依赖图；“不链接 Host”不等于模块的目标功能或所有平台已验收。
- 工程与有限契约测试通过、消费者实际调用成立，是这条路径的完成线。检索质量、性能、持久化、目标 Host 生命周期与发行版产品行为仍另验。

Record source/tool/lock inputs, recheck path dependencies after moving either directory, and use Cargo metadata to inspect the actual graph. The first path establishes compilation, limited contract behavior and a real consumer call. Retrieval quality, persistence, performance, Host lifecycle and distribution behavior remain target-specific work.

### 三种验证分别负责什么 / Three distinct checks

| 验证 / Check | 执行位置与用途 / Where and why |
|---|---|
| 官方配方回归 / Official recipe regression | OCLive checkout中维护官方上手路径，使用下面的 `e2e_source_sdk`；不自动发现、加载或验收任意作者模块 / Maintain this official recipe, not arbitrary modules |
| 作者模块测试 / Author module tests | 自己工程的 `cargo test` / `cargo run`，按自己的承诺检查真实调用、结果与失败；上面的窗口练习属于这里 / Verify your own promises and actual calls |
| 目标Host场景验收 / Target Host acceptance | 按目标Host的实际接线与场景检查主流程是否使用该实现及功能是否满足要求；独立工程通过不代替这一层 / Verify adoption and behavior in the chosen Host |

`doctor wiring` 只显示作者声明，不能作为上述实际调用或功能通过的证据。The declaration view establishes none of these runtime or target-functionality results.

### 维护者回归入口 / Maintainer regression

从完整OCLive checkout的根目录执行，预先准备Cargo依赖缓存：

```sh
cargo test --locked -p oclive-cli --test e2e_source_sdk -- --test-threads=1
```

一个外层集成测试会在临时独立工程中生成基础、添加runtime、复制原例子并真实check/test/run。它核对三项binary行为、消费者输出、workspace/依赖与派生锁稳定性；还会改坏临时Memory副本，要求原消费者测试真正失败，然后清理临时目录。该负控的子命令非零是预期，外层测试只有确认失败原因符合断言才通过。查看声明不代替此回归；这是维护者验配方的入口，作者日常开发仍使用第4节的Cargo命令。

Run from a full OCLive checkout with prepared Cargo dependencies. One outer integration test creates a temporary author workspace and really checks/tests/runs the copied example. It checks three binary behaviors, consumer output, dependency boundaries and derived-lock stability, then deliberately breaks the temporary Memory copy and requires the original consumer test to fail. That negative child's nonzero exit is expected and verified before the outer test passes and cleans up. This maintainer regression does not replace the author's ordinary commands or target-specific acceptance.

后续入口 / Continue with:

- [六槽 Base 接入点 / Base entry points](../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)：请求、返回、失败、绑定和 Host 接线位置。
- [Memory 开工包](../../human-docs/modules/slots/memory.md) / [English](../../human-docs-en/modules/slots/memory.md)：基础实现与丰富后端的不同路径。
- [CLI 的 library-embed 配方](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)：需要参考 Host 时再选择的路径，不是本配方的依赖。
- [CLI 指南](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)：生成参数、作者模板及既有反馈入口。反馈附失败命令、版本与最小复现即可；本片不另建反馈系统。
