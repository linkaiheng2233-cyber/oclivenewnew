# OCLive 后端（Tauri / Rust）

## 目录职责（简图）

| 路径 | 职责 |
|------|------|
| `api/` | Tauri 命令入口，参数校验与调用 domain |
| `domain/` | 业务逻辑；`chat_engine/` 为对话主编排（`mod.rs` + `context` / `scene` / `favor`） |
| `infrastructure/` | SQLite、Ollama HTTP、`LlmClient`、`llm_params`、`ollama_timeouts`、角色包与存储 |
| `models/` | 数据结构 |
| `state/` | 应用状态与路径解析 |
| `utils/` | `json_loose`（模型输出 JSON 截取）、可选 `ollama`/`emotion` 直连辅助 |

主对话 Prompt 在 **`domain/prompt_builder.rs`**（`PromptBuilder` / `PromptInput`），不在 `utils/`。

## 环境变量速查

### 引擎开关（`0` / `false` / `no` / `off` 为关，大小写不敏感）

| 变量 | 作用 |
|------|------|
| `OCLIVE_EVENT_IMPACT_LLM` | 事件类型 + 影响因子是否走 LLM（关则规则 `EventDetector`） |
| `OCLIVE_PORTRAIT_EMOTION_LLM` | 立绘情绪是否走第二次 LLM（关则启发式 + 规则纠偏） |

### 路径

| 变量 | 作用 |
|------|------|
| `OCLIVE_ROLES_DIR` | 角色资源目录覆盖（见 `lib.rs` / `state`） |
| `OCLIVE_LIST_DEV_ROLES` | 设为 `1` / `true` / `yes` / `on` 时，`list_roles` 包含 `manifest.dev_only == true` 的调试包（默认不列出） |

### LLM 采样（`infrastructure/llm_params.rs`）

| 变量 | 默认 | 作用 |
|------|------|------|
| `OCLIVE_LLM_TEMPERATURE` | `0.8` | 主对话温度 |
| `OCLIVE_LLM_TOP_P` | `0.9` | 主对话 top_p |
| `OCLIVE_LLM_TAG_TEMPERATURE` | `0.28` | 标签/短结构化输出温度 |
| `OCLIVE_LLM_TAG_TOP_P` | `0.85` | 标签任务 top_p |

### Ollama HTTP 超时（秒，`infrastructure/ollama_timeouts.rs`）

| 变量 | 默认 | 作用 |
|------|------|------|
| `OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS` | `120` | `OllamaClient` 单次请求 `reqwest` 超时 |
| `OCLIVE_OLLAMA_LEGACY_UTILS_TIMEOUT_SECS` | `30` | `utils::ollama::ollama_generate` 外包 `tokio::timeout` |

## 观测与排障

- 日志 target **`oclive_chat`**：每条 `send_message` 成功路径会打 **`send_message start`** / **`send_message ok`**，字段包含 `role_id`、`scene_id`、`main_llm_fallback`（主模型是否降级文案）、`duration_ms`、`offer_destination_picker`（是否建议前端展示选目的地条；实际切场景仅 `switch_scene`）。
- 过滤示例（依赖具体 log 实现）：在 RUST_LOG 中启用 `oclive_chat=info` 或 `info`。

## 构建与检查

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```
