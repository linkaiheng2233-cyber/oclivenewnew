# WEEK3-004 开发计划：API 命令实现

**目标**：实现 5 个后端 API 命令（`load_role` / `get_role_info` / `query_memories` / `query_events` / `create_event`），支持前端完整交互。

**工作量**：5 个任务，每个约 0.5～1h，共约 3～5h。

**验收**：所有命令可调用，集成测试绿，门禁通过。

> **与仓库对齐**：以下「示例代码」为设计草稿；**已实现源码**以 `src-tauri/src/api/*.rs`、`models/dto.rs`、`infrastructure/db.rs` 为准。当前无独立 `EventRepository` / `role_repository` trait，事件与 runtime 通过 **`DbManager`** + **`MemoryRepository`** + **`RoleStorage`**（`AppState::storage`）完成。

---

## 前置条件

- ✅ WEEK3-003 完成（`send_message`、编排内 `save_event`、集成测试、`MockLlmClient`）
- ✅ `MemoryRepository` / `FavorabilityRepository` 已实现；事件通过 `DbManager`
- ✅ `src-tauri/migrations/001_init.sql` 表结构已定版

---

## 任务 A：load_role 命令

**目标**：加载角色配置 + 初始化 runtime，返回完整 `RoleData`。

### 代码位置（已实现）

- `src-tauri/src/api/role.rs` — `load_role`、`load_role_impl`
- `src-tauri/src/models/dto.rs` — `RoleData`
- `src-tauri/src/lib.rs` — `generate_handler!` 注册

### 要点

1. `RoleStorage::load_role(role_id)` 读取 manifest。
2. `DbManager::ensure_role_runtime` 确保 `role_runtime` 行存在。
3. `AppState::get_current_personality` 聚合缓存 / DB / 默认性格。
4. `memory_repo.count_memories`、`db_manager.count_events` 填充计数。

### 验收标准

- ✅ `tests/week3_004_api.rs` 中 `week3_004_load_role_and_get_info` 绿
- ✅ 手动：`invoke('load_role', { role_id: 'mumu' })` 返回完整字段

---

## 任务 B：get_role_info 命令

**目标**：查询角色当前状态（**不**单独初始化 runtime）；若无 `role_runtime` 行则报错，提示先 `load_role`。

### 代码位置（已实现）

- `src-tauri/src/api/role.rs` — `get_role_info`、`get_role_info_impl`
- `src-tauri/src/models/dto.rs` — `RoleInfo`
- `DbManager::role_runtime_exists`、`get_latest_memory_created_at`

### 验收标准

- ✅ `week3_004_get_role_info_before_runtime_fails` 绿
- ✅ 先 `load_role` 再 `get_role_info` 成功

---

## 任务 C：query_memories 命令

**目标**：分页查询长期记忆。

### 代码位置（已实现）

- `src-tauri/src/api/memory.rs` — `query_memories`、`query_memories_impl`
- `MemoryRepository::load_memories_paged` + `DbManager::load_memories_paged`
- `QueryMemoriesRequest`、`MemoryItem`（`memory_type` 固定 `"long_term"`）

### 验收标准

- ✅ `week3_004_query_memories_and_events` 绿
- ✅ `limit` 1～100、`offset` ≥ 0 校验

---

## 任务 D：query_events 命令

**目标**：分页查询事件历史（含 `id`、`resolution` 映射为 `description`）。

### 代码位置（已实现）

- `src-tauri/src/api/event.rs` — `query_events`、`query_events_impl`
- `DbManager::list_events_paged`、`EventListRow`
- `QueryEventsRequest`、`EventItem`

### 验收标准

- ✅ 与 `create_event`、编排 `save_event` 写入的数据一致

---

## 任务 E：create_event 命令

**目标**：手动插入事件（调试 / 测试）；`event_type` 必须为 `models/event.rs` 中 **`EventType`** 的 `Debug` 字符串（如 `Quarrel`、`Praise`…），**无** `Rejection` 变体。

### 代码位置（已实现）

- `src-tauri/src/api/event.rs` — `create_event`、`create_event_impl`、`parse_event_type`
- `DbManager::insert_manual_event` → 返回 `(id, created_at)`
- `CreateEventRequest`、`CreateEventResponse`

### 测试用例（节选，已修复）

```rust
#[tokio::test]
async fn test_create_event_invalid_type() {
    let state = AppState::new_in_memory_with_llm(llm, roles_dir()).await.expect("state");
    load_role_impl(&state, "mumu").await.expect("load_role");

    let err = create_event_impl(
        &state,
        &CreateEventRequest {
            role_id: "mumu".to_string(),
            event_type: "InvalidType".to_string(),
            description: None,
        },
    )
    .await
    .unwrap_err();
    assert!(err.contains("Invalid event_type"));
}

#[tokio::test]
async fn test_create_event_all_types() {
    let state = AppState::new_in_memory_with_llm(llm, roles_dir()).await.expect("state");
    load_role_impl(&state, "mumu").await.expect("load_role");

    for event_type in &["Quarrel", "Apology", "Praise", "Complaint", "Confession", "Joke", "Ignore"] {
        let result = create_event_impl(
            &state,
            &CreateEventRequest {
                role_id: "mumu".to_string(),
                event_type: event_type.to_string(),
                description: Some(format!("Test {}", event_type)),
            },
        )
        .await;
        assert!(result.is_ok(), "{}", event_type);
    }
}
```

### 验收标准

- ✅ `week3_004_create_event_and_query` / `week3_004_create_event_invalid_type` 绿
- ✅ 无 clippy 警告

---

## WEEK3-004 总验收

**执行命令**（在 `src-tauri/` 目录下）：

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --tests
cargo build
```

**验收清单**：

- ✅ 所有 5 个命令在 `api/*.rs` 中实现
- ✅ 所有命令在 `lib.rs::generate_handler!` 中注册
- ✅ 相关 DTO 在 `models/dto.rs` 中定义
- ✅ 集成测试 `tests/week3_004_api.rs` + 既有 `tests/chat_integration.rs`
- ✅ 无 clippy 警告（`-D warnings`）
- ✅ `cargo fmt --check` 通过

---

## 代码锚点（仓库实际路径）

| 组件 | 位置 |
|------|------|
| load_role / get_role_info | `src-tauri/src/api/role.rs` |
| query_memories | `src-tauri/src/api/memory.rs` |
| query_events / create_event | `src-tauri/src/api/event.rs` |
| DTO | `src-tauri/src/models/dto.rs` |
| 分页 / 计数 / 插入事件 | `src-tauri/src/infrastructure/db.rs` |
| MemoryRepository 扩展 | `src-tauri/src/domain/repository.rs`、`infrastructure/repositories.rs` |
| 命令注册 | `src-tauri/src/lib.rs` |
| 集成测试 | `src-tauri/tests/week3_004_api.rs` |

---

## 常见错误（禁止项）

- ❌ 命令未在 `lib.rs` 注册 → ✅ `generate_handler!` 列出全部命令
- ❌ DTO 与前端字段名不一致 → ✅ 以 `models/dto.rs` 为准
- ❌ 直接绕过 `DbManager` / Repository 在 `domain` 写 SQL → ✅ SQL 留在 `infrastructure`
- ❌ 分页不校验 limit/offset → ✅ API 层校验
- ❌ 测试缺角色目录 → ✅ 集成测试使用 `CARGO_MANIFEST_DIR/../roles`（需存在 `mumu` 等）
