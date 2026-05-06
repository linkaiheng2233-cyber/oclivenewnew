# 轻量场景 × Cargo features × OOCP / invoke 对照

> 维护：与 `KERNEL_BOUNDARY.md`、`KERNEL_API_IMPLEMENTATION_MATRIX.md` 交叉引用；文档总索引见 [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](../getting-started/DOCUMENTATION_INDEX.md)。  
> 目的：嵌入式宿主、侧车 / pack-editor、官方桌面三种形态下，明确 **runtime 特性**、**OOCP 方法**与 **Tauri invoke** 的可用边界。

**术语**：下表中的 **`oclive_*_builtin`** 与进程内 Builtin，在产品文档中称 **官方默认记忆模块** 等，定义见 [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md) §1.1；与 **第九模块（专家模型设施）** 非同一概念。

---

## 1. 场景总览

| 场景 | 典型宿主 | `oclive_kernel_runtime` | HTTP / Axum | ZIP 角色包与插件归档 | 市场索引同步 | Agent / MCP |
|------|-----------|-------------------------|-------------|----------------------|--------------|-------------|
| **官方桌面** | `src-tauri`（默认依赖） | `full`（默认） | 开（`kernel-http-api`） | 开（`role-pack-zip`） | 开（`market-sync`） | 开（`kernel-agent` + `default-agent-providers`） |
| **kernel_server / pack-editor 试聊** | `oclive_kernel_server` | `full`（默认） | 开 | 开 | 开 | 开 |
| **嵌入式 lib / 玩偶侧车** | 自建进程，仅需 OOCP+编排 | `default-features = false` + 按需子特性 | 常关 | 常关 | 常关 | 常关 |

### 1.1 冷启动与延迟 I/O（runtime 行为摘要）

- **目录插件**：`DirectoryPluginRuntime::bootstrap` 不遍历磁盘；首次 `ensure_scanned` / `rescan_plugin_roots` 才扫描。`rescan_plugin_roots` 在 **`OCLIVE_APP_DATA_DIR/.oclive_plugin_scan_cache_v1.json`** 与扫描根 **mtime 指纹**一致时可跳过 manifest 解析（日志 `disk_cache_hit=true`）；**`OCLIVE_BUST_PLUGIN_SCAN_CACHE=1`** 删除缓存并强制全量扫描。
- **市场索引**：`market-sync` 能力由 Tauri / 宿主在 **用户触发同步** 时拉网；**`oclive_kernel_server`** 冷启动不主动跑 HTTP 索引同步。
- **MCP**：`McpClient::list_servers` 在 **`mcp-servers/`** 目录 `modified` 未变时复用内存列表，避免每次重读 JSON。
- **`lazy-init`**：见下表；嵌入式希望 **更快冷启动** 时保持 **`full`**（含 `lazy-init`）；希望 **首包前就绪** 时用 **`--no-default-features`** 再按需开特性并 **关闭 `lazy-init`**。

**`AppError` → Tauri**：类型定义在 `oclive_kernel_core`；桌面命令请使用 `map_err(|e: AppError| e.to_frontend_error())` 等到 `String`（不再通过 `oclive_kernel_runtime` 的 `tauri` 可选依赖做 `InvokeError` 转换）。

---

## 2. `oclive_kernel_runtime` 特性（聚合）

定义见 crate 根目录 `Cargo.toml`。

| Feature | 作用 |
|---------|------|
| **`full`**（默认） | `kernel-http-api` + `role-pack-zip` + `market-sync` + `kernel-agent` + **`lazy-init`** + **`facility-classic-algorithms`** + **`default-llm-providers`** + **`default-memory-providers`** + **`default-emotion-providers`** + **`default-event-providers`** + **`default-prompt-providers`** + **`default-complex-emotion-providers`** + **`default-agent-providers`** |
| **`facility-classic-algorithms`** | 为三个设施 crate **显式**打开 **`classic`**（完整记忆 importance 排序、情绪关键词七维、`affect_metrics_from_seven_dim`）。**`full`** 包含此项。典型 **`cargo check -p oclive_kernel_runtime --no-default-features`** 下三项均关 → 对应 **stub**（FIFO 记忆取样、强中性情绪、效价恒 0）。**注意**：若单独启用 **`default-*-providers`**，各 crate 的 `providers` feature 会 **隐含 `classic`**，该模块仍走完整算法，即使未列 `facility-classic-algorithms`。详见 [`FACILITY_CLASSIC_ALGORITHMS_AUDIT.md`](./FACILITY_CLASSIC_ALGORITHMS_AUDIT.md)。 |
| **`default-llm-providers`** | 内置 **Ollama**、**OpenAI-compatible 云 HTTP**，以及 **`OCLIVE_REMOTE_LLM_URL` JSON-RPC 侧车**（`RemoteLlmHttp` / `llm_remote_backend` 中的侧车分支）。**关闭**时 crate 仍可编译，`LlmClient` trait 可用，但内核**不**包含上述任一内置 LLM 网络路径；角色 **`plugin_backends.llm = remote`**（依赖侧车）亦无法生效。**须通过 `plugin_backends.llm = directory` 目录插件**（`PluginJsonRpcLlm` RPC）等方式接入 LLM，否则默认占位会得到明确的 `InvalidParameter` 错误。Ollama 模型列表/健康检查等 API 在关闭本特性时同样返回说明性错误。 |
| **`default-memory-providers`** | **官方默认记忆模块**（**`oclive_memory_builtin`** 的 `providers` feature）：进程内 **`MemoryRetrieval` Builtin / BuiltinV2**；关闭后 builtin / Remote 占位回退为轻量桩（`DisabledMemoryRetrieval`）。`providers` **隐含**设施 crate 的 **`classic`**（与 Builtin 实现一致）。`MemoryEngine` 与 Remote HTTP 仍通过 **`oclive_memory_builtin::classic`** 做上下文/搜索。恢复 **directory** 形态：安装示例插件 **`examples/oclive-memory-builtin-directory/`**（`com.oclive.builtin.memory`），在角色包设 `plugin_backends.memory = directory` 且 `directory_plugins.memory` 指向该 id，并授予 **`process:spawn`**。 |
| **`default-emotion-providers`** | **官方默认情绪模块**（**`oclive_emotion_builtin`** 的 `providers`）：进程内 **`UserEmotionAnalyzer` Builtin / BuiltinV2**；关闭后回退中性分布桩。`providers` 隐含 **`classic`**（关键词表）。编排层对 `EmotionAnalyzer` 的静态方法仍经 `emotion_analyzer` 重导出；**关 `facility-classic-algorithms`** 且无本项时走 **`classic` stub**（强中性，与 `DisabledUserEmotionAnalyzer` 对齐）。恢复 **directory** 形态：示例 **`examples/oclive-emotion-builtin-directory/`**（`com.oclive.builtin.emotion`，`emotion.analyze`），需 **`process:spawn`**。 |
| **`default-event-providers`** | **官方默认事件模块**：**`oclive_event_builtin`** 含 **`EventDetector` / `event_impact_ai`** 与 **`BuiltinEventEstimator*`**；runtime 仅 **`KernelEventImpactEngine`** 桥接 **`EventImpactEngine`**。关闭后回退 `Ignore` 桩。目录形态示例：**`examples/oclive-event-builtin-directory/`**（`event.estimate`，需 **`process:spawn`**）。见 [`KERNEL_V2_DESIGN.md`](./KERNEL_V2_DESIGN.md) §6。 |
| **`default-prompt-providers`** | **官方默认 Prompt 模块**（**`oclive_prompt_builtin`** 的 `providers`）：进程内 **`PromptAssembler` Builtin / BuiltinV2**（`PromptBuilder` 正文在设施 crate 的 **`classic`**）；关闭后 builtin / Remote 占位回退 **`DisabledPromptAssembler`**（空串 / 无 topic hint）。`providers` **隐含**设施 crate 的 **`classic`**。恢复 **directory** 形态：先 `cargo build -p oclive_prompt_builtin --features prompt-from-json-bin --bin oclive_prompt_from_json`，设 **`OCLIVE_PROMPT_FROM_JSON`**，再安装示例 **`examples/oclive-prompt-builtin-directory/`**（`com.oclive.builtin.prompt`，`prompt.build_prompt` / `prompt.top_topic_hint`），角色包设 `plugin_backends.prompt = directory` 且 `directory_plugins.prompt` 指向该 id，并授予 **`process:spawn`**。 |
| **`default-complex-emotion-providers`** | **官方默认复杂情感模块**（**`oclive_complex_emotion_builtin`** 的 `providers`）：进程内 **`ComplexEmotionProvider` 关键词实现**；关闭后回退轻量桩。`providers` 隐含 **`classic`**。编排路径中的 **`affect_metrics_from_seven_dim`** 在关 **`facility-classic-algorithms`** 时为 **`(0,0)` 桩**。恢复 **directory** 形态：示例 **`examples/oclive-complex-emotion-builtin-directory/`**（`com.oclive.builtin.complex_emotion`，`complex_emotion.resolve_turn`），需 **`process:spawn`**。 |
| **`default-agent-providers`** | **官方默认 Agent 模块**（**`oclive_agent_builtin`** 的 `providers`）：进程内 **`BuiltinReActAgent`**（基于 **`LlmClient` + `McpInvoke`**）。关闭后若仍开启 **`kernel-agent`**，仅装配 **`McpShellAgent`**（`AgentProvider::process` 恒返回未接管）。须 **`kernel-agent` + `default-agent-providers`** 同开才有进程内 ReAct。恢复 **directory**：示例 **`examples/oclive-agent-builtin-directory/`**（`com.oclive.builtin.agent`，`agent.process`），需 **`process:spawn`**。 |
| **`kernel-http-api`** | Axum HTTP + OOCP WebSocket（`http_api` 模块） |
| **`role-pack-zip`** | `zip` 依赖；`plugin_archive`、`role_pack_archive`；插件 / 角色包归档安装路径 |
| **`market-sync`** | `plugin_index_sync`、`plugin_reviews_index_sync`、`role_market_index_sync` |
| **`kernel-agent`** | MCP 客户端（`McpClient` / `McpInvoke`）、`RemoteAgentHttp`、目录 Agent HTTP 槽、**`McpShellAgent`**；与 **`default-agent-providers`** 组合时装配 **`BuiltinReActAgent`**。 |
| **`lazy-init`**（默认随 **`full`**） | **开启**：`KernelAppState::new` / 内存测试构造后不主动扫插件目录、不预取 MCP 清单（与目录 **`ensure_scanned`**、`McpClient::list_servers` 按需一致）。**关闭**：构造末尾执行 **`rescan_plugin_roots`**，并在 **`kernel-agent`** 编译进时 **`list_mcp_servers`** 预热 MCP 缓存。市场索引 HTTP 同步仍由桌面 **`invoke`** / 编写器触发，不在本 crate 冷启动路径。 |

**注意**：关闭 `role-pack-zip` 时，`plugin_install` 中带解压的实现会返回明确错误；关闭 `market-sync` 时，同步函数所在模块不参与编译，由宿主（如 `src-tauri` 的 `plugin_installer` / `role_market`）保证不与该组合链接。

### 2.1 事件模块：设施 crate 与 runtime 桥（阶段 7 已收尾）

- **Trait 所在**：**`EventEstimator`** 在 **`oclive_kernel_core::event_estimator`**，与 **`PromptAssembler`** 等门面同级；**不**依赖完整 runtime。
- **`default-event-providers` 开**：链接 **`oclive_event_builtin`**（**`EventDetector` / `event_impact_ai` / `BuiltinEventEstimator*`**）；**`EventImpactEngine`** 由 runtime **`KernelEventImpactEngine`** 委托 **`oclive_event_builtin::estimate_event_impact`**。
- **`default-event-providers` 关**：builtin 槽为 **`DisabledEventEstimator`**（`Ignore` / 零影响），与 **`MODULE_NONE_SEMANTICS.md` §3** 一致。
- **裁剪含义**：关 **`default-event-providers`** 可减小链接面（**`oclive_event_builtin` 仍为 runtime 直接依赖**，供规则检测等路径；极薄 SKU 可后续再拆 feature）。设计结论见 [`KERNEL_V2_DESIGN.md`](./KERNEL_V2_DESIGN.md) §6。

---

## 3. OOCP（`oclive_core::oocp_handler`）

capabilities 中的方法列表见 `OOCP_METHODS`。以下能力与 runtime 特性关系：

| 能力 | 关闭 `kernel-agent` 时的行为 |
|------|------------------------------|
| `agent.call_mcp_tool` | 适配层应返回错误（例如「Agent / MCP 未编译」）；协议仍可在 capabilities 中出现，由宿主决定是否收窄握手 |

其余 chat / role / time 等方法不依赖上述可选模块。

---

## 4. Tauri `invoke` 与前端契约

完整命令列表由 `src-tauri/src/lib.rs` 的 `tauri::generate_handler!` 注册（见 `KERNEL_API_IMPLEMENTATION_MATRIX.md`）。

### 4.1 按主题的 invoke 分组（便于 SKU / 前端对齐）

下列分组与 `src-tauri/invoke_lists/*.txt` 及 Cargo `invoke-*` 特性一致；**默认 `invoke-full` 仍为全量注册**。若本地执行了 `cargo check -p oclivenewnew-tauri --no-default-features --features tauri-app,custom-protocol`，`build.rs` 会把 `src/gen/tauri-invoke-capabilities.ts` 写成全 `false`；恢复官方前端契约可再跑一次默认的 `cargo check -p oclivenewnew-tauri`（或从版本库还原该文件）。

| 分组 | 命令示例（Rust 路径 → 前端 camelCase 参数） |
|------|---------------------------------------------|
| **Agent / MCP** | `api::agent::*` → `listMcpServers`、`callMcpTool` 等 |
| **角色包 I/O** | `api::role_pack::*`、`preview_local_plugin_archive_command`、`install_local_plugin_archive_command` |
| **角色 / 插件市场** | `api::role_market::*`、`api::plugin_index::*`、`api::plugin_reviews::*`、`api::plugin_update::*` |
| **创作者工具链** | `api::plugin_scaffold::*`、`api::plugin_pack::*`、`api::plugin_debug::*` |

### 4.2 桌面 `invoke-*` 特性与 `generate_handler!` 裁剪（已实现）

`tauri::generate_handler!` 过程宏**不能**在参数里嵌套展开子 `macro_rules!()` 片段；做法是在 `src-tauri/src/invoke_registry.rs` 的 **`oclive_invoke_handler!` 单条列表** 上，对可选命令逐条写 `#[cfg(feature = "invoke-…")]`（cfg 在过程宏之前剥离）。

- **Cargo**：`oclivenewnew-tauri` 的 `default` 包含 `invoke-full`；后者聚合 `invoke-agent`、`invoke-expert-models`、`invoke-role-market`、`invoke-plugin-market`、`invoke-plugin-creator`。极简 SKU 示例：`cargo build -p oclivenewnew-tauri --no-default-features --features tauri-app,custom-protocol`（仅核心 invoke；需同步前端能力文件，见下）。
- **前端契约**：`src-tauri/build.rs` 在带 `tauri-app` 的 `cargo build`/`check` 时重写 `src/gen/tauri-invoke-capabilities.ts`；`src/lib/tauriInvokeCapabilities.ts` 维护命令名到分组的映射；`src/utils/tauri-api.ts` 在 `invoke` 前对缺省分组给出友好错误。新增可选分组命令时须同时改 **Rust 宏列表** 与 **`COMMAND_CAPABILITY` 映射**。

---

## 5. `src-tauri` 与 kernel 重复依赖及 `http_api` 双轨（审计结论）

以下条目作为 **独立去重 PR** 的拟定说明（不在此 PR 强制删除依赖，以免牵连链接与版本对齐）。

### 5.1 重复的直接依赖

`src-tauri/Cargo.toml` 与 `oclive_kernel_runtime` 历史上均声明（或间接固定）例如：`sqlx`、`zip`、`axum`、`reqwest`、`ed25519-dalek` 等（workspace `reqwest` 已不再启用 **`blocking`**，见 `PERF_PHASES.md` P4）。**`http_api` 与 CORS 已迁入 runtime 后，`tower-http` 已从壳层移除**（壳层不再直接依赖）。长期方向：

- 桌面逻辑优先通过 **`oclive_kernel_runtime::...` 公开 API** 访问存储与市场 / 归档能力，避免在 `src-tauri` 再挂一层同类 crate。
- 壳层独有 vs 可删（当前快照，删前仍以 `cargo check -p oclivenewnew-tauri` 为准）：

| 依赖 / 类别 | 壳层独有（保留） | 与 kernel 重叠（逐项评估删除） |
|---------------|------------------|--------------------------------|
| **`tauri` / `tauri-build` / `tauri-plugin-deep-link`** | ✅ 桌面 only | — |
| **`notify`** | ✅ 目录插件 watcher | — |
| **`sysinfo`** | ✅ 系统信息 | — |
| **`axum`** | — | ✅ 已删直连；OOCP WS 仅 **`oclive_kernel_runtime::http_api`**；壳层集成测试保留 **`dev-dependencies`** 中的 `axum`（`tests/http_api_chat.rs`）；子计划见 `handoff/LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md` |
| **`sqlx`** | — | ✅ 壳层 lib 不再直连；集成测试经 **`dev-dependencies`** 使用（与 kernel 仍可能传递重复链接，见后续是否收紧） |
| **`zip` / `sha2` / `walkdir`（打包路径）** | — | ✅ `pack_plugin` 已改为 `plugin_archive::pack_plugin_directory_to_zip_deflated` |
| **`reqwest` / `ed25519-dalek` / `base64`** | — | ✅ 壳层未引用条目已移除（HTTP/验签在 kernel） |
| **`chrono` / `uuid`** | — | ✅ 壳层已移除直连（内存 TTL 缓存改用 `std::time`；`uuid` 无引用） |
| **`tower-http`** | — | ✅ 已删（仅 `http_api` CORS 用过；现由 runtime 承担） |

### 5.2 `http_api` 双轨

**已合并（单源）**：路由与 `serve_api` / `serve_api_with_options` / `api_router` 的完整实现仅在 **`crates/oclive_kernel_runtime/src/http_api.rs`**（`kernel-http-api`）。**`src-tauri/src/http_api.rs`** 仅为 **`pub use oclive_kernel_runtime::http_api::*`**，保留 `oclivenewnew_tauri::http_api` 路径兼容。集成测试可 `use oclive_kernel_runtime::http_api::api_router`（见 `src-tauri/tests/http_api_chat.rs`）。

---

## 6. 校验命令（亦见 CI）

```bash
cargo check -p oclive_kernel_runtime --no-default-features
```

可选组合示例：

```bash
cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api
cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api,kernel-agent
cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api,kernel-agent,default-llm-providers
cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api,kernel-agent,default-memory-providers,default-emotion-providers,default-event-providers,default-prompt-providers,default-complex-emotion-providers,default-agent-providers
```

仓库脚本：`scripts/check_kernel_runtime_minimal.sh`、Windows：`scripts/check_kernel_runtime_minimal.ps1`。

### 6.1 启动分段日志（`KernelAppState`）

`KernelAppState::new` 与 `new_in_memory_with_llm_and_policy_file` 使用日志 **`target = oclive_startup`** 输出各阶段耗时（毫秒），便于 P1 对比与排障：

- 磁盘/默认路径：`phase=db_open_and_migrate_ms`、`repos_llm_cloud_chat_model_ms`、`policy_registry_ms`、`storage_directory_plugins_plugin_host_ms`、`phase=kernel_app_state_total_ms`
- 内存测试构造：`phase=test_db_open_migrate_ms`、`phase=test_storage_plugin_host_ms`、`phase=new_in_memory_total_ms`

示例：`RUST_LOG=oclive_startup=info`（若宿主已初始化 `env_logger`/`tracing`）。

### 6.2 嵌入式裁剪验证（手工基线）

除上表 `cargo check` 组合外，可补充确认无头 crate 可链接：

```bash
cargo check -p oclive_kernel_server
```

**说明**：`oclive_kernel_server` 当前默认依赖 **`oclive_kernel_runtime` 的 `full`**；若产品要「极简内核 + OOCP」，应在 **自建宿主 crate** 中声明 `oclive_kernel_runtime` 的 `default-features = false` + 子特性。Release 二进制体积与冷启动需在本机对目标三重统计（未设 CI 阈值）。

### 6.3 Linux 无头交付：全功能 vs 裁剪（按需）

| 目标 | Cargo 命令 / 依赖声明 | 说明 |
|------|----------------------|------|
| **发行版无头服务（默认）** | `cargo build -p oclive_kernel_server --release` | 等同 `oclive_kernel_runtime` 默认 **`full`**：HTTP/OOCP、ZIP 角色包、市场同步、Agent/MCP、各 `default-*-providers` |
| **仅验证 runtime 最小编译闭包** | `cargo check -p oclive_kernel_runtime --no-default-features` | 无 HTTP、无 ZIP、无 Agent；用于依赖/链接冒烟 |
| **无头 + HTTP、关市场/Agent（示例组合）** | `cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api,role-pack-zip` | 按产品删减 `market-sync` / `kernel-agent` 等；须自行承担契约与角色包能力差异 |
| **自建二进制链接裁剪 runtime** | 在自有 bin crate 的 `Cargo.toml` 中为 `oclive_kernel_runtime` 设 `default-features = false` 与 §2 所列子 `features` | 与 `oclive_kernel_server` 并行存在；适合玩偶侧车只带 OOCP + 编排 |

**体积与启动**：受目标平台、LTO、`strip`、所链设施 crate 影响极大；请在 **目标 Linux** 上对本机构建结果执行：

```bash
# 示例：比较全功能 server 二进制大小（字节数因平台而异）
ls -lh target/release/oclive_kernel_server
/usr/bin/time -f 'elapsed_sec %e' target/release/oclive_kernel_server
# 探活后 Ctrl+C；生产请配合 systemd / Docker
```

Docker 多阶段镜像中对 `oclive_kernel_server` 执行 **`strip`** 以缩小磁盘占用（见根目录 **`Dockerfile.kernel-server`**）。**不在此文档承诺固定 MB 数**，以免与发版工具链漂移；发版 gate 以 **`cargo test -p oclive_kernel_runtime --features kernel-http-api`** 与 **`cargo build -p oclive_kernel_server --release`** 为准（见 **`.github/workflows/ci.yml`** 与 **`docs/LINUX_KERNEL_DEPLOY.md`**）。

---

## 7. 与 `KERNEL_BOUNDARY.md` 的关系

发行版专属内容（深链、快捷键、目录插件 watcher、`directory_plugin_invoke` 生命周期等）仍按 `KERNEL_BOUNDARY.md` §3；轻量配置不改变「不进内核」清单，仅增加 **runtime 可选编译单元** 的取舍维度。
