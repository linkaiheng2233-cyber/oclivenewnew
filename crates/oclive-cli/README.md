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

**Monolith（实验性）**：`--non-interactive` 下可加 **`--preset full --monolith`**（仅 **kernel_server**），生成 **`monolith.toml`**、`process_message_monolith.rs` 与双 **`[[bin]]`**；`cargo build --release --features monolith` 产出带 **`-monolith`** 后缀的二进制。

完整说明见 [creator-docs/cli/OCLIVE_CLI_GUIDE.md](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md) 与 [creator-docs/cli/SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md)。
