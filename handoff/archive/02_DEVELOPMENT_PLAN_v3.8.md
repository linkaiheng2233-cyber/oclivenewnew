# oclivenewnew 项目开发计划 v3.8（最终定稿版）

### 项目背景
**oclivenewnew** 是 Tauri + Vue + Rust 本地 AI 角色对话应用。**WEEK2 完成**（Models + Infrastructure + Domain 层全部实现，单元测试全绿）。**下一步**：WEEK3-4 实现 API 层和前端基础。

---

## 核心架构决策（10 条 ADR）

### 三件事原则
1. **唯一编排入口**：`domain/chat_engine::process_message()` 只做流程编排，通过注入的 Repository trait 访问数据
2. **唯一持久化**：只用 SQLx + migrations/，所有数据访问通过 Repository trait
3. **唯一 Tauri 出口**：所有命令在 `api/{module}.rs` 中定义，唯一注册点在 `lib.rs::run()` 的 `generate_handler!`

### 其他关键决策
- **Repository Pattern**：trait 在 domain，实现在 infrastructure，AppState 注入 `Arc<dyn ...>`
- **LlmClient trait**：OllamaClient + 三种 mock
- **DTO 定版**：前后端共用，包含 `api_version` 和 `schema`
- **集成测试**：无 Tauri 运行时，使用 mock，5+ 个测试
- **质量门禁**：每周 `cargo fmt/clippy/test/build` 全绿

---

## 5 个硬错误修正（与真实仓库对齐）

### 1. EventDetector 参数类型 ✅
**正确**：`detect(text: &str, user_emotion: &Emotion, bot_emotion: &Emotion)`（`models::emotion::Emotion` 枚举）

**编排层映射函数**（在 `domain/emotion_analyzer.rs`）：
```rust
// 优先复用仓库中已有的 EmotionAnalyzer::get_dominant_emotion
// 若需封装 EmotionResult::to_emotion()，应委托给该函数
impl EmotionResult {
    pub fn to_emotion(&self) -> Emotion {
        EmotionAnalyzer::get_dominant_emotion(self)
    }
}
```

**或编排层直接调用**：`EmotionAnalyzer::get_dominant_emotion(&result)`（二选一，避免重复）

---

### 2. 数据库 Schema（以真实迁移为准）✅
**真实 `role_runtime` 表结构**（`src-tauri/migrations/001_init.sql`）：
```
role_id, current_favorability, current_scene, last_interaction_at, created_at, updated_at
```

**注**：**无** `current_personality_json` 列；性格向量在 **`personality_vector` 表**（或内存/缓存），不在 `role_runtime`。

**其他表**：
- `long_term_memory`：`id INTEGER AUTOINCREMENT`, `importance REAL`（**无** `memory_type`）
- `favorability_history`：`delta REAL`（**不是** `value`）

---

### 3. 好感度更新目标表 ✅
**正确**：更新 **`role_runtime.current_favorability`**

**SQL 示例**：
```sql
UPDATE role_runtime 
SET current_favorability = current_favorability + ? 
WHERE role_id = ?
```

**Repository 方法**：
```rust
pub async fn apply_delta(
    &self,
    role_id: &str,
    delta: f32,
) -> Result<()> {
    // UPDATE role_runtime SET current_favorability = current_favorability + delta
}
```

---

### 4. PromptBuilder 返回类型 ✅
**正确**：返回 **`String`**，不是 Result

**完整调用**（5 个参数）：
```rust
let emotion_str = emotion.to_string();  // Emotion 已实现 Display
let prompt = PromptBuilder::build_prompt(
    &role,
    &personality_vector,
    &memories,
    user_message,           // &str
    &emotion_str,           // &str（从 Emotion 转换）
);
// 无 ? 操作符
```

---

### 5. Role vs PersonalityVector ✅
**正确**：`Role`（manifest）**无** `personality` 字段；从**运行时状态 / DB** 取 `PersonalityVector`

**编排层正确做法**：
```rust
let personality_vector = state.get_current_personality(&role.id).await?;
let emotion_str = user_emotion.to_string();
let prompt = PromptBuilder::build_prompt(
    &role,
    &personality_vector,
    &memories,
    user_message,
    &emotion_str,
);
```

---

## 集成测试修正

### Emotion 枚举映射
```rust
// ✅ 正确（用完整构造，EmotionResult 当前无 Default）
let result = EmotionResult {
    joy: 0.8,
    sadness: 0.0,
    anger: 0.0,
    fear: 0.0,
    surprise: 0.0,
    disgust: 0.0,
    neutral: 0.2,
};
let emotion = result.to_emotion();
assert_eq!(emotion, Emotion::Happy);  // 或其他真实枚举变体
```

**注**：若实现阶段为 `EmotionResult` 增加 `Default` trait，可改用 `EmotionResult::default().to_emotion()`；此处按当前仓库状态示例。

### EmotionResult 字段访问
```rust
// ✅ 正确（对 EmotionResult 断言）
let result = EmotionAnalyzer::analyze("我很开心！").unwrap();
assert!(result.joy > 0.5);
```

### EventType 比较
```rust
// ✅ 正确（用仓库中已有的枚举变体）
assert_eq!(event.event_type, EventType::Quarrel);
// 注：EventType 中已有 Quarrel、Apology 等，不用虚构 Breakup
```

### 缺失 API 标记
- **`FavorabilityEngine::calculate_delta`**：仓库尚无此方法，标为「待实现」或改用现有 API
- **`PersonalityEngine::calculate_delta`**：同上

### 测试数量
以 `cargo test` 输出为准，**不写死数字**。

---

## WEEK3-4 开发计划（可执行版）

### WEEK3-001：架构收敛第 1 步
- 删除 rusqlite 路径（方案 B：开发阶段删库重建）
- 定版 DTO（EmotionDto 结构体，不是 `[f32;7]`）
- 补充情绪映射（复用 `EmotionAnalyzer::get_dominant_emotion`，或封装 `EmotionResult::to_emotion()`，二选一）
- 定义 Repository trait（好感度用 `apply_delta`）

### WEEK3-002：架构收敛第 2 步
- 实现 Repository trait（按真实表名和列名：`role_runtime`, `long_term_memory`, `favorability_history`）
- 集中注册 Tauri 命令
- 实现 LlmClient trait
- 注入 Repository 到 AppState

### WEEK3-003：send_message 打通
- 实现 `domain/chat_engine::process_message()`（按真实 API 签名，5 参数 `build_prompt`，第 5 参数为 `&str`）
- 实现 `api/chat.rs::send_message`（薄封装）
- 编写集成测试（5+ 个，用真实 API 和类型，EventType 用 `Quarrel` 等已有变体，EmotionResult 用完整构造）

### WEEK3-004 ~ WEEK3-012：其他 API 命令 + 前端
- 实现其他 API 命令
- 前端 Tauri 命令封装
- 前后端联调

---

## 禁止项清单

**架构禁止**
- 业务逻辑散落在 commands/* 和 domain/ 并存
- 两套 schema 真相
- domain 直接调 infrastructure（应通过 Repository trait）

**代码禁止**
- 混淆 `EmotionResult` 和 `Emotion` 枚举
- 用虚构表名（`roles`），应用真实表名（`role_runtime`）
- 好感度操作用 `update`，应用 `apply_delta`
- `build_prompt` 加 `?`，应去掉
- 用 `role.personality`，应从 state 取 `PersonalityVector`
- **`Emotion::Joy`，应用 `Emotion::Happy` 等真实枚举变体**
- **`role_runtime` 写 `current_personality_json`，应在 `personality_vector` 表**
- **`build_prompt` 第 5 参数用 `&Emotion`，应用 `&str`（从 `Emotion::to_string()` 转换）**
- **`EventType::Breakup`，应用 `EventType::Quarrel` 等仓库中已有变体**
- **`EmotionResult::default()`，当前无 `Default` 实现，测试用完整构造**
- **创建新的情绪映射逻辑，应复用 `EmotionAnalyzer::get_dominant_emotion`**

---

## 关键文件位置

- **编排层**：`src-tauri/src/domain/chat_engine/`
- **情绪映射**：`src-tauri/src/domain/emotion_analyzer.rs`（复用 `get_dominant_emotion`）
- **Repository trait**：`src-tauri/src/domain/repository.rs`
- **Repository 实现**：`src-tauri/src/infrastructure/db.rs`
- **API 命令**：`src-tauri/src/api/{module}.rs`
- **Tauri 注册**：`src-tauri/src/lib.rs::run()`
- **数据库迁移**：`src-tauri/migrations/001_init.sql`（真实 schema 来源）
- **DTO 定义**：`src-tauri/src/models/dto.rs`
- **集成测试**：`tests/integration_tests.rs`

---

**一句话总结**：v3.8 与仓库对齐，内部自洽；5 处硬错误已修正，集成测试示例已按当前仓库状态调整，可作「给 Opus 的执行大纲」最终定稿交付。
