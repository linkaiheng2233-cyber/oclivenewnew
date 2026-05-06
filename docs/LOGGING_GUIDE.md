# Oclive 内核日志与诊断指南

本文说明 **`oclive_kernel_server`** 与 **`oclive_kernel_runtime`** 的日志行为，便于生产排障与性能分析。无头服务默认通过 **`tracing-subscriber`** 输出，环境变量 **`RUST_LOG`** 仍由 **`tracing_subscriber::EnvFilter`** 解析（与常见 `env_logger` 习惯兼容）。

---

## 1. 快速开始

```bash
export RUST_LOG=info
./oclive_kernel_server
```

常用取值：`error` / `warn` / `info` / `debug` / `trace`。可按模块过滤，例如：

```bash
export RUST_LOG=oclive_process_message=info,oclive_chat_io=debug,info
```

---

## 2. 级别建议

| 级别 | 适用场景 | 内核中的典型内容 |
|------|-----------|------------------|
| **ERROR** | 仅严重故障、需告警 | 事务失败且已无法恢复、关键断言失败（与现有 `log::error!` 并存） |
| **WARN** | 降级、重试后仍异常、配置不理想 | 慢事务、远程插件回退、角色路径异常提示 |
| **INFO** | 生产默认；关键业务里程碑 | **`oclive_process_message`**：`process_message finished`（含 `elapsed_ms`、`role_id`、`scene_id`、`session_ns`、`user_len`）；启动分段 `oclive_startup` |
| **DEBUG** | 预发、单用户复现 | **`oclive_chat_io`**：单条 DB/加载步骤耗时（如 `ensure_role_runtime`、`ensure_role_loaded`、`apply_chat_turn_atomic`）；原有 `oclive_chat` 调试行 |
| **TRACE** | 本机开发 | 更细粒度内部状态（慎用，体积大） |

依赖库中仍使用 **`log`** crate 的日志，在无头进程中经 **`tracing-log`** 桥接到同一套 subscriber，故 **`RUST_LOG`** 同样生效。

---

## 3. 关键 target 一览

| target | 说明 |
|--------|------|
| `oclive_process_message` | 单次 `process_message` 总耗时（INFO，`elapsed_ms`） |
| `oclive_chat_io` | 单次对话路径上的关键 I/O 分段耗时（DEBUG） |
| `oclive_chat` | 编排过程已有 debug/info（历史行为） |
| `oclive_startup` | 数据库打开、策略、插件引导等冷启动分段 |
| `oclive_api` | HTTP 服务绑定与生命周期 |

---

## 4. 与桌面（Tauri）宿主

桌面壳若未注册 **`tracing`** subscriber，runtime 内 **`tracing::info!`** 等事件可能不可见；仅 **`log`** 仍走宿主原有配置。无头 **`oclive_kernel_server`** 已在 `main` 中初始化 subscriber，开箱即用。

---

## 5. 相关文档

- Linux 部署与 **`RUST_LOG`** 环境变量：**[LINUX_KERNEL_DEPLOY.md](./LINUX_KERNEL_DEPLOY.md)**  
- 健康检查与探活：同书「运维脚本」与 **`GET /health?verbose=true`**
