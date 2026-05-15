# A4 插件与安全边界 — 结项汇总（2026-05-15）

## 范围与结论

本迭代落实 **A4.1 可演示底线** 中的核心宿主行为：**MCP `http` / `stdio` 首次出站或子进程**、**目录插件 manifest 含 `process` 时的子进程 spawn**，均须在应用数据目录的 **`high_risk_grants.json`** 中显式授权后方可执行；否则返回机器码 **`HIGH_RISK_CAPABILITY_NOT_GRANTED`**（内核 `AppError` 与目录插件 API 映射一致），对话主路径在 `plugin_backends.* = directory` 时继续 **记日志并回退内置 / Ollama**（既有行为保留）。

**未纳入本迭代（诚实缺口）**

- **Remote 插件 HTTP / 广义 `network:*` 出站**：尚未接同一套 grant 存储；仍依赖既有 Remote 配置与网络可达性，见下阶段 issue 建议。
- **`oclive_validation` 对目录插件 manifest 的 `permissions` 枚举校验**：宿主 `OclivePluginManifest` 仍未声明 `permissions` 字段；运行时门禁以 **能力触发点**（MCP 传输、process spawn）为准，而非 manifest 白名单解析。**A4.2「校验 crate」** 记为部分完成，待 manifest 扩展后对齐。

## 实现要点（文件级）

| 区域 | 说明 |
|------|------|
| `crates/oclive_kernel_runtime/src/error.rs` | 新增 `AppError::HighRiskCapabilityNotGranted`，`code` = `HIGH_RISK_CAPABILITY_NOT_GRANTED`。 |
| `src-tauri/src/infrastructure/high_risk_grants.rs` | 持久化 `mcp_http` / `mcp_stdio` / `directory_plugin_process_spawn` 三组 id。 |
| `src-tauri/src/infrastructure/mcp_client.rs` | `list_tools` / `call_tool` 前校验授权；动态 list 失败时对 grant 错误不静默回退静态 tools。 |
| `src-tauri/src/infrastructure/directory_plugins/runtime.rs` | `ensure_rpc_url_impl` 在 spawn 前检查 `directory_plugin_process_spawn`。 |
| `src-tauri/src/domain/plugin_host.rs` / `state/mod.rs` | 构造顺序：`HighRiskGrantStore` → `DirectoryPluginRuntime::bootstrap` → `PluginHost::new`（共享同一 `Arc`）。生产 `enforce=true`，`new_in_memory*` 为 `false`。 |
| `src-tauri/src/domain/agent.rs` | 未授权 MCP 的 server 从 Agent 工具 schema 中排除（不回落到 manifest 预声明 tools）。 |
| `src-tauri/src/api/high_risk.rs` + `lib.rs` | `list_high_risk_grants` / `grant_high_risk_capability` / `revoke_high_risk_capability`。 |
| `src-tauri/src/api/error.rs` | `map_directory_rpc_url_error` 识别 spawn 未授权 → 同码 JSON。 |
| 前端 | `AgentDebugPanel.vue` + `tauri-api.ts`；`apiErrors` 中 `HIGH_RISK_CAPABILITY_NOT_GRANTED` 中英文。 |
| Remote 兜底 | `migrations/014_remote_fallback_app_setting.sql`；`remote_fallback_policy.rs`；Remote `*_http.rs` + `RemoteLlmPlaceholder`；设置页开关；`REMOTE_SERVICE_UNAVAILABLE` i18n；见 **[REMOTE_FALLBACK_POLICY_DECISION.md](./REMOTE_FALLBACK_POLICY_DECISION.md)**。 |
| CI | `.github/workflows/ci.yml` 的 OOCP job 增加 `OCLIVE_SKIP_HIGH_RISK_GRANTS=1`（与 `OCLIVE_HTTP_API_MOCK_LLM` 同类保险）。 |
| 文档 | `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md` §高风险；`handoff/PLUGIN_HIGH_RISK_ACCEPTANCE.md` 记录行。 |

## 环境变量

| 变量 | 语义 |
|------|------|
| `OCLIVE_SKIP_HIGH_RISK_GRANTS=1` | 跳过 MCP / 目录 spawn 授权检查（**仅 CI / 本地排障**；勿用于面向用户生产）。 |

## 发版清单对应关系

- **A4.1**：MCP + 目录 process spawn 路径已可演示「拒绝 → 可见错误码 / 主路径降级」；完整「弹窗」产品化可在后续将 `grant_*` 从调试面板提升为首次调用的模态流程。
- **A4.2**：文档与运行时与 **错误码 / i18n** 已对齐；**校验 crate** 待目录 manifest `permissions` 与宿主结构统一后再勾满。
