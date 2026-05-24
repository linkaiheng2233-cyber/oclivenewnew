# oclivenewnew 开发报告（Cursor 交接 4.6）

**报告日期**：以仓库当前状态为准  
**范围**：**WEEK3 首轮**（编排层 + Repository + Tauri `send_message`）+ **WEEK3-003**（事件落库、Mock LLM、集成测试）  
**目的**：便于 4.6 / Opus 继续实现其余 API、前端联调与深化测试。

---

## 1. 背景与目标

- **项目**：Tauri + Vue + Rust 本地 AI 角色对话（Ollama + SQLite）。
- **本轮目标**：
  - 统一 **唯一编排入口** `domain/chat_engine::process_message`；
  - 数据访问经 **Repository trait**（domain 定义，infrastructure 实现）；
  - **唯一 Tauri 命令注册** 在 `lib.rs::run()` 的 `generate_handler!`；
  - 打通 **`send_message`**，便于前端 `invoke` 验证端到端。

---

## 2. 已完成工作清单

| 类别 | 内容 |
|------|------|
| 依赖 | `Cargo.toml`：增加 `async-trait`；`sqlx` 启用 `migrate` |
| 编排 | `src-tauri/src/domain/chat_engine/`：`process_message` 串联情绪分析、事件检测、性格演化、Prompt 构建、LLM、好感度增量、长期记忆写入；**`save_memory` 成功后 `DbManager::save_event`**（WEEK3-003） |
| Repository | 新增 `domain/repository.rs`；`infrastructure/repositories.rs`（`SqliteMemoryRepository` / `SqliteFavorabilityRepository`） |
| LLM 抽象 | `infrastructure/llm.rs`：`LlmClient` trait + `OllamaClient` 实现 + `ollama_llm()` + **`MockLlmClient`**（WEEK3-003） |
| 数据库 | `infrastructure/db.rs`：新增 `ensure_role_runtime`、`apply_favorability_delta`（更新 `role_runtime.current_favorability`，并写 `favorability_history`） |
| 情绪 | `emotion_analyzer`：`EmotionResult::to_emotion()` 委托 `EmotionAnalyzer::get_dominant_emotion` |
| 模型 | `models/dto.rs`：`SendMessageRequest` / `SendMessageResponse` / `EmotionDto` 等；`models/personality.rs`：`From<&PersonalityDefaults> for PersonalityVector` |
| 状态 | `state/mod.rs` 重构：启动时 `sqlx::migrate!("./migrations")`，注入 `db_manager`、repos、`Arc<dyn LlmClient>`、`RoleStorage::new("./roles")`，默认 Ollama 模型可经环境变量覆盖 |
| API | `api/chat.rs`：`send_message`；`api/mod.rs` 导出；`lib.rs` 注册 `api::chat::send_message` |
| 清理 | 删除未纳入构建的遗留目录 **`src-tauri/src/db`（rusqlite）**、**`src-tauri/src/commands`**（旧命令，曾依赖 rusqlite） |
| 集成测试 | **`src-tauri/tests/chat_integration.rs`**：`process_message` + 内存库 + 注入 `LlmClient`；`AppState::new_in_memory_with_llm`（WEEK3-003） |

---

## 3. 架构对齐说明（相对 v3.8）

- **EventDetector**：使用 `detect(text, user_emotion: &Emotion, bot_emotion: &Emotion)`；编排中用户情绪来自 `EmotionResult::to_emotion()`，机器人侧当前为 **`Emotion::Neutral`**（可后续改为从状态/库读取）。
- **PromptBuilder**：`build_prompt` 共 **5 个参数**，返回 **`String`**（无 `?`）。
- **Schema**：以 `src-tauri/migrations/001_init.sql` 为准；好感度更新针对 **`role_runtime.current_favorability`**，非虚构 `roles` 表。
- **Tauri**：命令仅在 **`lib.rs`** 注册；**保留** 原有 `.setup` 与 `AppState::manage`。

---

## 4. 对外 API（当前已实现）

### `send_message`

- **命令名**：`send_message`（注册路径：`api::chat::send_message`）
- **请求**（`SendMessageRequest`）：`role_id`，`user_message`，可选 `scene_id`
- **响应**（`SendMessageResponse`）：含 `api_version`、`schema`、`reply`、`emotion`（`EmotionDto`）、`favorability_delta`、`favorability_current`、`events`、`scene_id`、`timestamp`

前端示例（Tauri 1.x 风格，若 Tauri 2 请改用 `@tauri-apps/api/core`）：

```ts
import { invoke } from '@tauri-apps/api/tauri';
await invoke('send_message', { role_id: 'mumu', user_message: '你好', scene_id: null });
```

---

## 5. 环境与运行

| 变量 | 含义 | 默认 |
|------|------|------|
| `OLLAMA_BASE_URL` | Ollama 地址 | `http://localhost:11434` |
| `OLLAMA_MODEL` | 模型名 | `llama3.2` |

- 角色包目录：**`./roles`**（相对进程工作目录）；需存在如 `roles/mumu/` 与 `manifest.json`。
- 数据库：`./data/app.db`（与 `lib.rs` 中 `AppState::new` 一致）。

**质量门禁（本轮已跑通）**：

```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --tests
```

（测试数量以 `cargo test` 输出为准，文档不写死。）

---

## 6. 已知限制与后续建议（给 4.6）

1. **事务性**：`save_memory` 与 `save_event` 非同一事务；若 `save_event` 失败，可能出现记忆已写而事件未写（可按产品要求改为事务或补偿）。
2. **机器人情绪**：固定为 `Neutral`；若产品与文档要求「角色当前情绪」，需从缓存或 DB 读取并传入 `EventDetector`。
3. **其余命令**：`load_role`、`get_memory` 等仍可按 v3.8 在 `api/*.rs` 增加，并在 **`lib.rs` 同一处** `generate_handler!` 注册。
4. **前端**：Vue 侧尚未接 `invoke` 封装；建议在 `src/` 增加薄封装并与 DTO 对齐。

---

## 7. 关键文件索引（便于检索）

| 路径 | 说明 |
|------|------|
| `handoff/02_DEVELOPMENT_PLAN_v3.8.md` | 执行大纲定稿 |
| `src-tauri/src/domain/chat_engine/` | 对话编排 |
| `src-tauri/src/domain/repository.rs` | Repository trait |
| `src-tauri/src/infrastructure/repositories.rs` | SQLite 实现 |
| `src-tauri/src/infrastructure/llm.rs` | LLM trait |
| `src-tauri/src/infrastructure/db.rs` | DbManager 与 SQL |
| `src-tauri/src/state/mod.rs` | AppState、迁移、注入 |
| `src-tauri/src/api/chat.rs` | `send_message` |
| `src-tauri/src/lib.rs` | `generate_handler!` |
| `src-tauri/migrations/001_init.sql` | Schema 唯一来源 |
| `src-tauri/tests/chat_integration.rs` | 集成测试（Mock LLM、事件 DB 与响应一致） |

---

## 8. 一句话摘要（给 4.6）

**已完成**：`send_message` 全链路（编排 + Repository + Ollama + SQLite 迁移注册）、**事件落库**、**集成测试**（`MockLlmClient`），并与 v3.8 的表结构、Prompt 签名、Tauri 注册方式一致；**遗留**：更多 API、前端封装、e2e、可选事务与 bot 情绪来源。

---

*本报告由 Cursor 开发会话整理，与仓库当前代码一并交付。*
