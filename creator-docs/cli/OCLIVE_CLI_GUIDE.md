# oclive-cli 使用指南

**oclive-cli** 是 oclive 官方 **内核 / 无头项目** 脚手架：在终端中交互（或脚本化）生成**可独立 `cargo build`** 的最小工程，便于硬件、侧车与多发行形态复用同一套配置形状。

**源码**：[`crates/oclive-cli/`](../../crates/oclive-cli/)  
**契约参考**（正式宿主）：[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)  
**`plugin_backends` 字段级权威说明**：[SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md)

---

## 安装与帮助

在 **oclivenewnew 仓库根目录**：

```bash
cargo build -p oclive-cli
cargo run -p oclive-cli -- --help
cargo run -p oclive-cli -- init --help
```

`init --help` **末尾**附有 **预设与 `plugin_backends` 矩阵**（与生成项目根目录 **`CONFIG_REFERENCE.md`** 一致）。

---

## `init`：创建项目

### 交互式（默认）

```bash
cargo run -p oclive-cli -- init -o ./out/my-kernel
```

流程包括：项目名、类型（无头可执行 / 库）、后端槽位多选、`builtin` / `remote` / `directory` / `none`（`llm` 槽另有 **`ollama`**）选择、可选插件开关、是否生成示例 `roles/default`；**无头服务（kernel_server）** 末尾另有 **开发者编译选项**（默认关闭）。

### 非交互 + 预设

| 预设 | 说明 |
|------|------|
| `minimal` | 六槽全 `builtin` 语义；`llm` 写 **`ollama`**；`agent` **省略 JSON 键**；`complex_emotion` 为 `none`；插件占位关 |
| `mixed` | 与矩阵一致：`llm=ollama`，`agent`/`complex_emotion` 为 `builtin`；部分插件说明开启 |
| `full` | `llm=remote`，`complex_emotion=remote`，其余槽 `builtin`；插件说明全开 |

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal -o /tmp/my-kernel
```

启用 Monolith（**实验性**：`--non-interactive` 下加 **`--monolith`**；仅 **kernel_server**）：

```bash
cargo run -p oclive-cli -- init --non-interactive --preset full --monolith -o /tmp/my-monolith-kernel
cargo build --release --manifest-path /tmp/my-monolith-kernel/Cargo.toml
cargo build --release --features monolith --manifest-path /tmp/my-monolith-kernel/Cargo.toml
```

矩阵全文见 **`init --help`** 或 [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) 中「`oclive-cli` 预设矩阵」一节。

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
| `--monolith` | 非交互：**实验性** Monolith；生成 `monolith.toml`、双 `[[bin]]` 与 `process_message_monolith.rs`（**仅 kernel_server**；与 `--project-type library` 互斥时自动忽略） |

非交互时 **不必** 传入任何 `--backend-*` 即可生成；传入则只覆盖所列槽位。

---

## 生成物说明

- **占位 `Cargo.toml`**：当前仅依赖 **`serde` / `serde_json`**，不假设本机已存在 `oclive_kernel_runtime` 拆分 crate。接入真实内核时，请改为 `path` / 版本依赖并替换 `main.rs` / `lib.rs` 入口。
- **`roles/default/settings.json`**：含 **`_comment_*`** 与完整 **`plugin_backends`**（含第 7 键 `complex_emotion`）；与主应用完全对齐时请以 [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) 为准裁剪非法键（如主应用不接受的 `none` 字符串）。
- **`CONFIG_REFERENCE.md`（项目根）**：预设矩阵与各槽一句话；含 **开发者编译选项（实验性）** 与 Monolith RFC 链接。
- **`init --help` 末尾**：含预设矩阵、**`--monolith`** 说明，指向 [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)。
- **README（生成）**：根据插件勾选，写入接入 `oclive_kernel_server`、OOCP、目录插件的**文字指引**。

---

## 高耦合编译模式（Monolith，实验性，第一阶段）

**适用**：无头 **`kernel_server`** 占位工程；需要对比 **标准** 与 **`-monolith`** Release 二进制的开发者。**不适用**：嵌入式 **library**（`--monolith` 会被忽略）。

**行为**：生成 **`monolith.toml`**（`enabled = true`，`weld_modules = []` 表示七槽占位焊接）、**`src/process_message_monolith.rs`**（同 crate 内 `welded_*` 静态桩，可编译；接入真实内核后替换为 `oclive_*_builtin` 等）、**`Cargo.toml`** 中 **`[features] monolith`** 与第二 **`[[bin]]`**（`{package}-monolith`，`required-features = ["monolith"]`）。**`main.rs`** 在 `feature = "monolith"` 时调用 `process_message_monolith::run_monolith_pipeline_demo()`。

**风险**：双 `[[bin]]` 共享 `src/main.rs` 时 Cargo 会提示「同一文件对应多个 bin」警告，可接受；占位焊接 **无** 真实 `PluginHost` 行为。

权威设计：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)。

---

## 与 CI 的关系

仓库 **`.github/workflows/ci.yml`** 的 **`cli`** job 会 `cargo test -p oclive-cli`（含端到端：生成临时目录并 `cargo build`）。

---

## 后续路线（建议）

1. 在 workspace 中落地 **`oclive_kernel_runtime`** 后，为 CLI 增加 **`--kernel-source path`**，自动写入 `Cargo.toml` 依赖。  
2. 与 `MODULE_NONE_SEMANTICS` 对齐时，为「逻辑 none」与「可加载 JSON」生成 **自动校验** 或 `cargo oclive-validate-settings` 子命令。
