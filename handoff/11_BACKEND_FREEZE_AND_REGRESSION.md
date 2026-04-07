# Backend Freeze And Regression (v1)

本文件用于后端“封板”阶段：冻结核心接口，明确回归门禁，保证前后端并行不互相阻塞。

## 1) Frozen APIs (only additive changes allowed)

以下命令进入冻结状态：可新增字段，不可改名/删除字段，不可改变语义。

- `send_message`
- `load_role`
- `get_role_info`
- `query_memories`
- `query_events`
- `create_event`

### Critical field contracts

- `SendMessageResponse.reply` must remain `reply` (not `response`)
- `SendMessageResponse.events[].event_type` must keep `Debug` string enum form
- `EventItem` keeps `user_emotion` and `bot_emotion` for UI context

## 2) Runtime consistency guarantees

- Chat turn writes are atomic:
  - personality_vector
  - role_runtime.current_favorability
  - favorability_history
  - long_term_memory (filtered + FIFO trim)
  - events
  - role_runtime.current_emotion
- Cache mutation happens after DB commit

## 3) Required checks before merge

In `src-tauri`:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --tests
cargo build
```

## 4) Regression checklist

- Happy path: `send_message` returns reply/emotion/events/favorability
- Invalid parameter: returns `[INVALID_PARAMETER] ...`
- Transaction failures return `[TXN_*] ...` format
- Event query returns emotion context fields (`user_emotion`, `bot_emotion`)
- Memory policy:
  - low-value chats can skip long_term_memory
  - per-role FIFO cap remains 500
- Current emotion:
  - persisted in role_runtime
  - available in `load_role` and `get_role_info`

## 5) Performance baseline

- slow transaction warning at `>=300ms` (`TXN_SLOW_WARN`)
- critical slow transaction at `>=800ms` (`TXN_SLOW_CRITICAL`)
- composite indexes:
  - `long_term_memory(role_id, created_at DESC)`
  - `events(role_id, created_at DESC)`

