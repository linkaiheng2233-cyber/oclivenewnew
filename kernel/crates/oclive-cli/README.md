# oclive-cli

官方脚手架：在终端里交互（或非交互）生成**可独立 `cargo build`** 的最小内核项目骨架。

> **说明**：当前主仓工作区尚未拆分 `oclive_kernel_runtime` / `oclive_kernel_server` 等独立 crate；生成项目使用 **std + serde** 的占位运行时，便于硬件/无头场景先打通目录与配置形状。接入真实内核时，把生成 `Cargo.toml` 中的占位依赖换成你的 crate 路径即可。

## 安装 / 运行

```bash
# 在 oclivenewnew 仓库根目录
cargo build -p oclive-cli
cargo run -p oclive-cli -- --help
cargo run -p oclive-cli -- init --help
```

## 用法

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

**快速上手**：`cargo run -p oclive-cli -- doctor` → `cargo run -p oclive-cli -- init --quick -o ./my-chat`（见 [KERNEL_FACTORY_VISION.md](../../creator-docs/getting-started/KERNEL_FACTORY_VISION.md)）。

**`doctor`** / **`bench`**（报告 schema v2：延迟 + 二进制大小 + 峰值内存 + 编译时间）/ **`--list-templates`** / **`--quick`**：见 [OCLIVE_CLI_GUIDE.md](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md) 与 [SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md)。

**Shell 补全**：`cargo run -p oclive-cli -- completions bash`（亦支持 `zsh`、`fish`、`powershell`）；安装说明见 CLI 指南「巩固强化」节。

**领域感知 CI（Stage 1 影子模式）**：

```bash
cargo run -p oclive-cli -- ci plan --shadow --base HEAD^ --head HEAD
cargo run -p oclive-cli -- ci explain --format markdown
```

`plan` 只读取 `data/ci/` 的中央影响图、模块描述与受信验证目录，输出 `target/oclive-ci/plan.json`；`explain` 从该 JSON 渲染摘要，不执行验证器。当前影子计划不得用于跳过现有 CI job。设计边界见 [OCLive 领域感知 CI](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)。
