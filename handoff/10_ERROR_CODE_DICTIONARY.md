# Error Code Dictionary (Backend -> Frontend)

本表用于前后端统一错误处理与告警，前端按 `src/utils/tauri-api.ts` 的映射展示中文文案。

## Transaction (TXN_*)

| Code | Meaning | Suggested Handling |
|------|---------|--------------------|
| `TXN_BEGIN_FAILED` | 事务开启失败 | 提示稍后重试；记录一次错误 |
| `TXN_RUNTIME_ENSURE_FAILED` | 运行时状态初始化失败 | 提示初始化失败；建议重载角色 |
| `TXN_PERSONALITY_INSERT_FAILED` | 性格向量写入失败 | 提示保存失败；重试 |
| `TXN_FAVORABILITY_UPDATE_FAILED` | 好感度更新失败 | 提示状态保存失败；重试 |
| `TXN_FAVORABILITY_HISTORY_INSERT_FAILED` | 好感度历史写入失败 | 提示保存失败；重试 |
| `TXN_MEMORY_INSERT_FAILED` | 长期记忆写入失败 | 提示记忆保存失败；重试 |
| `TXN_MEMORY_FIFO_TRIM_FAILED` | FIFO 修剪失败 | 提示系统繁忙；记录告警 |
| `TXN_EVENT_INSERT_FAILED` | 事件写入失败 | 提示事件保存失败；重试 |
| `TXN_FAVORABILITY_READ_FAILED` | 提交前读取好感度失败 | 提示状态读取失败；重试 |
| `TXN_COMMIT_FAILED` | 事务提交失败 | 高优先级提示，重试并告警 |
| `TXN_ROLLBACK_FAILED` | 事务回滚失败（日志） | 运维告警，人工排查 |
| `TXN_SLOW_WARN` | 慢事务（>=300ms） | 观察性能 |
| `TXN_SLOW_CRITICAL` | 严重慢事务（>=800ms） | 触发性能告警 |

## Common

| Code | Meaning |
|------|---------|
| `DB_ERROR` | 通用数据库错误 |
| `IO_ERROR` | 文件/磁盘 IO 错误 |
| `LLM_ERROR` | LLM 调用错误 |
| `ROLE_NOT_FOUND` | 角色不存在 |
| `ROLE_PACK_EXISTS` | 角色包已存在（覆盖需确认） |
| `INVALID_PARAMETER` | 参数无效 |
| `API_PERMISSION_DENIED` | 目录插件等权限拒绝（与宿主展示对齐） |
| `API_PLUGIN_NOT_FOUND` | 目录插件 id 不存在于扫描表 |
| `SERDE_ERROR` | 序列化/反序列化错误 |
| `UNKNOWN_ERROR` | 未分类错误（见下节 **P0-3**） |
| `CHAT_GENERATION_CANCELLED` | 用户取消本轮生成 |

## `AppError::Unknown`（P0-3 审计结论）

- **`oclive_kernel_runtime/src`**：业务路径 **不构造** `AppError::Unknown`；热路径错误已收敛为 `InvalidParameter` / `DatabaseError` / `TransactionError` 等可分类变体。
- **`oclive_kernel_core::error::AppError::Unknown`**：枚举变体 **保留**，用于极少数兜底与 **`crates/oclive_kernel_runtime/tests/public_api_error_contract.rs`** 对 `UNKNOWN_ERROR` / `[CODE]` 前缀的契约测试。
- **`src-tauri/src/error.rs`**：仅测试代码引用 `Unknown`，非产品路径。

新增 `AppError` 变体或前端码时，请同步本表 **Common** 段与 `to_frontend_error()` 映射。

---

## `oclive_kernel_runtime` 消息体括号前缀（便于 grep）

部分 `INVALID_PARAMETER` / `DatabaseError` 文案内嵌 `[PREFIX]`，与上表 **code 字段** 无关，仅用于日志与排障。常见前缀包括：`[PLUGIN_INSTALL_*]`、`[PLUGIN_MANIFEST]`、`[PLUGIN_INDEX_*]`、`[PLUGIN_REVIEWS_*]`、`[ROLE_INDEX_*]`、`[ROLE_PACK_*]`、`[MCP_*]`、`[PLUGIN_STATE_PERSIST]`、`[ROLE_DELETE_JOIN]` 等；完整列表以源码为准。

## Alerting Baseline

- `TXN_COMMIT_FAILED` / `TXN_ROLLBACK_FAILED`: 立即告警
- 5 分钟窗口内 `TXN_*` 总数 >= 20: 警告
- 5 分钟窗口内 `TXN_SLOW_CRITICAL` >= 5: 性能告警

