# PluginHost 调度机制（开发者指南）

> **受众**：在 `oclive_kernel_runtime` / `oclive_kernel_core` 上扩展后端、接设施 crate 或理解降级链的 Rust 贡献者。  
> **契约**：角色包字段语义仍以 [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)、[`PROFILE_SCHEMA_v1.md`](./PROFILE_SCHEMA_v1.md) 为准；本文聚焦 **运行时装配与解析**。

---

## 1. 概念分层

| 层级 | 职责 | 主要类型 / 位置 |
|------|------|----------------|
| **契约与路由表** | 描述「每个模块用哪种后端」 | `PluginBackends`、`PluginBackendsOverride` — `crates/oclive_kernel_runtime/src/models/plugin_backends.rs` |
| **Trait 边界** | 模块能力接口（与实现 crate 解耦） | `MemoryRetrieval`、`UserEmotionAnalyzer`、`EventEstimator`、`PromptAssembler`、`LlmClient`、`AgentProvider`、`ComplexEmotionProvider` — `crates/oclive_kernel_core/src/*.rs` |
| **注册表** | 持有各类后端实现的 `Arc<dyn Trait>`，按枚举选实例 | `BackendRegistry` — `crates/oclive_kernel_runtime/src/domain/plugin_host.rs` |
| **解析** | 角色默认 + 会话覆盖 → 有效 `PluginBackends` → 绑定 `Arc` | `PluginResolver`、`PluginHost` — 同文件 |
| **宿主状态** | 进程级单例：DB、目录插件运行时、会话覆盖 Map | `KernelAppState` — `crates/oclive_kernel_runtime/src/state/app_state.rs` |

`PluginHost` 本身**不**执行业务编排；它只负责 **「给定 `PluginBackends`，返回一组可复用的 `Arc<dyn …>`」**。主编排仍在 [`chat_engine::process_message`](../../crates/oclive_kernel_runtime/src/domain/chat_engine/process_message.rs)。

---

## 2. 初始化链路：从 `KernelAppState` 到 `ResolvedRolePlugins`

### 2.1 进程启动：`KernelAppState::new`（节选逻辑）

1. 打开 SQLite、`DbManager`、仓储实现。  
2. 构造默认 `LlmClient`（`default-llm-providers` 特性下为 Ollama 等；关闭时用桩实现）。  
3. **`DirectoryPluginRuntime::bootstrap(roles_dir, app_data_dir)`** — 目录插件子进程运行时（可为 `PluginHost` 提供 `ensure_rpc_url`）。  
4. **`PluginHost::new(db_manager, llm, Some(directory_runtime), app_data_dir, cloud_llm_user)`**  
   - 内部调用 **`BackendRegistry::from_runtime`**：装配 builtin / remote / agent / complex_emotion / `none` 等槽位，并按 **权限位**决定是否用真实 HTTP 侧车或占位实现（见 §5）。  
5. **`bootstrap_local_plugin_providers`** — 扫描 `roles_dir/_local_plugins/`，向 `PluginHost` 注册 `memory = local` 所需的 **Local** 能力描述符。  
6. 将 `PluginHost` 存入 `KernelAppState.plugins`。

锚点：`KernelAppState::new` 中 `PluginHost::new` 与 `bootstrap_local_plugin_providers` — `app_state.rs`（约 351–361 行，以当前文件为准）。

### 2.2 单次对话：`process_message` 如何拿到 `ResolvedRolePlugins`

1. 解析会话命名空间 `srid`（与 `session_id` 推导规则一致）。  
2. **`effective_plugin_backends_for_session(role, srid)`** — 将包内 `role.plugin_backends` 与 `session_plugin_overrides[srid]` 合并（`PluginBackendsOverride::apply_to`）。  
3. **`resolved_plugins_for_session(role, Some(srid))`** → **`PluginHost::resolve_for_role_with_override`** → **`PluginResolver::resolve`**：对合并后的 `PluginBackends` 每个模块调用 `BackendRegistry` 上对应的 `*_for_plugin_backends`，得到 **`ResolvedRolePlugins`**（七个 `Arc`）。  
4. 编排代码在整轮 `send_message` 内 **复用** 同一组 `pl.*`，避免重复解析。

锚点：`process_message.rs` 中 `effective_plugin_backends_for_session` 与 `resolved_plugins_for_session`；`plugin_host.rs` 中 `PluginResolver::resolve`、`resolve_for_role_with_override`。

### 2.3 无会话覆盖时

- **`PluginHost::resolve_for_role(role)`** 等价于 `session_override: None`，即仅使用 **`role.plugin_backends`**。  
- `RoleManager` 等测试路径会构造内存 DB + `PluginHost::new(..., None, ...)` 做离线演示 — `role_manager.rs`。

---

## 3. 与 `plugin_backends` 路由表的交互

- **来源**：`Role::plugin_backends` 来自磁盘 `settings.json`（经 `RoleStorage::load_role` 等路径），字段定义见 `PluginBackends` / `DirectoryPluginSlots`。  
- **使用**：`BackendRegistry` 上的分发函数（如 `user_emotion_analyzer_for_backends`）对 **`PluginBackends` 整表** 读取对应枚举；其中 **`directory`** 分支会读取 **`directory_plugins.<slot>`** 的 manifest `id`。  
- **Local 记忆**：`memory = Local` 时还会读 **`local_memory_provider_id`**，并与 `LocalPluginRegistry` 中已注册能力匹配 — `memory_local_slot_for`。

锚点：`plugin_backends.rs`（结构体与 `apply_to`）；`plugin_host.rs` 中 `memory_retrieval_for_plugin_backends`、`user_emotion_analyzer_for_backends` 等 `match`。

---

## 4. 与 `PluginBackendsOverride`（会话覆盖）的交互

- **存储**：`KernelAppState.session_plugin_overrides: HashMap<String, PluginBackendsOverride>`，key 为会话命名空间。  
- **合并规则**：**仅 `Some` 字段覆盖包内默认值**；`directory_plugins` 为 **按槽合并**（override 某槽为空字符串则回退包内值）— `PluginBackendsOverride::apply_to`。  
- **解析入口**：`PluginHost::resolve_for_role_with_override(role, Some(&ov))` 内部先 `ov.apply_to(&role.plugin_backends)` 再选型。  
- **来源快照**：`effective_plugin_backend_sources_for_session` 用于调试/UI，标记各字段来自包默认、会话或（LLM 专用）环境变量；若你新增会话可覆盖字段，需同步考虑该快照是否要打标（当前实现以 `app_state.rs` 为准）。

锚点：`plugin_backends.rs` `PluginBackendsOverride`；`app_state.rs` `set_session_backend_override`、`effective_plugin_backends_for_session`。

---

## 5. 降级链与「builtin 不可用」时的行为

降级发生在 **`BackendRegistry` 选型内部** 或 **占位 Provider 委托**，常见几类：

### 5.1 Remote HTTP 未授权或未连接

- 在 **`from_runtime`** 中，若用户未授予 **`network:*`** 给系统 provider id（如 `system:remote_plugin_http`、`system:remote_llm_http`），则 **memory/emotion/event/prompt** 的 remote 槽位被替换为 **`*Placeholder`**（内部委托 **builtin v1** 并 `warn_once`）。  
- **LLM remote** 未授权时用 **`RemoteLlmPlaceholder`**（委托进程内默认 LLM）。  
- **Agent / complex_emotion remote** 类似：未授权时回退到 **builtin** 或 **`DegradedToBuiltinComplexEmotionProvider`**。

### 5.2 `directory` 后端但运行时或权限不足

以 **`emotion_directory_slot`** 为模板（其它模块对称）：

1. **`directory_runtime` 为 `None`** → 回退 **`emotion_builtin`**。  
2. **`directory_plugins.emotion` 缺失或空** → 回退 **`emotion_builtin`**。  
3. **`process:spawn` 未授权** → 回退 **`emotion_builtin`**。  
4. **`ensure_rpc_url` 失败** → 记录 error 日志 → 回退 **`emotion_builtin`**。

LLM 的 directory 失败时回退 **`llm_ollama`**（进程内注入的默认客户端），与其它模块「回退 builtin」略有不同 — 见 `llm_directory_slot`。

### 5.3 关闭默认设施特性（`default-*-providers`）

例如关闭 **`default-memory-providers`**：

- **`default_memory_slot_v1` / `v2`** 返回 **`DisabledMemoryRetrieval`** 桩，而不是 `oclive_memory_builtin`。  
- **Remote 占位**仍委托 **`default_memory_slot_v1()`**，因此整体仍可用但 builtin 为「轻量桩」。

锚点：`memory_retrieval.rs` 顶部 `cfg(feature = "default-memory-providers")`；`Cargo.toml` `[features]` 说明；`plugin_host.rs` `from_runtime` 与各 `*_directory_slot`。

### 5.4 日志约定

降级路径普遍使用 `target: "oclive_plugin"`，便于过滤。

---

## 6. 如何接入设施 crate（以 `oclive_memory_builtin` 为例）

1. **Trait 定义在 `oclive_kernel_core`**：如 `memory_retrieval::MemoryRetrieval`。  
2. **实现放在设施 crate**：`oclive_memory_builtin` 提供 `BuiltinMemoryRetrieval` / `BuiltinMemoryRetrievalV2`（由该 crate 的 `providers` feature 导出）。  
3. **runtime 门面**：`crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs` 中  
   - `#[cfg(feature = "default-memory-providers")]` 下 `pub use oclive_memory_builtin::{...}`；  
   - **`default_memory_slot_v1()` / `v2()`** 返回 `Arc<dyn MemoryRetrieval>`。  
4. **特性接线**：根特性 **`default-memory-providers`** 在 `oclive_kernel_runtime/Cargo.toml` 中声明为  
   `["oclive_memory_builtin/providers"]`。  
5. **注册进 `BackendRegistry`**：`from_runtime` 里 **`memory_builtin`** / **`memory_builtin_v2`** 字段直接赋值为上述 `default_memory_slot_*()`，无需再手写 `register_local_provider`（那是 **Local** 桥接专用）。

情绪、复杂情感、Agent 同理：`user_emotion_analyzer.rs`、`complex_emotion.rs`、`plugin_host.rs` 中的 `cfg` 与 `default_*_slot_*`。

---

## 7. 如何新增一种后端类型（以已有 `directory` 为范本）

> **实践建议**：第三方自定义推理逻辑优先做成 **目录插件**（`directory`）或 **HTTP 侧车**（`remote`），通常 **不必** 改内核枚举。若必须在进程内增加新枚举变体（例如 `acme`），按下列锚点改全链路。

下列步骤以 **emotion** 模块为例（当前树中 **`EmotionBackend::Directory`** 已实现，可对照阅读）。

| 步骤 | 做什么 | 代码锚点（文件） |
|------|--------|------------------|
| 1 | 扩展枚举 + serde 名 | `models/plugin_backends.rs` — `EmotionBackend` |
| 2 | 会话覆盖可选字段 | 同上 — `PluginBackendsOverride::emotion` 已是 `Option<EmotionBackend>`；若新后端需额外配置，考虑扩展 `PluginBackends` 或槽位结构 |
| 3 | 注册表持有新 `Arc` | `domain/plugin_host.rs` — `BackendRegistry` 增加字段（如 `emotion_acme`） |
| 4 | 构造时初始化 | `BackendRegistry::from_runtime` 中赋值 |
| 5 | 分发 | `user_emotion_analyzer_for_backends` 的 `match` 增加变体分支 |
| 6 | 若需从角色包读 id/URL | 参考 `emotion_directory_slot`：权限检查 + `DirectoryPluginRuntime` 或新的配置源 |
| 7 | 角色包校验 | `oclive_validation` 若校验 `plugin_backends` 枚举，需同步允许新字符串 |
| 8 | 桌面 / 前端 DTO | `src-tauri` 与 `src/utils/tauri-api.ts` 等保持 camelCase 字段一致 |

**Directory 已实现路径速查**（复制模式即可）：

- 槽位 id：`DirectoryPluginSlots::emotion`  
- 实现：`emotion_directory_slot` → `RemoteUserEmotionAnalyzerHttp` + `RemotePluginHttpConfig::for_directory_plugin_rpc`  
- 文件：`plugin_host.rs`（`emotion_directory_slot` 与 `user_emotion_analyzer_for_backends`）。

---

## 8. 最小示例：为 `emotion` 增加虚构的进程内变体 `Acme`

下面为 **说明性伪流程**；合并前需补齐错误处理、测试与校验 crate。

```rust
// 1) plugin_backends.rs — 枚举增加变体（serde 小写 acme）
// pub enum EmotionBackend { ... Acme, }

// 2) 实现 Trait（可在新 crate 或 tests 模块内）
// struct AcmeUserEmotionAnalyzer;
// impl UserEmotionAnalyzer for AcmeUserEmotionAnalyzer { ... }

// 3) plugin_host.rs — BackendRegistry
// emotion_acme: Arc<dyn UserEmotionAnalyzer>,

// 4) from_runtime 中：
// let emotion_acme: Arc<dyn UserEmotionAnalyzer> = Arc::new(AcmeUserEmotionAnalyzer);
// 并在最后的 Self { ... emotion_acme, ... }

// 5) user_emotion_analyzer_for_backends 的 match：
// EmotionBackend::Acme => self.emotion_acme.clone(),
```

若 `Acme` 实际走外部进程，更稳妥做法是 **不增加枚举**，把插件 manifest 标为 emotion 能力，令 **`plugin_backends.emotion = "directory"`** 并填写 **`directory_plugins.emotion`**。

---

## 9. 相关文档

- [KERNEL_SDK.md](./KERNEL_SDK.md) — `KernelAppState`、对外 API 轮廓  
- [HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md) — 替换模块的广义步骤  
- [EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md) — 扩展点索引  
- [MODULE_NONE_SEMANTICS.md](./MODULE_NONE_SEMANTICS.md) — `backend = none` 语义  
- [LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md) — `memory = local` 与 `_local_plugins`

---

## 10. 变更检查清单（新增后端时自测）

- [ ] `cargo check -p oclive_kernel_runtime`（按需开关 `default-features`）  
- [ ] 会话覆盖：`PluginBackendsOverride::apply_to` 对新字段行为符合预期  
- [ ] 权限：directory / remote 是否要走 `check_directory_plugin_permission` / `check_remote_http_permission`  
- [ ] 降级：失败路径是否回到文档化的安全默认（builtin / ollama / 占位）  
- [ ] 日志：`target = "oclive_plugin"` 是否足够定位
