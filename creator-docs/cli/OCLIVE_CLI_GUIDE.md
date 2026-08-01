# oclive-cli 使用指南

**oclive-cli** 是 oclive 官方 **内核 / 无头项目** 脚手架：在终端中交互（或脚本化）生成**可独立 `cargo build`** 的最小工程，便于硬件、侧车与多发行形态复用同一套配置形状。

**源码**：[`kernel/crates/oclive-cli/`](../../kernel/crates/oclive-cli/)  
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

**与实现对齐**：顶层子命令以 `kernel/crates/oclive-cli/src/main.rs` 的 `Commands` 枚举为准。默认帮助只展示稳定入口；试验命令仍可通过已知名称调用，但必须显式传全局 `--experimental`。旧工程归档 `template` 保持兼容可调用但默认隐藏。

---

## 官方命令面

### 默认可见的稳定入口

| 域 | 命令 | 说明 |
|----|------|------|
| 项目初始化 | `init` | 配方、`--preset`、`--smart`（环境推荐）、`--with-role-pack`、`--kernel-source`、`--quick` 等；`--monolith` 仍属试验能力 |
| 角色包 | `pack create` / `validate` / `publish` | 创建、校验、`.oclivepack` 打包 |
| 插件 | `plugin create` / `install` / `uninstall` / `test` / `manage` | 脚手架与工程内安装；**manage** 为高级槽位/蓝图 |
| 环境 | `doctor`（`--fix`）、`config` | 诊断与 `~/.oclive/config.toml` |
| 质量与 CI | `lint`、`ci init/check/plan/explain` | 静态检查、CI 模板与领域感知影子规划；第三方脚手架不能扩展 `ci` |
| 脚手架工具 | `scaffold list/inspect/validate/resolve/generate` | 本地声明发现、信任提示、确定性解析与锁后声明式生成；不执行第三方命令 |
| 本地开发 | `dev`、`registry`、`profile` | 角色监听、工程注册表、依赖/体积画像 |
| 契约与迁移 | `kernel`、`explain`、`migrate-app-data`、`completions` | 运行时信息、错误解释、数据迁移与补全 |

### 默认隐藏但保留调用兼容

- **试验命令**：`build`、`bench`、`blueprint`、`compose`、`debug`、`dashboard`、`learn`、`test`、`market`、`collab`；使用时在子命令前传全局 `--experimental`。
- **旧工程归档**：`template create/pack` 与 `init --template-url` 只表示 `.oclive-template.tar.gz` 兼容流，不是 Scaffold Package；顶层 `publish` 已移除，没有 deprecated 别名。

Scaffold Package 的发现、版本、命名空间、来源锁定和 CI 隔离以 [RFC_SCAFFOLD_PACKAGE_V1.md](../rfc/RFC_SCAFFOLD_PACKAGE_V1.md) 为准。

**计划中（未实现，勿写入「已完成」）**：`pack diff` / `pack update`、`oclive kernel update`、`dev --inject`、`bench history clear` / `export` / `import` — 见 [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md#oclive-cli-脚手架计划中)。

---

## 确定性加固（AB1–AB6）

| 编号 | 能力 | 说明 |
|------|------|------|
| **AB1** | `narrative_hint` 契约 | 集成测 + [NARRATIVE_HINT_CONTRACT.md](../testing/NARRATIVE_HINT_CONTRACT.md) |
| **AB2** | 侧车 / 内核错误分层 | `oclive_validation::protocol_boundary`；OOCP **S12** |
| **AB3** | `bench --equivalence` | 标准 vs Monolith `/chat` 回复逐条对比（MOCK_LLM） |
| **AB4** | `test --loom` | `cargo-loom` 模型检查（Nightly `loom` job；失败可见但不阻塞 main） |
| **AB5** | 模糊测试 | [FUZZING.md](../testing/FUZZING.md)；`kernel/fuzz/` + proptest |
| **AB6** | `bench --soak` | 加速冒烟或显式 `--soak-real-time` 真长稳；采样内核 PID 的 RSS/CPU、请求与回收状态 |

```bash
cargo test -p oclivenewnew-tauri --test narrative_hint_contract_audit
cargo test -p oclivenewnew-tauri --test protocol_boundary_sidecar
cargo run -p oclive-cli -- --experimental bench --equivalence --release -o ./my-kernel
cargo run -p oclive-cli -- --experimental test --loom
cargo test -p oclive_validation --test proptest_fuzz_parsing
cargo run -p oclive-cli -- --experimental bench --soak --soak-real-time --soak-duration 24 --soak-sample-interval 60 -o ./my-kernel
cargo run -p oclive-cli -- --experimental test --equivalence-check -o ./my-kernel
```

---

## 巩固强化（AA1–AA11）

| 编号 | 命令 | 说明 |
|------|------|------|
| **AA1** | `bench --cold-start` | 重启内核 `--api`，测首条 `/chat` 与热启动对比；`--cold-start-runs` / `--cold-start-warm-messages` |
| **AA2** | `test --coverage` | 需 `cargo-llvm-cov`；HTML → `target/llvm-cov/html/index.html`；`--open` 打开浏览器 |
| **AA3** | `test --miri` | 需 `cargo-miri`；`--miri-only <crate>` 限定包 |
| **AA4** | `explain <CODE>` | 解析 [ERROR_CODES.md](../getting-started/ERROR_CODES.md)（`<!-- code:... -->`）；`--json` |
| **AA5** | `init --dry-run` | 只打印目录树，不写盘；`--json` |
| **AA10** | `init --check` | 生成前预检（Rust、kernel-source、Monolith 链、角色包等）；失败 exit 1 |
| **AA7** | `lint --audit-ci` | 检查 CI 是否含 `cargo-audit` 及 `continue-on-error` |
| **AA8** | `doctor --sbom` | 需 `cargo-cyclonedx`；`--sbom-format spdx` |
| **AA9** | `completions <shell>` | `bash` / `zsh` / `fish` / `powershell`（或 `power-shell`） |

```bash
cargo run -p oclive-cli -- --experimental bench --cold-start --cold-start-runs 3 -o ./my-kernel
cargo run -p oclive-cli -- --experimental test --coverage -o .
cargo run -p oclive-cli -- --experimental test --miri -o .
cargo run -p oclive-cli -- explain LLM_ERROR
cargo run -p oclive-cli -- init --dry-run --template robot-soul -o ./preview
cargo run -p oclive-cli -- init --check --template robot-soul --kernel-source . -o ./out
cargo run -p oclive-cli -- lint --audit-ci
cargo run -p oclive-cli -- doctor --sbom -o .
cargo run -p oclive-cli -- completions bash > oclive.bash
```

**Shell 补全安装（bash 示例）**：`eval "$(cargo run -p oclive-cli -- completions bash)"` 或写入 `~/.bash_completion.d/oclive` 后 `source`。

补全由 **`clap_complete` 从当前 `Cli` 派生**；默认帮助与补全聚焦 **15** 个稳定顶层入口。隐藏的试验/兼容命令不计入默认认知面。已移除的顶层 **`publish`**、**`plugin search`/`update`**、**`registry login`** 不会重新出现；角色包分发请用 **`pack publish`**。

调优闭环见 [PERFORMANCE.md §5](../getting-started/PERFORMANCE.md#5-用-oclive-bench-做性能调优实战闭环)。

---

## 质量深耕（Z11–Z16 / Z14 / Z19）

| 代号 | 命令 | 说明 |
|------|------|------|
| **Z14** | `init --from-existing` / `--share` | 从现有工程反推完整 `oclive init` 复现命令；`--share` 写 `.oclive-share.toml` |
| **Z11** | `bench --stress` | 对运行中的内核 HTTP `/chat` 做并发压测（`--stress-concurrency` / `--stress-duration`） |
| **Z12** | `test --ci-parity` | 本地按 `.github/workflows/ci.yml` 顺序执行 fmt/clippy/build/test 等 |
| **Z13** | `lint --deps` | `cargo audit` + lockfile 中 yanked 包检查 |
| **Z13b** | `lint --deny` | `cargo deny check licenses` + `bans`（需 `cargo-deny` 与项目根 `deny.toml`） |
| **Z15** | `doctor --watch` | 每 60s 轮询环境；磁盘 &lt;1GiB、内存 &lt;500MiB、Ollama 停止时告警 |
| **Z16** | （全局） | **CLI 输出统一英文**（避免终端乱码） |
| **Z19** | `kernel info` | `oclive_kernel_runtime` path/version 与兼容性摘要 |

```bash
cargo run -p oclive-cli -- init --from-existing ./my-kernel --json
cargo run -p oclive-cli -- --experimental bench --stress --stress-concurrency 5 --stress-duration 10 -o ./my-kernel
cargo run -p oclive-cli -- --experimental test --ci-parity -o ./my-kernel --skip-oocp
cargo run -p oclive-cli -- lint --deps -o ./my-kernel
cargo run -p oclive-cli -- lint --deny -o .
cargo run -p oclive-cli -- --experimental test -o ./my-kernel --json
cargo run -p oclive-cli -- doctor --watch
cargo run -p oclive-cli -- kernel info -o ./my-kernel --json
```

---

## 平台能力速览（N–S）

| 命令 | 作用 |
|------|------|
| `registry` | 本地工程注册表 `~/.oclive/registry.json`（`init` 后自动注册） |
| `compose` | 多内核 `oclive-compose.yml` 编排 `up` / `down` / `ps` |
| `template pack` | 隐藏兼容入口：打包旧 `.oclive-template.tar.gz` 工程归档 |
| `init --tui` | ratatui 模板选择器（TTY 不可用时回退） |
| `bench --watch` | 源码变更触发自动 bench + 历史对比 |
| `debug` | `OCLIVE_DEBUG_TRACE` 逐步骤追踪 |

---

## U 维：可视化与上手

| 命令 | 作用 |
|------|------|
| `dashboard` | 本地 Web 仪表盘（默认 `http://127.0.0.1:8420`）：工程列表、模板库、`bench_history` 折线 |
| `bench --live` | 终端实时刷新标准/焊接 p50·P95·内存·体积（sparkline；`--dashboard` 为 deprecated 别名） |
| `learn` | 五步交互教程（`doctor` → 模板说明 → `init` → `cargo build` → curl 提示） |

```bash
cargo run -p oclive-cli -- --experimental dashboard
cargo run -p oclive-cli -- --experimental bench --live --release -o ./my-kernel
cargo run -p oclive-cli -- --experimental learn -o ./oclive-learn-demo
```

> **端口**：`dashboard` 与内核 HTTP API 默认均可能占用 **8420**；请勿同时启动。教程 HTTP 示例使用 **8421**。

---

## V 维：质量与矩阵基准

| 命令 | 作用 |
|------|------|
| `bench --matrix` | Monolith 档位（none/latency/memory/embedded）× preset（minimal/mixed/full）矩阵；各 3 轮 |
| `test` | `cargo check`、clippy、角色包 `pack validate`；**`--oocp`** 自动起内核并跑 OOCP **S0–S12（13 场景；可选 S13/S14）**（`--skip-oocp` 仅提示路径） |
| `lint` | 目录结构、`Cargo.toml` 元数据、`settings.json` 第 1–6 模块、`monolith.toml`、Git 脏检查 |

```bash
cargo run -p oclive-cli -- --experimental bench --matrix --release -o ./my-kernel
cargo run -p oclive-cli -- --experimental bench --matrix --json -o ./my-kernel
cargo run -p oclive-cli -- --experimental test -o ./my-kernel
cargo run -p oclive-cli -- lint -o ./my-kernel --json
```

---

## W 维：插件生态 CLI

| 子命令 | 作用 |
|--------|------|
| `plugin install <id>` | 解析 `manifest.json` 的 `plugin_dependencies`，拓扑排序安装 |
| `plugin uninstall <id>` | 卸载并提示被依赖关系 |
| `plugin test <path>` | 子进程 + RPC 烟测（`health` / 槽位方法） |
| `plugin search` | 扫描**已安装**目录插件（`manifest.json`）；`--provides <slot>` 按槽位过滤；在线发现用 `market search` |
| `plugin update` | **deprecated** — 请用 `market install` / 目录内 `git pull` |

`plugin_dependencies` 为 manifest 可选字符串数组；环依赖报错。见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)。

---

## X 维：焊接与编排定制

| 能力 | 说明 |
|------|------|
| `init --tui` + Monolith | 模板确认后进入**自定义焊接**页：空格勾选七焊接键（第 1–6 模块 + `complex_emotion`），生成 `monolith.toml` |
| `init --weld-modules` | 非 TUI：`memory,emotion,prompt,llm` 等逗号列表 |
| `init --pipeline` | `default` \| `emotion-first` \| `memory-last`；生成 `docs/PIPELINE_CUSTOM.md` 与 `src/oclive_pipeline_order.rs` |
| `profile` | `cargo tree`、release 二进制体积、可选 `cargo bloat` Top crate |

```bash
cargo run -p oclive-cli -- init --pipeline memory-last --template dialogue-only -o ./p -n demo --non-interactive
cargo run -p oclive-cli -- profile -o ./my-kernel --json
```

---

## T 维：协作与分发

### `market` — 插件 / 模板市场

| 子命令 | 作用 |
|--------|------|
| `market search <kw>` | 搜索插件、模板、角色包（复用 `OCLIVE_PLUGIN_INDEX_URL` / `OCLIVE_MARKET_INDEX_URL`） |
| `market browse` | TUI：左侧分类、右侧列表与详情；**Enter** 安装，**Esc** 退出 |
| `market install <id>` | 安装条目（插件→`distros/chat-pro/plugins/`；模板→`init`；角色包→`distros/chat-pro/roles/`） |
| `market info <id>` | 查看详情 |

离线缓存：`~/.oclive/plugin_index_cache.json`（在线拉取失败时自动回退）。默认索引与桌面一致：`awesome-oclive-plugins` 的 `plugins.json`；官方示例草稿见主仓 `data/plugins.json`（可用 `OCLIVE_PLUGIN_INDEX_URL` 指向其 raw URL）。

```bash
cargo run -p oclive-cli -- --experimental market browse
cargo run -p oclive-cli -- --experimental market search llm
cargo run -p oclive-cli -- --experimental market install template:dialogue-only --template-output ./from-market
```

### `registry` 云端同步

| 子命令 | 作用 |
|--------|------|
| `registry login` | **deprecated** — 内部写入 `config.toml`；请用 `oclive config set OCLIVE_REGISTRY_URL` / `OCLIVE_REGISTRY_TOKEN` |
| `registry logout` | 删除凭据 |
| `registry push <name>` | 将本地注册工程打包为 `.oclive-template.tar.gz` 并 `POST /api/v1/projects/{name}` |
| `registry pull <name>` | `GET .../archive` 解压并写入 `~/.oclive/registry.json` |
| `registry search <kw>` | `GET /api/v1/projects?q=` |

环境变量：**`OCLIVE_REGISTRY_URL`**、**`OCLIVE_REGISTRY_TOKEN`**（可覆盖 auth 文件）。协议：REST + Bearer；服务端需实现约定路径（见下文「云端注册表 API」）。

```bash
cargo run -p oclive-cli -- registry login https://registry.example.com your-token
cargo run -p oclive-cli -- registry push my-kernel
cargo run -p oclive-cli -- registry pull my-kernel -o ./my-kernel
```

### `collab` — 角色包 Git 协作

在**角色包根目录**（含 `manifest.json`）：

| 子命令 | 作用 |
|--------|------|
| `collab init --remote <git-url>` | 写入 `.oclive-collab.yml` 并 `git remote add origin` |
| `collab status` | 未提交变更 / 领先·落后远程提交数 |
| `collab pull` / `push` / `diff` | 包装 `git pull` / `push` / `diff origin/<branch>` |

`push` 前要求工作区干净；远程有新提交时 `push` 会提示先 `pull`。见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) § 协作。

---

## 持续精进（Y1–Y6）

| 能力 | 命令 |
|------|------|
| **Y3 配置** | `oclive config set/get/list/unset/init`（`~/.oclive/config.toml` / `.oclive.toml`） |
| **Y1 CI** | `oclive ci init` / `ci check`；主仓另有 `ci plan` / `ci explain` 领域感知影子计划（只报告，不跳 job） |
| **Y6 修复** | `oclive doctor --fix` / `--fix --yes` |
| **Y2 回归门禁** | `oclive bench --regression` / `--regression-threshold 3` |
| **Y5 跨版本** | `oclive bench --compare-versions v0.2.0` |
| **Y4 模板** | `oclive template create <name>` |

```bash
cargo run -p oclive-cli -- config set OCLIVE_REGISTRY_URL https://registry.example.com --global
cargo run -p oclive-cli -- ci init -o ./my-kernel
cargo run -p oclive-cli -- ci plan --shadow --base HEAD^ --head HEAD
cargo run -p oclive-cli -- ci explain --format markdown
cargo run -p oclive-cli -- doctor --fix --yes
cargo run -p oclive-cli -- --experimental bench --release --save -o ./my-kernel
cargo run -p oclive-cli -- --experimental bench --release --regression -o ./my-kernel
cargo run -p oclive-cli -- template create my-team -o ./my-kernel
```

`ci plan` 默认读取 `data/ci/impact-map.v1.json` 与 `validation-catalog.v1.json`，从 Git diff 或重复的 `--changed-file` 计算直接模块、影响闭包、验证 profile、建议 validator 和 fail-safe 全量原因，输出 `target/oclive-ci/plan.json`。`ci explain` 只解释这份 JSON，不重新计算、不执行命令。Stage 1 的 `ci-impact-plan` 为 `continue-on-error` 可见性 job；它不跳过任何主 CI 硬门禁。目录中标为 `nightly` 的验证器由独立 Nightly/手动工作流执行，不属于选择性 PR。契约、第三方隔离与脚手架辅助边界见 [OCLive 领域感知 CI](../roadmap/SOMEDAY_TOOLCHAIN_CI.md)。

---

## `scaffold`：本地声明发现、锁定与受限生成

```bash
cargo run -p oclive-cli -- scaffold list -o .
cargo run -p oclive-cli -- scaffold inspect com.oclive.scaffold.plugin -o .
cargo run -p oclive-cli -- scaffold validate ./.oclive/scaffolds/example/oclive.scaffold.json
cargo run -p oclive-cli -- scaffold resolve -o . --write-lock --json
cargo run -p oclive-cli -- scaffold generate dev.example.scaffold project \
  -o . --output ../generated --set project_name=demo --accept-untrusted
cargo run -p oclive-cli -- scaffold generate dev.example.scaffold project \
  -o . --output ../preview --set project_name=demo --accept-untrusted --dry-run --json
```

Stage 2A 按配置解析 project、user、编译内置 official 三类来源，并在 `.oclive/scaffold.lock.json` 记录来源、维护者、信任分类、权限、命名空间、兼容范围与 SHA-256。Stage 2B 只允许已选中的本地 `instruction` 生成器把文件物化到**不存在的新目录**：包必须声明 `project.write`，清单固定指令 SHA-256，指令固定每个源文件 SHA-256；项目/用户包还须与当前 lock 完全一致，并为每次调用传 `--accept-untrusted`。`--set` 只传字符串，`--dry-run` 完成相同校验但零写入；成功时 `.oclive/scaffold.provenance.json` 记录来源与产物摘要，不记录变量值。

修改包清单后，应先审查差异，再重新运行 `scaffold resolve --write-lock`。缺少指令摘要的 v1.0 包仍能被发现，但 `generate` 会拒绝，并提示把包的 `scaffold_contract` 范围提高到 `>=1.1,<2` 后补摘要。官方 `builtin` 生成器不在此处复制实现，命令会提示继续使用 `oclive init`、`oclive plugin create` 或 `oclive pack create` 等领域入口。

Scaffold Package 仍**不**联网安装、不执行 `commands[].entry`、脚本或 hook，不解析组合，也不能控制 CI workflow、验证器、Runner、Secret 或门禁。完整契约见 [RFC_SCAFFOLD_PACKAGE_V1.md](../rfc/RFC_SCAFFOLD_PACKAGE_V1.md)。

---

## `doctor`：环境诊断

```bash
cargo run -p oclive-cli -- doctor
cargo run -p oclive-cli -- doctor --json
cargo run -p oclive-cli -- doctor -o ./my-project
cargo run -p oclive-cli -- doctor --fix
cargo run -p oclive-cli -- doctor --fix --yes
```

检查 Rust/Cargo、C++ 工具链、系统内存、磁盘剩余、Ollama（`http://127.0.0.1:11434/api/tags`）、GitHub 连通、工作区可写。在 **oclivenewnew 根**且存在 `distros/chat-pro/roles/*/pipeline.ocblueprint` 时，按 `schema_version` 精确分派 v2/v3/v4，并额外执行三项蓝图检查：**`blueprint_file_format`**（文件存在且 JSON 合法）、**`slot_registry_llm`**（至少一个 `type: llm`）、**`slot_position_unique`**（同 type 下 `position` 不重复）。`--fix` 可对 Rust（`rustup update stable`）、Ollama（尝试启动 serve）等项自动修复。存在 **fail** 项时退出码非 0。JSON Schema：`kernel/crates/oclive-cli/schemas/oclive_doctor_report.schema.json`。

**`doctor config-resolve`**（六槽有效 backends + 来源链；**默认**走 `oclive_kernel_runtime::resolve_session_plugin_backends` 纯解析 + 磁盘角色包，**无** SQLite / Axum / Tauri）：

```bash
cargo run -p oclive-cli -- doctor config-resolve mumu
cargo run -p oclive-cli -- doctor config-resolve mumu --session-id demo --json
cargo run -p oclive-cli -- doctor config-resolve mumu -o distros/chat-pro/roles --json
# 深度诊断（可选）：in-memory AppState 全链 parity，需 diagnostics-host feature
cargo run -p oclive-cli --features diagnostics-host -- doctor config-resolve mumu --via-host --json
```

`--json` 时 **stdout 仅输出一个 JSON 文档**；人类可读模式的标题行走 stderr。依赖边界见 [COMPATIBILITY.md](../COMPATIBILITY.md) · [`doctor_config_resolve.rs`](../../kernel/crates/oclive-cli/src/doctor_config_resolve.rs) · runtime SSOT [`plugin_resolution.rs`](../../kernel/crates/oclive_kernel_runtime/src/domain/plugin_resolution.rs)。

**`doctor execution-plan`**（v4 扩展、Provider 候选、权限/依赖和跨发行版降级；只读，不启动插件）：

```bash
cargo run -p oclive-cli --features diagnostics-host -- doctor execution-plan mumu --json
cargo run -p oclive-cli --features diagnostics-host -- doctor execution-plan my-role \
  -o ./distros/chat-pro/roles \
  --app-data-dir ./tmp/app-data \
  --distro-profile ./distros/desktop-tauri/resources/distro-profiles/theater.oclive.toml \
  --json
```

该命令因复用宿主角色解析、Capability Registry 与 Plan Compiler 而显式要求 `diagnostics-host` feature；默认 CLI 依赖面保持轻量。输出中的 `ExecutionPlan` 只存在于内存，不写回 `pipeline.ocblueprint`；`resource_coordination: not_evaluated` 且省略 `resource_plan` 表示纯编译没有探测设备。桌面诊断才会刷新 Resource Coordinator 并附上只读候选资源计划。无 Provider 或权限时，必需扩展为 `blocked`，可选扩展为 `degraded`。

### `test --oocp`（本地 OOCP 闭环）

在 **oclivenewnew 仓库根**执行（需已能 `cargo build -p oclivenewnew-tauri --release`）：

```bash
cargo run -p oclive-cli -- --experimental test --oocp -o .
```

流程：启动 `cargo run --release -p oclivenewnew-tauri -- --api`（`OCLIVE_HTTP_API_MOCK_LLM=1`）→ 轮询 `GET /health`（默认 `http://127.0.0.1:8420`，**30s** 超时）→ `node examples/oocp-test-suite/run.mjs` → 终止内核进程。可设 **`OCLIVE_API_BASE`** 覆盖探活 URL。

---

## `pack`：角色包校验与打包

在仓库根目录：

```bash
# 蓝图角色包（按 schema_version 精确分派）
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/mumu --host-version 0.2.0
# legacy manifest/settings 包
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/legacy-example --profile legacy
# RobotSoulPack（在 legacy 校验通过后追加规则）
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/legacy-example --profile robot-soul
# 创作者 profile：仅 meta 子集 + prompts/（不校验 slot_registry / runtime_config）
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/mumu --profile creator
# Portable Core：跨发行版基础人格 + 七张默认情绪图
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/mumu --profile portable-core
cargo run -p oclive-cli -- pack validate-persona ./exports/mumu.ocpersona
cargo run -p oclive-cli -- pack validate-memory ./exports/mumu.ocmemory
cargo run -p oclive-cli -- pack create -o ./out/my-role --flat --id com.example.demo --name Demo --format-blueprint-v4
cargo run -p oclive-cli -- pack publish ./out/my-role -o ./dist/com.example.demo-0.1.0.oclivepack
```

- **`validate`（精确 v2/v3/v4 分派）**：校验 `pipeline.ocblueprint`（`meta`、`slot_registry`、至少一个 `type: llm` 等）；**v4** 是 Stable，**v3** 为冻结的双核 Beta，**v2** 含 `runtime_config` 时仅警告。见 [`ROLE_PACK_SPEC.md`](../role-pack/ROLE_PACK_SPEC.md)。
- **`validate --profile creator`**：仅角色包（`meta` 创作者子集 + **`prompts/`**）；不校验 `slot_registry` / `runtime_config`。见 [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)。
- **`validate --profile legacy`**：校验 `manifest.json` / `settings.json` 合并、`plugin_backends`、`min_runtime_version` 与 `--host-version` 等（旧包路径）。
- **`validate --profile robot-soul`**：在 **legacy** 校验通过后追加 RobotSoulPack 规则（见 ROLE_PACK_SPEC §6）。
- **`validate --profile portable-core`**：校验 v2/v3/v4 蓝图，以及非空 `core_personality.txt`、启用的 `portrait_catalog` 和七个固定默认情绪图片 ID；发行版增强能力不在此 profile 内。
- **`create`**：生成最小可校验目录；Stable 新包推荐 **`--format-blueprint-v4`**，`--format-blueprint-v2` 仅保留兼容；`--flat` 时 `-o` 即为角色根。
- **`publish`**：将角色目录打成 **ZIP**，扩展名 **`.oclivepack`**；ZIP 内顶层文件夹名为包内 **`meta.id`**（v2/v3/v4）或 **`manifest.id`**（legacy）。

**JSON Schema**（IDE / `ajv` 等）：`kernel/crates/oclive-cli/schemas/pipeline.ocblueprint.v2.schema.json`、`pipeline.ocblueprint.v3.schema.json`、`pipeline.ocblueprint.v4.schema.json`；legacy 见 `role_pack_manifest.schema.json`、`role_pack_settings.schema.json`、`role_pack_index.schema.json`。

---

## `plugin create`：插件脚手架

一键生成**目录插件**（Node `rpc_server.mjs` + 子进程）或 **Remote HTTP 插件**（Python `rpc_server.py`）骨架，含 `manifest.json`（`id` / `provides` / `permissions` / `rpcMethods`）、README 与 RPC 方法桩。

```bash
# 非交互：目录插件，提供 llm 槽
cargo run -p oclive-cli -- plugin create my-llm-plugin --type directory --provides llm -o ./distros/chat-pro/plugins/

# Remote 侧车，多槽
cargo run -p oclive-cli -- plugin create my-remote --type remote --provides memory --provides emotion -o ./out/plugin --non-interactive

# 交互：选择类型、槽位、输出目录
cargo run -p oclive-cli -- plugin create my-plugin
```

**`--provides`**：`llm` | `memory` | `emotion` | `event` | `prompt` | `agent` | `complex_emotion`（可重复）。输出目录默认为 `./distros/chat-pro/plugins/`；最终包路径为 `<output>/<plugin_id>/`（`id` 由名称 slug 为 `com.oclive.plugin.<name>`）。

生成 manifest 经 **`oclive_validation`** 权限校验（目录插件）。快速上手见 [PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)。

**`slot_attachment`**：在 `manifest.json` 中声明后，安装时加 **`--role <pack-dir>`** 自动更新 `pipeline.ocblueprint`（见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)）。

---

## `plugin manage`：高级槽位与蓝图

主应用默认**不**展示架构图；开发者用本子命令组管理 **`slot_registry`**。

```bash
# 列出槽位（默认 ./distros/chat-pro/roles/ 下唯一包，或 --role）
cargo run -p oclive-cli -- plugin manage list
cargo run -p oclive-cli -- plugin manage list --role distros/chat-pro/roles/mumu --json

# 增删槽、改 backend、关联目录插件
cargo run -p oclive-cli -- plugin manage add-slot llm "My LLM" --role distros/chat-pro/roles/mumu
cargo run -p oclive-cli -- plugin manage set-backend llm directory --role distros/chat-pro/roles/mumu
cargo run -p oclive-cli -- plugin manage link llm com.example.my-llm --role distros/chat-pro/roles/mumu
cargo run -p oclive-cli -- plugin manage unlink llm --role distros/chat-pro/roles/mumu
cargo run -p oclive-cli -- plugin manage remove-slot memory_2 --role distros/chat-pro/roles/mumu

# TUI 总览（环序示意 + 槽位列表）
cargo run -p oclive-cli -- plugin manage --tui --role distros/chat-pro/roles/mumu
```

| 子命令 | 说明 |
|--------|------|
| `list` | 列出 `slot_registry` 键、type、backend、plugin |
| `add-slot <type> <label>` | 新增实例 |
| `remove-slot <key>` | 删除实例（须保留至少一个 `llm`） |
| `set-backend <key> <backend>` | 修改 backend |
| `link <key> <plugin-id>` | 设为 `directory` 并写入 `plugin` |
| `unlink <key>` | 清除 `plugin` |

安装并自动装配：`cargo run -p oclive-cli -- plugin install <id> --role distros/chat-pro/roles/mumu`。

---

## `dev`：角色包目录监听

在**已存在**的内核 / 脚手架项目根（含 `Cargo.toml`）执行。使用 **notify 递归模式**监听 **`distros/chat-pro/roles/**/manifest.json`** 与 **`distros/chat-pro/roles/**/settings.json`**（任意子目录角色包）；**500ms 防抖**后打印：

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

流程包括：项目名、类型（无头可执行 / 库）、后端槽位多选、`builtin` / `remote` / `directory` / `none`（`llm` 槽另有 **`ollama`**）选择、可选插件开关、是否生成示例 `distros/chat-pro/roles/default`；**无头服务（kernel_server）** 末尾另有 **开发者编译选项**（默认关闭）。

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

`--skip-role-pack`：不生成 `distros/chat-pro/roles/`（空白内核工程）。

### 环境推荐（`--smart`）

```bash
# 仅打印推荐（不生成工程）
cargo run -p oclive-cli -- init --smart --non-interactive -o ./out --project-name my_kernel
```

交互式 `init` 默认会先做一次轻量探测（Ollama / NVIDIA GPU / 内存）；可用 **`--no-smart`** 关闭。完整诊断仍用 **`oclive doctor`**。

对含 `oclive_kernel_*` 依赖的内核工程，`doctor` 另增五项 **contracts 实现** 检查（`plugin_host_port_impl`、`llm_client_impl`、`slot_registry_resolver_impl`、`event_estimator_impl`、`agent_provider_impl`）：在 `src/` 或 `distros/desktop-tauri/src/` 下搜索对应 `impl Trait` 块。在 monorepo 根目录执行时探测 `distros/desktop-tauri/Cargo.toml`。

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
cargo run -p oclive-cli -- --experimental init --non-interactive --template headless-api --monolith -o ./out/api-weld
cargo run -p oclive-cli -- init --non-interactive --template library-embed --kernel-source . -o ./out/embed
```

**`--with-role-pack`**：`robot-soul-minimal`（七维 + `prompts/system.md`）| `default`（通用 `distros/chat-pro/roles/default`）。未指定且未用模板时，非交互仍生成 **default** 示例包（与历史一致）。

**`--with-example-plugin`**：复制 `com.oclive.example.llamacpp_llm/` 到 `distros/chat-pro/plugins/`（源自主仓 `examples/directory-plugin-llamacpp/`；默认关闭）。

生成工程含 **`distros/chat-pro/plugins/README.md`**、**`docs/BLUEPRINT_REFERENCE.md`**、**`docs/ORCHESTRATION_REFERENCE.md`**（中英编排参考）。

**蓝图校验**（`[experimental/legacy]`，不改变桌面宿主主路径；新工程优先 `init --pipeline`）：

```bash
cargo run -p oclive-cli -- --experimental blueprint validate ./distros/chat-pro/roles/myrole/pipeline.ocblueprint
cargo run -p oclive-cli -- --experimental blueprint validate ./path.json --json
```

启用 Monolith（`--non-interactive` 下加 **`--monolith`**；仅 **kernel_server**）：

```bash
cargo run -p oclive-cli -- --experimental init --non-interactive --preset full --monolith --monolith-preset latency -o /tmp/my-monolith-kernel
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
| `--monolith-preset` | 仅 Monolith 启用时：`latency`（七焊接键全焊）\| `memory` \| `embedded`；预填 `monolith.toml` 的 `weld_modules` |
| `--monolith-bench-preset` | 同档位枚举；生成后自动 release 双构建 + `bench --runs 5` → `bench_results/report.json`（失败不阻塞） |
| `--list-templates` | 打印模板矩阵后退出；交互 `init` 亦可在项目类型前选模板 |
| `--quick` / `-q` | 极速：`preset=full`、无 Monolith、无 `distros/chat-pro/roles/`；交互仅问项目名与输出目录 |
| `--template` | `robot-soul` \| `robot-gateway` \| `dialogue-only` \| `headless-api` \| `library-embed`（内核工厂套餐；见上表） |
| `--with-role-pack` | `robot-soul-minimal` \| `default`；与 `--skip-role-pack` 互斥 |
| `--with-example-plugin` | 附带 llamacpp 目录插件示例到 `distros/chat-pro/plugins/` |
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
cargo run -p oclive-cli -- --experimental compose init
cargo run -p oclive-cli -- --experimental compose up
cargo run -p oclive-cli -- --experimental compose down
cargo run -p oclive-cli -- --experimental compose ps
```

见项目根 **`oclive-compose.yml`**（`services.<id>.path` / `port` / `env` / `depends_on`）。**`up`** 按依赖顺序启动，日志带 **`[服务名]`** 前缀；状态写入 **`.oclive-compose.pids.json`**。

---

## `template`：旧工程归档兼容

```bash
cargo run -p oclive-cli -- template pack -o ./my-kernel -O ./my-kernel.oclive-template.tar.gz
cargo run -p oclive-cli -- template create my-team -o ./my-kernel
cargo run -p oclive-cli -- init --template-url https://example.com/template.tar.gz -o ./from-remote
```

- **`template pack`**：将工程打包为 **`.oclive-template.tar.gz`**（排除 `target/`、`.git/` 等；含 `template.json`）。
- **`template create`**：反向分析工程并注册 `~/.oclive/templates/`。
- 顶层 **`publish` 已移除**；旧归档不参与 `oclive scaffold` 的 v1 发现链，不能携带 CI 控制配置。

---

## `debug`：逐步骤追踪

```bash
cargo run -p oclive-cli -- --experimental debug -o ./my-kernel --kernel-source  # 工程须已接主仓内核
cargo run -p oclive-cli -- --experimental debug -o . --step build_prompt --json
```

设置 **`OCLIVE_DEBUG_TRACE=1`** 并启动 **`--api`**（默认 Mock LLM）。详见生成工程 **`docs/DEBUG_REFERENCE.md`**。

---

## 生成物说明

- **占位 `Cargo.toml`**：当前仅依赖 **`serde` / `serde_json`**，不假设本机已存在 `oclive_kernel_runtime` 拆分 crate。接入真实内核时，请改为 `path` / 版本依赖并替换 `main.rs` / `lib.rs` 入口。
- **`distros/chat-pro/roles/default/settings.json`**：含 **`_comment_*`** 与完整 **`plugin_backends`**（含第 7 键 `complex_emotion`）；与主应用完全对齐时请以 [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) 为准裁剪非法键（如主应用不接受的 `none` 字符串）。
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
cargo run -p oclive-cli -- --experimental build -o /path/to/kernel-project
cargo run -p oclive-cli -- --experimental build -o /path/to/kernel-project --release --features somefeat
cargo run -p oclive-cli -- --experimental build -o /path/to/kernel-project --no-cargo
```

- **`--no-cargo`**：仅再生成 `process_message_monolith.rs` 与 vendor，不调用 `cargo`。
- **`--release`** / **`--features`**：传给每次 `cargo build`；Monolith 次构建会自动并入 **`monolith`** feature。
- **`--` 之后**：附加参数透传给 `cargo build`。

**常见编译错误与修复**：`cargo build` 失败时，CLI 会解析 stderr 并给出中文建议（crate 未找到、缺少 C 链接器、Rust 版本过低、OpenSSL 开发库、内存不足等）。未匹配时保留原始输出并提示运行 **`oclive doctor`**。

### `bench` 子命令

`bench` 当前仍在实验门内，命令必须在子命令前加全局 **`--experimental`**；以下示例均已包含。

再生成源码、双构建后，对两个二进制各跑 `--runs` 次子进程；子进程内通过环境变量 **`OCLIVE_KERNEL_BENCH_ITERS`** 做热循环。输出 **JSON**（`schema_version: 2`），除延迟分位数外含 **`binary_size`**（字节）、**`peak_memory`**（MiB）、**`build_time`**（秒）。Schema：`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`。

```bash
cargo run -p oclive-cli -- --experimental bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- --experimental bench --release -o /path/to/kernel-project --json
```

- **`--save`**：将本次报告追加到项目根 **`bench_history.json`**（本地文件，勿提交）。
- **`--compare`**（历史对比）：**不运行采样**；对比 `bench_history.json` 中**最近两次** save 记录（需 ≥2 条）。用于开发时肉眼看趋势，**无退出码门禁**。
- **`--regression`**（回归门禁）：**先跑本轮 bench**，再与历史中**最近一条** save 对比；任一指标超阈值则 **退出码 1**（CI 用）。与 `--compare` 不同，勿混用。
- **`--compare-versions <git-ref>`**：当前工作区 vs 指定 Git 引用各跑多轮，输出矩阵表。
- **`--live`**：终端 sparkline 实时仪表盘（`q` 退出）。**勿与** 顶层 Web **`oclive dashboard`** 混淆；`--dashboard` 为 deprecated 别名。
- **`--history`**：打印 **`bench_history.json`** 全部记录的趋势表；可加 **`--json`**。
- **`--soak --soak-duration <小时>`**：默认走 8～120s 的加速冒烟时钟；支持小数小时，但不构成长时间泄漏证据。
- **`--soak-real-time`**：将 `--soak-duration` 解释为真实墙钟小时；`--soak-sample-interval <秒>` 控制资源采样间隔（默认 60）。soak 先完成一次 `warmup_chats`，再启动时钟并取得稳态 RSS 基线；之后直接采样 Release 内核 PID。schema v2 同时记录 RSS/CPU、请求失败、提前退出、工作线程 join 与 `process_reaped`；任何验收失败均非零退出。

**未实现**：`bench history clear` / `export` / `import` 子命令（见路线图「计划中」）。

```bash
cargo run -p oclive-cli -- --experimental bench --release -o ./my-kernel --save
cargo run -p oclive-cli -- --experimental bench --history -o ./my-kernel
cargo run -p oclive-cli -- --experimental bench --history -o ./my-kernel --json
cargo run -p oclive-cli -- --experimental bench --watch -o ./my-kernel
cargo run -p oclive-cli -- --experimental bench --soak --soak-real-time --soak-duration 0.01 --soak-sample-interval 5 -o ./my-kernel --output ./soak.json
```

- **`--watch`**：监听 **`src/`** 与 **`Cargo.toml`**（2s 防抖），自动 release 构建 + **3 轮** bench 并 **`--save`**，打印相对上一轮 **↑/↓/→**。

`--json`：仅将报告 JSON 打印到 **stdout**（进度走 **stderr**），便于管道与 Schema 校验。`--output <文件>`：为普通基准、`--stress` 与 `--soak` 在目标目录原子保存 JSON，适合长稳与破坏性测试证据；两者同时出现时保持 `--json` 的 stdout 语义。

**风险**：占位工程 **无** 真实 `PluginHost` 行为。

权威设计：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)。

---

## 与 CI 的关系

仓库 **`.github/workflows/ci.yml`** 的 **`cli`** job 会 `cargo test -p oclive-cli`（含端到端：`init`、`build`、`bench` smoke）。独立 **`.github/workflows/nightly-advisory.yml`** 的 **`cli-bench`** job 跑一轮 `bench`（不设性能阈值）并上传 JSON 证据。

---

## 后续路线（建议）

1. 在 workspace 中落地 **`oclive_kernel_runtime`** 后，为 CLI 增加 **`--kernel-source path`**，自动写入 `Cargo.toml` 依赖。  
2. 与 `MODULE_NONE_SEMANTICS` 对齐时，为「逻辑 none」与「可加载 JSON」生成 **自动校验** 或 `cargo oclive-validate-settings` 子命令。

---

[English](../../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md)
