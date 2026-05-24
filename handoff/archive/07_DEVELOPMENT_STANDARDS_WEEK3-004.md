# WEEK3-004 开发标准

**目标**：确保 WEEK3-004 的 5 个 API 命令实现质量一致、可维护性高、测试覆盖完整。

---

## 代码规范

### API 命令设计

#### 1. 命令签名规范

```rust
#[tauri::command]
pub async fn command_name(
    param1: Type1,
    param2: Type2,
    state: State<'_, AppState>,
) -> Result<ResponseType, String> {
    // 实现
}
```

**规则**：
- 所有命令必须是 `async`
- 最后一个参数必须是 `state: State<'_, AppState>`
- 返回类型必须是 `Result<T, String>`（错误统一用 String）
- 参数顺序：业务参数 → state

#### 2. 错误处理规范

```rust
// ✅ 正确
let result = repository
    .query(...)
    .await
    .map_err(|e| format!("Failed to query: {}", e))?;

// ❌ 错误
let result = repository.query(...).await?;  // 不清晰
```

**规则**：
- 所有 `.map_err()` 必须包含上下文信息
- 错误消息格式：`"Failed to [操作]: [原因]"`
- 不允许 panic（使用 Result）

#### 3. 参数验证规范

```rust
// ✅ 正确
if req.limit <= 0 || req.limit > 100 {
    return Err("limit must be between 1 and 100".to_string());
}

// ❌ 错误
// 无验证
```

**规则**：
- 所有分页参数（limit / offset）必须验证
- limit 范围：1～100
- offset 范围：>= 0
- event_type 必须在允许列表中

#### 4. 状态管理规范

```rust
// ✅ 正确
let runtime = state
    .role_repository
    .get_or_create_runtime(&role_id)
    .await
    .map_err(|e| format!("Failed to initialize role runtime: {}", e))?;

// ❌ 错误
let runtime = state.role_repository.get_runtime(&role_id).await?;  // 无错误上下文
```

**规则**：
- 所有 state 操作必须通过 Repository trait
- 不允许直接访问 state 的内部字段
- 所有异步操作必须 `.await`

> **与仓库对齐**：当前实现使用 `AppState::storage`、`MemoryRepository`、`DbManager`；若文档示例出现 `role_repository` / `EventRepository`，以 `src-tauri/src/state/mod.rs` 与 `infrastructure/db.rs` 为准。

---

## DTO 设计规范

### 1. 命名规范

| 类型 | 命名 | 示例 |
|------|------|------|
| 请求 | `{Command}Request` | `LoadRoleRequest` / `QueryMemoriesRequest` |
| 响应 | `{Command}Response` 或 直接用数据类型 | `CreateEventResponse` / `RoleData` |
| 数据项 | `{Entity}Item` | `MemoryItem` / `EventItem` |

### 2. 字段规范

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleData {
    pub role_id: String,           // 必须有 role_id
    pub name: String,
    pub description: String,
    pub personality_vector: Vec<f64>,  // 7 维
    pub current_favorability: f64,
    pub current_emotion: String,   // 枚举值的字符串表示
    pub memory_count: i32,
    pub event_count: i32,
}
```

**规则**：
- 所有 DTO 必须 derive `Debug, Clone, Serialize, Deserialize`
- 所有字段必须是 `pub`
- 时间戳统一用 ISO 8601 字符串（`String`）
- 枚举值用字符串表示（便于前端）
- 数值类型：整数用 `i32` / `i64`，浮点用 `f64`

### 3. 可选字段规范

```rust
// ✅ 正确
pub description: Option<String>,
pub last_interaction: Option<String>,

// ❌ 错误
pub description: String,  // 应该是 Option
```

**规则**：
- 可能为空的字段必须用 `Option<T>`
- 不允许用空字符串表示无值

---

## 测试规范

### 1. 单元测试结构

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Happy path
    #[tokio::test]
    async fn test_command_success() {
        let state = AppState::new_in_memory_with_llm(MockLlm::new()).await;
        let result = command(..., State::new(state)).await;
        
        assert!(result.is_ok());
        // 验证返回值
    }

    // 边界情况
    #[tokio::test]
    async fn test_command_invalid_param() {
        let state = AppState::new_in_memory_with_llm(MockLlm::new()).await;
        let result = command(invalid_param, State::new(state)).await;
        
        assert!(result.is_err());
    }

    // 不存在的资源
    #[tokio::test]
    async fn test_command_not_found() {
        let state = AppState::new_in_memory_with_llm(MockLlm::new()).await;
        let result = command("nonexistent", State::new(state)).await;
        
        assert!(result.is_err());
    }
}
```

**规则**：
- 每个命令至少 3 个测试：happy path + 边界 + 错误
- 使用 `#[tokio::test]` 标记异步测试
- 使用 `AppState::new_in_memory_with_llm(MockLlm::new())` 创建测试状态
- 断言必须清晰（`assert!(result.is_ok())` 或 `assert_eq!(...)`）

> **与仓库对齐**：Mock 类型名为 `MockLlmClient`；集成测试可调用 `*_impl(&AppState, ...)` 见 `tests/week3_004_api.rs`。

### 2. 集成测试结构

```rust
// 在 tests/chat_integration.rs 中

#[tokio::test]
async fn test_load_role_after_send_message() {
    let state = setup_test_state().await;

    // 1. 先 send_message
    let send_req = SendMessageRequest { ... };
    let _ = send_message(send_req, State::new(state.clone())).await;

    // 2. 再 load_role
    let result = load_role("mumu".to_string(), State::new(state)).await;

    assert!(result.is_ok());
    let role_data = result.unwrap();
    assert_eq!(role_data.memory_count, 1);
}
```

**规则**：
- 集成测试验证命令间的交互
- 使用 `setup_test_state()` 初始化共享状态
- 测试名称格式：`test_{command1}_{command2}_interaction`

### 3. 测试覆盖清单

| 命令 | Happy Path | 边界 | 错误 | 集成 |
|------|-----------|------|------|------|
| load_role | ✅ | ✅ | ✅ | ✅ |
| get_role_info | ✅ | ✅ | ✅ | ✅ |
| query_memories | ✅ | ✅ (pagination) | ✅ | ✅ |
| query_events | ✅ | ✅ (empty) | ✅ | ✅ |
| create_event | ✅ | ✅ (all types) | ✅ | ✅ |

---

## 质量门禁

### 1. 每周五执行

**命令**（在 `src-tauri/` 目录下）：

```bash
# 1. 格式检查
cargo fmt --check

# 2. Lint 检查
cargo clippy --all-targets -- -D warnings

# 3. 单元测试
cargo test --lib

# 4. 集成测试
cargo test --tests

# 5. 编译
cargo build
```

### 2. 失败处理

| 失败项 | 处理方式 |
|--------|---------|
| `fmt` | 运行 `cargo fmt` 自动修复 |
| `clippy` | 修改代码或添加 `#[allow(...)]`（需注释说明） |
| `test --lib` | 修复测试或代码 |
| `test --tests` | 修复集成测试或代码 |
| `build` | 检查依赖版本或编译错误 |

### 3. 门禁通过标准

- ✅ `cargo fmt --check` 无输出
- ✅ `cargo clippy --all-targets -- -D warnings` 无警告
- ✅ `cargo test --lib` 所有测试绿
- ✅ `cargo test --tests` 所有测试绿
- ✅ `cargo build` 编译成功

---

## 代码审查清单

### 实现检查

- [ ] 所有 5 个命令已实现
- [ ] 所有命令在 `api/*.rs` 中
- [ ] 所有命令在 `lib.rs::generate_handler!` 中注册
- [ ] 所有 DTO 在 `models/dto.rs` 中
- [ ] 所有参数已验证
- [ ] 所有错误已处理（`.map_err()` 包含上下文）
- [ ] 所有异步操作已 `.await`

### 测试检查

- [ ] 每个命令至少 3 个单元测试
- [ ] 至少 5 个集成测试
- [ ] 所有测试绿（`cargo test --lib` / `cargo test --tests`）
- [ ] 测试覆盖 happy path + 边界 + 错误

### 质量检查

- [ ] 无 clippy 警告（`cargo clippy --all-targets -- -D warnings`）
- [ ] 代码格式正确（`cargo fmt --check`）
- [ ] 编译成功（`cargo build`）
- [ ] 无 panic（所有错误用 Result）
- [ ] 错误消息清晰

### 文档检查

- [ ] 每个命令有注释说明功能
- [ ] 复杂逻辑有行内注释
- [ ] DTO 字段有说明

---

## 常见问题

### Q1：为什么所有错误都用 String？

**A**：Tauri 命令的返回类型限制。前端接收到错误时是字符串。如需更复杂的错误类型，可在 DTO 中定义 `ErrorResponse` 结构体。

### Q2：为什么要用 `Option<T>` 而不是空字符串？

**A**：类型安全。`Option<T>` 强制前端处理无值情况，避免 bug。

### Q3：为什么 limit 最大 100？

**A**：防止一次查询过多数据导致性能问题。可根据实际需求调整。

### Q4：为什么要在 API 层验证参数？

**A**：防止无效请求进入业务逻辑，提高错误处理的清晰度。

---

## 一句话摘要

**WEEK3-004 标准**：5 个 API 命令（load_role / get_role_info / query_memories / query_events / create_event）集中在 `api/*.rs`，所有 DTO 在 `models/dto.rs`，每个命令 3+ 单元测试 + 集成测试，门禁（fmt / clippy / test / build）全绿。
