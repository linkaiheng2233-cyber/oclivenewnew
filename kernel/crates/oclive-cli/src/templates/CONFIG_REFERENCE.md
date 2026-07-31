# oclive-cli 生成项目：`plugin_backends` 预设对照

本文件由 **`oclive-cli init`** 自动生成，与 `init --help` 中的预设矩阵一致。正式契约以主仓 **[PLUGIN_V1.md](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)** 与 **`src-tauri/src/models/plugin_backends.rs`** 为准；权威说明见 **[SETTINGS_REFERENCE.md](../../../creator-docs/cli/SETTINGS_REFERENCE.md)**。

## 内核工厂模板（`--template`）

与 `init --help` 末尾模板表一致；显式传入的 **`--preset`** / **`--project-type`** / **`--monolith`** / **`--dual-core`** / **`--with-role-pack`** 优先于模板默认值。

**`--dual-core`**（需生成 `roles/`）：示例包写入 **`pipeline.ocblueprint` `schema_version: 3`**，含 `runtime_config.dual_core.enabled: true` 与 `pipeline.stable` / `pipeline.experimental`（Stable 段仅供文档，宿主 Stable 核仍为 `co_present`）。与 **`--monolith`** 同用时在 `monolith.toml` 增加 `[dual_core] enabled = true`。

| template | preset | Monolith | project-type | 默认角色包 |
|----------|--------|----------|--------------|------------|
| `robot-soul` | minimal | 启用 | kernel_server | `robot-soul-minimal`（`prompts/system.md` + 七维） |
| `robot-gateway` | mixed | 启用 | kernel_server | 无（厂商自订 `roles/`） |
| `dialogue-only` | full | 关闭（可加 `--monolith`） | kernel_server | `default`（通用示例） |
| `headless-api` | full | 关闭（可加 `--monolith`） | kernel_server | 无 |
| `library-embed` | minimal | 关闭 | library | 无 |

**`--monolith-preset`**（仅 Monolith 启用时写入 `monolith.toml` 的 `weld_modules`）：

| 档位 | weld_modules |
|------|----------------|
| `latency` | 全部七焊接键 |
| `memory` | memory, prompt, llm |
| `embedded` | emotion, memory, llm |

愿景说明：[KERNEL_FACTORY_VISION.md](../../../creator-docs/getting-started/KERNEL_FACTORY_VISION.md)

生成工程含 **`docs/BLUEPRINT_V2_POINTER.md`**、**`docs/WELD_BENCH_REPORT.md`**（中英焊接对比报告模板），以及 **`plugins/README.md`**。

| 参数 | 说明 |
|------|------|
| `--list-templates` | 打印模板矩阵后退出 |
| `--monolith-bench-preset` | Monolith 启用时：设定 `weld_modules` 并在生成后自动 `bench --runs 5` → `bench_results/report.json` |
| `--with-example-plugin` | 复制 `com.oclive.example.llamacpp_llm` 示例 |
| `--author` / `--license` / `--description` | 写入生成 `Cargo.toml` 的 package 元数据（license 默认 MIT） |

**插件脚手架**（仓库根）：`cargo run -p oclive-cli -- plugin create <name> --type directory|remote --provides <slot> …`

**平台扩展**：`registry` · `compose` · `publish` / `init --template-url` · `init --tui` · `bench --watch` · `debug` — 见 [OCLIVE_CLI_GUIDE.md](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)。

**`robot-gateway` 模板**额外生成 **`mcp_servers/`** 与 **`roles/gateway/settings.json`**（`agent` = builtin + `agent_mcp` 占位）。

## 预设矩阵（逻辑槽位）

| 槽位 | minimal | mixed | full |
|------|---------|-------|------|
| memory | builtin | builtin | builtin |
| emotion | builtin | builtin | builtin |
| event | builtin | builtin | builtin |
| prompt | builtin | builtin | builtin |
| llm | ollama | ollama | remote |
| agent | none（JSON 省略键，回退宿主默认 builtin） | builtin | builtin |
| complex_emotion | none | builtin | remote |

说明：

- **`llm`**：主应用 v1 枚举为 **`ollama` \| `remote` \| `directory`**，无字面量 `builtin`。对照表中「本地默认」对应 JSON 中的 **`ollama`**（进程内 Ollama 客户端）。
- **`agent` = none**：内核结构体无 `none` 变体；脚手架在 **`settings.json` 中省略 `agent` 键**，加载时与显式 **`builtin`** 等价（均为默认内置实现）。
- **`complex_emotion`**：当前桌面宿主 **`PluginBackends` 仅含六槽**；该键写在 **`plugin_backends` 内便于阅读**，宿主反序列化时会**忽略未知字段**，不影响 `load_role`。

## 各槽一句话

| 槽位 | 作用 |
|------|------|
| memory | 记忆检索与排序 |
| emotion | 用户情绪分析 |
| event | 事件影响估计 |
| prompt | 主 prompt 组装 |
| llm | 主对话与短分类生成 |
| agent | 工具编排 / ReAct |
| complex_emotion | 复杂情感扩展（路线图；键保留供侧车实验） |

## 切换后端（概要）

1. 编辑 **`roles/default/settings.json`** 的 `plugin_backends` 对应字段。
2. **`remote`**：配置 **`OCLIVE_REMOTE_PLUGIN_URL`** / **`OCLIVE_REMOTE_LLM_URL`** 等（见 PLUGIN_V1 与 REMOTE_PLUGIN_PROTOCOL）。**远端失败是否静默降级内置**由主应用 `app_settings.remote_fallback_to_builtin` 与 **`OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`** 环境变量控制（默认允许降级）；关闭时不可达侧车将返回 **`REMOTE_SERVICE_UNAVAILABLE`**。
3. **`directory`**：在包内配置 **`plugin_backends.directory_plugins`** 各槽的 manifest **`id`**，并放置 **`plugins/<id>/`**（见 DIRECTORY_PLUGINS.md）。

## 开发者编译选项（已可用）

**高耦合编译模式（Monolith）**：在编译期增加 **`Cargo` feature `monolith`** 与第二二进制 **`{package名}-monolith`**；`src/process_message_monolith.rs` 与 **`vendor/oclive_monolith_builtin/`** 由 **`oclive init`** 或 **`cargo run -p oclive-cli -- --experimental build`** 生成。**仅 `kernel_server` 项目**会生成 **`monolith.toml`**；嵌入式 **library** 忽略 Monolith。

- **非交互**：`cargo run -p oclive-cli -- --experimental init --preset full --monolith --monolith-preset latency -o ./out`（勿与 `--project-type library` 同用）。
- **蓝图校验**：`cargo run -p oclive-cli -- pack validate <角色根>`（按 `schema_version` 精确分派 v2/v3/v4；见 `docs/BLUEPRINT_V2_POINTER.md`）。
- **交互**：流程末尾「是否启用开发者编译选项？」→「编译模式」（标准 / 全槽焊接 / 自定义焊接范围）。
- **再生成**：`cargo run -p oclive-cli -- --experimental build -o ./out`（默认继续两次 `cargo build`；`--no-cargo` 仅写源码与 vendor）。
- **构建**：亦可手动 `cargo build --release`（标准）、`cargo build --release --features monolith`（焊接产物）。
- **权威设计**：[RFC_OCLIVE_MONOLITH_MODE.md](../../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)。

## `oclive dev`（角色包目录监听）

在**已生成**的内核 / 脚手架项目根（含 `Cargo.toml`）执行；默认监听 **`roles/`** 下递归变更，对 **`manifest.json`** / **`settings.json`** 防抖后打印提示，可选 `--reload-cmd` 触发自定义命令。

```bash
cargo run -p oclive-cli -- dev -o /path/to/project
cargo run -p oclive-cli -- dev -o /path/to/project --roles roles --reload-cmd "echo reload"
cargo run -p oclive-cli -- dev -o /path/to/project --no-watch
```

详见主仓 [OCLIVE_CLI_GUIDE.md](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)（若与本仓库并列检出）。

## `oclive bench --save` / `--compare`（Monolith 项目）

在含 **`monolith.toml`** 的项目根：

- **`--save`**：在 **`bench_history.json`**（项目根，勿提交版本库）中追加本次 JSON 报告。
- **`--compare`**：**不**重新跑采样；读取历史中**最近两次**记录并打印对比摘要（需先至少两次带 **`--save`** 的 bench，或等价历史）。

```bash
cargo run -p oclive-cli -- --experimental bench --release -o /path/to/monolith-project --runs 20 --save
cargo run -p oclive-cli -- --experimental bench --release -o /path/to/monolith-project --compare
```

与基础 **`bench`**（输出单次 JSON）的关系见主仓 **`creator-docs/cli/OCLIVE_CLI_GUIDE.md`** 与 **`crates/oclive-cli/src/bench_cmd.rs`** 内帮助说明。
