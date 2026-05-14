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

**角色包规范与校验**：见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)；子命令 **`pack`** 见同文档第 6 节与下文。

---

## `pack`：角色包校验与打包

在仓库根目录：

```bash
cargo run -p oclive-cli -- pack validate ./roles/mumu --host-version 0.2.0
cargo run -p oclive-cli -- pack create -o ./out/my-role --flat --id com.example.demo --name Demo
cargo run -p oclive-cli -- pack publish ./out/my-role -o ./dist/com.example.demo-0.1.0.oclivepack
```

- **`validate`**：校验 `manifest.json` / `settings.json` 合并、`plugin_backends` 反序列化、`default_personality` 七维范围、`interaction_mode`、`min_runtime_version` 与 `--host-version` 等（与宿主磁盘加载阶段对齐，不跑 DB）。
- **`create`**：生成最小可校验目录；`--flat` 时 `-o` 指向的目录即为角色根（否则创建 `roles/<id>/`）。
- **`publish`**：将角色目录打成 **ZIP**，扩展名 **`.oclivepack`**；ZIP 内顶层文件夹名为 **`manifest.id`**。

**JSON Schema**（IDE / `ajv` 等）：`crates/oclive-cli/schemas/role_pack_manifest.schema.json`、`role_pack_settings.schema.json`、`role_pack_index.schema.json`。

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
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal --skip-role-pack -o /tmp/my-kernel-no-roles
```

`--skip-role-pack`：不生成 `roles/`（空白内核工程）。

启用 Monolith（`--non-interactive` 下加 **`--monolith`**；仅 **kernel_server**）：

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
| `--monolith` | 非交互：启用 Monolith；生成 `monolith.toml`、`vendor/oclive_monolith_builtin/`、双 `[[bin]]`（`main.rs` / `main_monolith.rs`）与 `process_message_monolith.rs`（**仅 kernel_server**；与 `--project-type library` 互斥时自动忽略） |

非交互时 **不必** 传入任何 `--backend-*` 即可生成；传入则只覆盖所列槽位。

---

## 生成物说明

- **占位 `Cargo.toml`**：当前仅依赖 **`serde` / `serde_json`**，不假设本机已存在 `oclive_kernel_runtime` 拆分 crate。接入真实内核时，请改为 `path` / 版本依赖并替换 `main.rs` / `lib.rs` 入口。
- **`roles/default/settings.json`**：含 **`_comment_*`** 与完整 **`plugin_backends`**（含第 7 键 `complex_emotion`）；与主应用完全对齐时请以 [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) 为准裁剪非法键（如主应用不接受的 `none` 字符串）。
- **`CONFIG_REFERENCE.md`（项目根）**：预设矩阵与各槽一句话；含 **开发者编译选项（Monolith）** 与 RFC 链接。
- **`init --help` 末尾**：含预设矩阵、**`--monolith`** 说明，指向 [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)。
- **README（生成）**：根据插件勾选，写入接入 `oclive_kernel_server`、OOCP、目录插件的**文字指引**。

---

## 高耦合编译模式（Monolith）

**适用**：无头 **`kernel_server`** 占位工程；需要对比 **标准** 与 **`-monolith`** 二进制的开发者。**不适用**：嵌入式 **library**（`--monolith` 会被忽略）。

**行为**：`init --monolith` 生成 **`monolith.toml`**、`vendor/oclive_monolith_builtin/`、**`src/process_message_monolith.rs`**（已焊接槽静态调用 vendor crate；未焊接槽为 trait/PluginHost 占位）、**`Cargo.toml`** 中 **`[features] monolith`** 与第二 **`[[bin]]`**（**`src/main.rs`** 为标准入口，**`src/main_monolith.rs`** 为 Monolith 入口，避免双 bin 同路径警告）。

### `build` 子命令

在**已存在**的 Monolith 项目根执行（须含 `monolith.toml`）：

```bash
cargo run -p oclive-cli -- build -o /path/to/kernel-project
cargo run -p oclive-cli -- build -o /path/to/kernel-project --release --features somefeat
cargo run -p oclive-cli -- build -o /path/to/kernel-project --no-cargo
```

- **`--no-cargo`**：仅再生成 `process_message_monolith.rs` 与 vendor，不调用 `cargo`。
- **`--release`** / **`--features`**：传给每次 `cargo build`；Monolith 次构建会自动并入 **`monolith`** feature。
- **`--` 之后**：附加参数透传给 `cargo build`。

### `bench` 子命令

再生成源码、双构建后，对两个二进制各跑 `--runs` 次子进程；子进程内通过环境变量 **`OCLIVE_KERNEL_BENCH_ITERS`** 做热循环。输出 **JSON**（`schema_version: 1`），Schema 见仓库 **`crates/oclive-cli/schemas/oclive_bench_report.schema.json`**。

```bash
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --json
```

`--json`：仅将报告 JSON 打印到 **stdout**（进度走 **stderr**），便于管道与 Schema 校验。

**风险**：占位工程 **无** 真实 `PluginHost` 行为。

权威设计：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)。

---

## 与 CI 的关系

仓库 **`.github/workflows/ci.yml`** 的 **`cli`** job 会 `cargo test -p oclive-cli`（含端到端：`init`、`build`、`bench` smoke）。另有轻量 **`cli-bench`** job 跑一轮 `bench`（不设性能阈值）。

---

## 后续路线（建议）

1. 在 workspace 中落地 **`oclive_kernel_runtime`** 后，为 CLI 增加 **`--kernel-source path`**，自动写入 `Cargo.toml` 依赖。  
2. 与 `MODULE_NONE_SEMANTICS` 对齐时，为「逻辑 none」与「可加载 JSON」生成 **自动校验** 或 `cargo oclive-validate-settings` 子命令。
