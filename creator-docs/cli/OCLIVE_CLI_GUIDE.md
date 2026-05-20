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

**5 分钟上手**（`doctor` → `init --quick` → `cargo run`）：[KERNEL_FACTORY_VISION.md](../getting-started/KERNEL_FACTORY_VISION.md#5-分钟从零到对话纯内核脚手架)。

---

## 平台能力速览（N–S）

| 命令 | 作用 |
|------|------|
| `registry` | 本地工程注册表 `~/.oclive/registry.json`（`init` 后自动注册） |
| `compose` | 多内核 `oclive-compose.yml` 编排 `up` / `down` / `ps` |
| `publish` | 打包 `.oclive-template.tar.gz`；`init --template-url` 远程初始化 |
| `init --tui` | ratatui 模板选择器（TTY 不可用时回退） |
| `bench --watch` | 源码变更触发自动 bench + 历史对比 |
| `debug` | `OCLIVE_DEBUG_TRACE` 逐步骤追踪 |

---

## U 维：可视化与上手

| 命令 | 作用 |
|------|------|
| `dashboard` | 本地 Web 仪表盘（默认 `http://127.0.0.1:8420`）：工程列表、模板库、`bench_history` 折线 |
| `bench --dashboard` | 终端实时刷新标准/焊接 p50·P95·内存·体积；底部 ASCII sparkline；`q` 退出 |
| `learn` | 五步交互教程（`doctor` → 模板说明 → `init` → `cargo build` → curl 提示） |

```bash
cargo run -p oclive-cli -- dashboard
cargo run -p oclive-cli -- bench --dashboard --release -o ./my-kernel
cargo run -p oclive-cli -- learn -o ./oclive-learn-demo
```

> **端口**：`dashboard` 与内核 HTTP API 默认均可能占用 **8420**；请勿同时启动。教程 HTTP 示例使用 **8421**。

---

## V 维：质量与矩阵基准

| 命令 | 作用 |
|------|------|
| `bench --matrix` | Monolith 档位（none/latency/memory/embedded）× preset（minimal/mixed/full）矩阵；各 3 轮 |
| `test` | `cargo check`、clippy、角色包 `pack validate`、OOCP 路径提示（`--skip-oocp`） |
| `lint` | 目录结构、`Cargo.toml` 元数据、`settings.json` 七槽、`monolith.toml`、Git 脏检查 |

```bash
cargo run -p oclive-cli -- bench --matrix --release -o ./my-kernel
cargo run -p oclive-cli -- bench --matrix --json -o ./my-kernel
cargo run -p oclive-cli -- test -o ./my-kernel
cargo run -p oclive-cli -- lint -o ./my-kernel --json
```

---

## W 维：插件生态 CLI

| 子命令 | 作用 |
|--------|------|
| `plugin install <id>` | 解析 `manifest.json` 的 `plugin_dependencies`，拓扑排序安装 |
| `plugin uninstall <id>` | 卸载并提示被依赖关系 |
| `plugin test <path>` | 子进程 + RPC 烟测（`health` / 槽位方法） |
| `plugin search <kw>` | 从 `OCLIVE_PLUGIN_INDEX_URL`（默认仓库 `examples/plugin-index.json` 的 raw）搜索 |
| `plugin update <id>` | 检查索引版本并更新 |

`plugin_dependencies` 为 manifest 可选字符串数组；环依赖报错。见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)。

---

## X 维：焊接与编排定制

| 能力 | 说明 |
|------|------|
| `init --tui` + Monolith | 模板确认后进入**自定义焊接**页：空格勾选七槽，生成 `monolith.toml` |
| `init --weld-modules` | 非 TUI：`memory,emotion,prompt,llm` 等逗号列表 |
| `init --pipeline` | `default` \| `emotion-first` \| `memory-last`；生成 `docs/PIPELINE_CUSTOM.md` 与 `src/oclive_pipeline_order.rs` |
| `profile` | `cargo tree`、release 二进制体积、可选 `cargo bloat` Top crate |

```bash
cargo run -p oclive-cli -- init --pipeline memory-last --template dialogue-only -o ./p -n demo --non-interactive
cargo run -p oclive-cli -- profile -o ./my-kernel --json
```

---

## `doctor`：环境诊断

```bash
cargo run -p oclive-cli -- doctor
cargo run -p oclive-cli -- doctor --json
cargo run -p oclive-cli -- doctor -o ./my-project
```

检查 Rust/Cargo、系统内存、磁盘剩余、Ollama（`http://127.0.0.1:11434/api/tags`）、GitHub 连通、工作区可写。存在 **fail** 项时退出码非 0。JSON Schema：`crates/oclive-cli/schemas/oclive_doctor_report.schema.json`。

---

## `pack`：角色包校验与打包

在仓库根目录：

```bash
cargo run -p oclive-cli -- pack validate ./roles/mumu --host-version 0.2.0
cargo run -p oclive-cli -- pack validate ./roles/mumu --host-version 0.2.0 --profile robot-soul
cargo run -p oclive-cli -- pack create -o ./out/my-role --flat --id com.example.demo --name Demo
cargo run -p oclive-cli -- pack publish ./out/my-role -o ./dist/com.example.demo-0.1.0.oclivepack
```

- **`validate`**：校验 `manifest.json` / `settings.json` 合并、`plugin_backends` 反序列化、`default_personality` 七维范围、`interaction_mode`、`min_runtime_version` 与 `--host-version` 等（与宿主磁盘加载阶段对齐，不跑 DB）。
- **`create`**：生成最小可校验目录；`--flat` 时 `-o` 指向的目录即为角色根（否则创建 `roles/<id>/`）。
- **`publish`**：将角色目录打成 **ZIP**，扩展名 **`.oclivepack`**；ZIP 内顶层文件夹名为 **`manifest.id`**。

**JSON Schema**（IDE / `ajv` 等）：`crates/oclive-cli/schemas/role_pack_manifest.schema.json`、`role_pack_settings.schema.json`、`role_pack_index.schema.json`。

---

## `plugin create`：插件脚手架

一键生成**目录插件**（Node `rpc_server.mjs` + 子进程）或 **Remote HTTP 插件**（Python `rpc_server.py`）骨架，含 `manifest.json`（`id` / `provides` / `permissions` / `rpcMethods`）、README 与 RPC 方法桩。

```bash
# 非交互：目录插件，提供 llm 槽
cargo run -p oclive-cli -- plugin create my-llm-plugin --type directory --provides llm -o ./plugins/

# Remote 侧车，多槽
cargo run -p oclive-cli -- plugin create my-remote --type remote --provides memory --provides emotion -o ./out/plugin --non-interactive

# 交互：选择类型、槽位、输出目录
cargo run -p oclive-cli -- plugin create my-plugin
```

**`--provides`**：`llm` | `memory` | `emotion` | `event` | `prompt` | `agent` | `complex_emotion`（可重复）。输出目录默认为 `./plugins/`；最终包路径为 `<output>/<plugin_id>/`（`id` 由名称 slug 为 `com.oclive.plugin.<name>`）。

生成 manifest 经 **`oclive_validation`** 权限校验（目录插件）。快速上手见 [PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)。

---

## `dev`：角色包目录监听

在**已存在**的内核 / 脚手架项目根（含 `Cargo.toml`）执行。使用 **notify 递归模式**监听 **`roles/**/manifest.json`** 与 **`roles/**/settings.json`**（任意子目录角色包）；**500ms 防抖**后打印：

`[oclive dev] 检测到角色包 '<id>' 变更，已重载`

**`--reload-cmd`** 可在变更后执行一条 shell 命令（如通知侧车重载）。

```bash
cargo run -p oclive-cli -- dev -o /path/to/project
cargo run -p oclive-cli -- dev -o /path/to/project --roles roles --reload-cmd "echo reload"
cargo run -p oclive-cli -- dev -o /path/to/project --no-watch
```

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

### 内核工厂模板（`--template`）

套餐封装 **preset / Monolith / project-type**（显式 CLI 参数优先）。浏览矩阵：`oclive init --list-templates`。愿景：[KERNEL_FACTORY_VISION.md](../getting-started/KERNEL_FACTORY_VISION.md)

| template | 场景 | preset | Monolith 默认 | project-type | 默认 `--with-role-pack` |
|----------|------|--------|---------------|--------------|-------------------------|
| `robot-soul` | 玩偶 / 嵌入式 | minimal | 启用 | kernel_server | `robot-soul-minimal` |
| `robot-gateway` | 智能网关 / 家庭中枢 | mixed | 启用 | kernel_server | `gateway` + `mcp_servers/` |
| `dialogue-only` | 纯对话服务 | full | 关闭 | kernel_server | `default` |
| `headless-api` | 纯 API 无头 | full | 关闭 | kernel_server | 无 |
| `library-embed` | 库嵌入 | minimal | 关闭 | library | 无 |

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --template robot-soul -o ./out/doll
cargo run -p oclive-cli -- init --non-interactive --template robot-gateway -o ./out/gateway
cargo run -p oclive-cli -- init --non-interactive --template dialogue-only -o ./out/chat
cargo run -p oclive-cli -- init --non-interactive --template headless-api --monolith -o ./out/api-weld
cargo run -p oclive-cli -- init --non-interactive --template library-embed --kernel-source . -o ./out/embed
```

**`--with-role-pack`**：`robot-soul-minimal`（七维 + `prompts/system.md`）| `default`（通用 `roles/default`）。未指定且未用模板时，非交互仍生成 **default** 示例包（与历史一致）。

**`--with-example-plugin`**：复制 `com.oclive.example.llamacpp_llm/` 到 `plugins/`（源自主仓 `examples/directory-plugin-llamacpp/`；默认关闭）。

生成工程含 **`plugins/README.md`**、**`docs/BLUEPRINT_REFERENCE.md`**、**`docs/ORCHESTRATION_REFERENCE.md`**（中英编排参考）。

**蓝图校验**（不改变桌面宿主主路径）：

```bash
cargo run -p oclive-cli -- blueprint validate ./roles/myrole/pipeline.ocblueprint
cargo run -p oclive-cli -- blueprint validate ./path.json --json
```

启用 Monolith（`--non-interactive` 下加 **`--monolith`**；仅 **kernel_server**）：

```bash
cargo run -p oclive-cli -- init --non-interactive --preset full --monolith --monolith-preset latency -o /tmp/my-monolith-kernel
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
| `--monolith-preset` | 仅 Monolith 启用时：`latency`（七槽全焊）\| `memory` \| `embedded`；预填 `monolith.toml` 的 `weld_modules` |
| `--monolith-bench-preset` | 同档位枚举；生成后自动 release 双构建 + `bench --runs 5` → `bench_results/report.json`（失败不阻塞） |
| `--list-templates` | 打印模板矩阵后退出；交互 `init` 亦可在项目类型前选模板 |
| `--quick` / `-q` | 极速：`preset=full`、无 Monolith、无 `roles/`；交互仅问项目名与输出目录 |
| `--template` | `robot-soul` \| `robot-gateway` \| `dialogue-only` \| `headless-api` \| `library-embed`（内核工厂套餐；见上表） |
| `--with-role-pack` | `robot-soul-minimal` \| `default`；与 `--skip-role-pack` 互斥 |
| `--with-example-plugin` | 附带 llamacpp 目录插件示例到 `plugins/` |
| `--kernel-source` | 指向 oclivenewnew 根目录，生成 path 依赖与真实 HTTP 入口 |
| `--author` | 写入生成 `Cargo.toml` 的 `[package].authors` |
| `--license` | SPDX 许可证（默认 **MIT**） |
| `--description` | 写入 `[package].description`（留空则不写） |
| `--template-url` | 从 URL 下载 `.oclive-template.tar.gz` 并解压到 `-o` |
| `--tui` | 使用 ratatui 可视化选模板（TTY 不可用时回退） |

非交互时 **不必** 传入任何 `--backend-*` 即可生成；传入则只覆盖所列槽位。交互模式在输入项目名后会询问作者（默认 `git config user.name`）、许可证与简短描述。

---

## `registry`：本地工程注册表

数据文件：**`~/.oclive/registry.json`**（可用 **`OCLIVE_HOME`** 覆盖根目录）。**`oclive init` 成功后会自动注册**。

```bash
cargo run -p oclive-cli -- registry list
cargo run -p oclive-cli -- registry list --json
cargo run -p oclive-cli -- registry add my-kernel ./path/to/project --template robot-soul
cargo run -p oclive-cli -- registry remove my-kernel
cargo run -p oclive-cli -- registry switch my-kernel
```

---

## `compose`：多内核编排

```bash
cargo run -p oclive-cli -- compose init
cargo run -p oclive-cli -- compose up
cargo run -p oclive-cli -- compose down
cargo run -p oclive-cli -- compose ps
```

见项目根 **`oclive-compose.yml`**（`services.<id>.path` / `port` / `env` / `depends_on`）。**`up`** 按依赖顺序启动，日志带 **`[服务名]`** 前缀；状态写入 **`.oclive-compose.pids.json`**。

---

## `publish`：模板包

```bash
cargo run -p oclive-cli -- publish --type template -o ./my-kernel.oclive-template.tar.gz
cargo run -p oclive-cli -- init --template-url https://example.com/template.tar.gz -o ./from-remote
```

排除 **`target/`**、**`.git/`**、**`bench_results/`** 等；包内附带 **`template.json`** 元数据。

---

## `debug`：逐步骤追踪

```bash
cargo run -p oclive-cli -- debug -o ./my-kernel --kernel-source  # 工程须已接主仓内核
cargo run -p oclive-cli -- debug -o . --step build_prompt --json
```

设置 **`OCLIVE_DEBUG_TRACE=1`** 并启动 **`--api`**（默认 Mock LLM）。详见生成工程 **`docs/DEBUG_REFERENCE.md`**。

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

**常见编译错误与修复**：`cargo build` 失败时，CLI 会解析 stderr 并给出中文建议（crate 未找到、缺少 C 链接器、Rust 版本过低、OpenSSL 开发库、内存不足等）。未匹配时保留原始输出并提示运行 **`oclive doctor`**。

### `bench` 子命令

再生成源码、双构建后，对两个二进制各跑 `--runs` 次子进程；子进程内通过环境变量 **`OCLIVE_KERNEL_BENCH_ITERS`** 做热循环。输出 **JSON**（`schema_version: 2`），除延迟分位数外含 **`binary_size`**（字节）、**`peak_memory`**（MiB）、**`build_time`**（秒）。Schema：`crates/oclive-cli/schemas/oclive_bench_report.schema.json`。

```bash
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --json
```

- **`--save`**：将本次报告追加到项目根 **`bench_history.json`**（本地文件，勿提交）。
- **`--compare`**：不运行采样；读取 **`bench_history.json`** 中**最近两次**记录并输出对比（需已有至少两条历史）。
- **`--history`**：打印 **`bench_history.json`** 全部记录的趋势表（日期、标准/Monolith p50、峰值内存、二进制体积）；记录 ≥2 条时附相对上一行的 **↑/↓/→**。可加 **`--json`** 供外部工具消费。

```bash
cargo run -p oclive-cli -- bench --release -o ./my-kernel --save
cargo run -p oclive-cli -- bench --history -o ./my-kernel
cargo run -p oclive-cli -- bench --history -o ./my-kernel --json
cargo run -p oclive-cli -- bench --watch -o ./my-kernel
```

- **`--watch`**：监听 **`src/`** 与 **`Cargo.toml`**（2s 防抖），自动 release 构建 + **3 轮** bench 并 **`--save`**，打印相对上一轮 **↑/↓/→**。

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

---

[English](../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md)
