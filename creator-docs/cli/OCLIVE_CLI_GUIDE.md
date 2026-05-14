# oclive-cli 使用指南

**oclive-cli** 是 oclive 官方 **内核 / 无头项目** 脚手架：在终端中交互（或脚本化）生成**可独立 `cargo build`** 的最小工程，便于硬件、侧车与多发行形态复用同一套配置形状。

**源码**：[`crates/oclive-cli/`](../../crates/oclive-cli/)  
**契约参考**（正式宿主）：[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)

---

## 安装与帮助

在 **oclivenewnew 仓库根目录**：

```bash
cargo build -p oclive-cli
cargo run -p oclive-cli -- --help
cargo run -p oclive-cli -- init --help
```

---

## `init`：创建项目

### 交互式（默认）

```bash
cargo run -p oclive-cli -- init -o ./out/my-kernel
```

流程包括：项目名、类型（无头可执行 / 库）、后端槽位多选、`builtin`/`stub`/`none` 选择、可选插件开关、是否生成示例 `roles/default`。

### 非交互 + 预设

| 预设 | 说明 |
|------|------|
| `minimal` | 多为 `stub` / `none`，不生成示例角色包目录 |
| `full` | 全 `builtin`，插件说明全开，带示例角色包 |
| `mixed` | 混合，用于 CI 与文档示例 |

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal -o /tmp/my-kernel
```

库类型：

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --preset mixed --project-type library -o /tmp/my-lib
```

### 常用参数

| 参数 | 含义 |
|------|------|
| `-o` / `--output` | 输出目录（须为空或不存在；将创建） |
| `--non-interactive` | 使用 `--preset`，不弹出 dialoguer |
| `--quiet` | 不打印配置摘要与完成提示（适合脚本） |
| `--preset` | `minimal` \| `full` \| `mixed` |
| `--project-type` | `kernel-server` \| `library` |
| `--project-name` | 默认 `my_oclive_kernel` |

---

## 生成物说明

- **占位 `Cargo.toml`**：当前仅依赖 **`serde` / `serde_json`**，不假设本机已存在 `oclive_kernel_runtime` 拆分 crate。接入真实内核时，请改为 `path` / 版本依赖并替换 `main.rs` / `lib.rs` 入口。
- **`settings.json`**：`plugin_backends` 六槽 + `_oclive_cli.complex_emotion` 元数据；与桌面端完全对齐前，请视为 **脚手架形状**，按 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) 校正枚举值。
- **README（生成）**：根据插件勾选，写入接入 `oclive_kernel_server`、OOCP、目录插件的**文字指引**。

---

## 与 CI 的关系

仓库 **`.github/workflows/ci.yml`** 的 **`cli`** job 会 `cargo test -p oclive-cli`（含端到端：生成临时目录并 `cargo build`）。

---

## 后续路线（建议）

1. 在 workspace 中落地 **`oclive_kernel_runtime`** 后，为 CLI 增加 **`--kernel-source path`**，自动写入 `Cargo.toml` 依赖。  
2. 为 `stub` / `none` 生成 **cfg(feature)** 开关，与 `MODULE_NONE_SEMANTICS` 文档对齐。  
3. 提供 **`oclive-cli check`**：对生成目录跑 `cargo clippy` + 可选 `cargo audit`。
