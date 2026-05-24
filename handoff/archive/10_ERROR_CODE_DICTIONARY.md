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
| `INVALID_PARAMETER` | 参数无效 |
| `SERDE_ERROR` | 序列化/反序列化错误 |
| `UNKNOWN_ERROR` | 未分类错误 |

## Alerting Baseline

- `TXN_COMMIT_FAILED` / `TXN_ROLLBACK_FAILED`: 立即告警
- 5 分钟窗口内 `TXN_*` 总数 >= 20: 警告
- 5 分钟窗口内 `TXN_SLOW_CRITICAL` >= 5: 性能告警

