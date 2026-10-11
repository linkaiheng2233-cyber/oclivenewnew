# oclive-cli

官方脚手架：提供 `kernel-base` 契约基础工程，以及既有无头参考运行时 / library 集成配方。按目标选模板；模块作者先了解下方常用动作即可开工。

`kernel-base` 必须提供 `--kernel-source`，只引用现有 contracts/types，没有 Host 或占位回退；首次准备和完整生成命令见 [CLI 基础工程入口](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start)。

> **原运行配方的边界**：不传 `--kernel-source` 时，生成工程使用 **std + serde** 占位实现，只验证目录、配置与接口形状；传入 `--kernel-source <主仓根>` 后，`kernel_server` / `library` 会链接当前完整参考运行时，其中 library 暴露 `oclive_kernel_host::OcliveKernel` 进程内门面。这两种生成物都不等于已经物理抽离的“最小内核”；后者仍由技术债 **K-CORE-BOUNDARY-01** 跟踪，见 [TECHNICAL_DEBT_INVENTORY.md](../../../handoff/TECHNICAL_DEBT_INVENTORY.md)。

## 常用指令：基础模块开发

| 动作 | 指令 / 入口 | 执行目录 |
|---|---|---|
| 生成工程 | `init --template kernel-base --kernel-source <源码根>`；[完整命令](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start) | OCLive 源码根 |
| 编译检查 | `cargo check` | 生成工程根 |
| 测试 | `cargo test` | 生成工程根 |
| 格式检查 | `cargo fmt -- --check`；修正格式用 `cargo fmt` | 生成工程根 |
| 查参数 | 下方 `init --help` | OCLive 源码根 |

```bash
# 在 oclivenewnew 仓库根目录
cargo run -p oclive-cli -- init --help
```

生成后的 Memory 示例、移入库源码和消费者入口见生成 README。以上 Cargo 指令属于基础开发工具，不是 CLI 的试验 `test/build`。全部指令按常用 / 不常用的阅读分类见 [CLI 指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#cli-common-commands)；频率分类不改变稳定/试验状态。

## 不常用指令：按需专项

模板浏览、`init --check/--dry-run`、元数据选项、`scaffold` 包管理和 CLI 单独编译，都是按需动作；无需日常反复操作。以下是原运行配方、参考角色包与维护入口；完整帮助用 `cargo run -p oclive-cli -- --help`。

### 交互式

```bash
cargo run -p oclive-cli -- init
```

### 非交互（CI / 脚本）

```bash
cargo run -p oclive-cli -- init --non-interactive --preset minimal -o /tmp/my-kernel
```

预设：`minimal` | `full` | `mixed`。`init --help` 末尾有 **预设与 `plugin_backends` 矩阵**；生成项目根目录含 **`CONFIG_REFERENCE.md`**。

**Monolith**：`--non-interactive` 下可加 **`--preset full --monolith`**（仅 **kernel_server**，须全局 `--experimental`），生成 **`monolith.toml`**、`vendor/oclive_monolith_builtin/`、`process_message_monolith.rs` 与双 **`[[bin]]`**（`main.rs` / `main_monolith.rs`）。修改 `monolith.toml` 后：`cargo run -p oclive-cli -- --experimental build -o <项目根>`（默认双构建）；`cargo run -p oclive-cli -- --experimental bench --release -o <项目根>` 输出 JSON 报告（Schema：`schemas/oclive_bench_report.schema.json`）。

**快速上手**：`cargo run -p oclive-cli -- doctor` → `cargo run -p oclive-cli -- init --quick -o ./my-chat`（见 [KERNEL_FACTORY_VISION.md](../../../creator-docs/getting-started/KERNEL_FACTORY_VISION.md)）。

**`doctor`** / **`bench`**（报告 schema v2：延迟 + 二进制大小 + 峰值内存 + 编译时间）/ **`--list-templates`** / **`--quick`**：见 [OCLIVE_CLI_GUIDE.md](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md) 与 [SETTINGS_REFERENCE.md](../../../creator-docs/cli/SETTINGS_REFERENCE.md)。

**Shell 补全**：`cargo run -p oclive-cli -- completions bash`（亦支持 `zsh`、`fish`、`powershell`）；安装说明见 CLI 指南「巩固强化」节。

**最小角色本地准备**：`cargo run -p oclive-cli -- pack validate-minimal-local <资产根目录> <相对定义文件>` 复用共享逻辑校验与有界资产读取。默认限制为定义文件 64 KiB、单资产 4 MiB、资产总量 16 MiB；可用 `--max-definition-bytes`、`--max-asset-bytes`、`--max-total-asset-bytes` 按宿主策略覆盖。文件名由调用方指定；成功不代表媒体解码或角色激活，现有 `pack validate/create` 仍服务参考宿主组合格式。语义边界见 [ROLE_PACK_BOUNDARY §0](../../../handoff/ROLE_PACK_BOUNDARY.md#0-三层-contract不得混称)。

**领域感知 CI（Stage 1 影子模式）**：

```bash
cargo run -p oclive-cli -- ci plan --shadow --base HEAD^ --head HEAD
cargo run -p oclive-cli -- ci explain --format markdown
```

`plan` 只读取 `data/ci/` 的中央影响图、模块描述与受信验证目录，输出 `target/oclive-ci/plan.json`；`explain` 从该 JSON 渲染摘要，不执行验证器。当前影子计划不得用于跳过现有 CI job。设计边界见 [OCLive 领域感知 CI](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)。
