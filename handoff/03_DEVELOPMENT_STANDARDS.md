# 开发流程与标准（仓库版 + 勘误）

## 对外部长文档的勘误（重要）

若你手上有另一份《开发流程与标准》（例如由其他会话生成），请先核对以下 **与当前仓库不一致** 的常见错误，再写代码：

| 问题 | 以仓库为准 |
|------|------------|
| `Emotion` 枚举变体写成 Joy / Fearful… | 见 `src-tauri/src/models/emotion.rs`：`Happy, Sad, Angry, Neutral, Excited, Confused, Shy` |
| `EmotionResult` 放在 `models/emotion.rs` | 实际在 `domain/emotion_analyzer.rs`，且已有 `to_emotion()` |
| `EventType::Breakup` 等 | 见 `src-tauri/src/models/event.rs` 实际枚举 |
| `send_message` 返回 `response` 字符串、`emotion` 字符串 | 见 `models/dto.rs`：`reply`、`emotion: EmotionDto`、多字段 |
| `Repository` 直接 `impl for DbManager` | 实际为 `SqliteMemoryRepository` / `SqliteFavorabilityRepository` 包装 `Arc<DbManager>` |
| 编排里 `FavorabilityEngine::calculate_delta` | **未实现**；当前用 `EventDetector::get_impact_factor` 等 |
| `PersonalityEngine::evolve` 单函数 | 实际为 `adjust_by_user_emotion` + `evolve_by_event` |
| Tauri 2 入口 / `generate_handler!` 无前缀 | 当前为 **Tauri 1.5**，`tauri::generate_handler![api::chat::send_message]` |
| 用 `curl` 打 Vite 的 `/api/send_message` | **默认不存在**；联调用 `invoke('send_message', …)` |

完整类型与禁止项仍以 **`02_DEVELOPMENT_PLAN_v3.8.md`** 为准。

---

## 三件事原则（必须）

1. **唯一编排**：`domain/chat_engine::process_message`（或后续拆分的编排模块）负责顺序；业务公式留在各 `*_engine` / analyzer。  
2. **唯一持久化**：SQLx + `migrations/`，业务侧通过 **Repository trait**（定义在 `domain/repository.rs`，实现在 `infrastructure/repositories.rs`）。不要在 `domain` 里直接写 `sqlx::query`（实现文件除外）。  
3. **唯一 Tauri 出口**：命令在 `api/*.rs`，**仅**在 `src-tauri/src/lib.rs` 的 `invoke_handler` 注册；保留 `.setup` 与 `AppState::manage`。

---

## 新增命令的推荐流程

1. 在 `models/dto.rs`（或新建模块）定义 Request/Response。  
2. 若需新持久化能力，扩展 **Repository trait** 与 `DbManager`/`repositories` 实现。  
3. 在 `domain` 实现纯逻辑或扩展现有编排。  
4. 在 `api/{module}.rs` 添加 `#[tauri::command]`，只做薄封装与 `map_err(|e| e.to_string())`（若保持与现有一致）。  
5. 在 `lib.rs` 的 `generate_handler!` 中追加。  
6. 补单元测试 / 集成测试；跑 `fmt`、`clippy --all-targets -D warnings`、`cargo test --tests`。

---

## 质量门禁（提交前）

```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --tests
cargo build
```

---

## 前端调用（示例）

以项目 `package.json` 中 `@tauri-apps/api` 版本为准选择导入路径（1.x 常为 `@tauri-apps/api/tauri` 的 `invoke`）。

```typescript
import { invoke } from '@tauri-apps/api/tauri';

const res = await invoke<SendMessageResponse>('send_message', {
  role_id: 'mumu',
  user_message: '你好',
  scene_id: null,
});
// res.reply, res.emotion.joy, ...
```

类型应与 `models/dto.rs` 同步（可手写 TS 或用工具生成）。
