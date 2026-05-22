# A4 插件与安全边界 — 结项汇总（2026-05-16）

## 范围与结论

**A4.1 + A4.2 已收口**：目录插件 **`manifest.json` → `permissions`**、校验 crate **`oclive_validation::plugin_permissions`**、宿主 **`high_risk_grants.json`** 与运行时门禁使用**同一套权限标识**（`process:spawn`、`network:*`、`mcp:http`、`mcp:stdio`）。规范见 [PLUGIN_V1.md §权限规范](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)。

- **MCP `http` / `stdio`**：按 server `id` 检查 `mcp:http` / `mcp:stdio`。
- **目录插件 `process` spawn**：显式 `process:spawn` 或旧版省略 `permissions` + 存在 `process` 块；须 grant `process:spawn`。
- **Remote 侧车 HTTP**（`OCLIVE_REMOTE_*`）：出站 JSON-RPC 前检查 `network:*`（grant id `remote:plugin` / `remote:llm`）。
- **目录插件 localhost RPC**（`plugin_backends.* = directory`）：spawn 授权后 HTTP 至子进程 RPC，不重复 `network:*`（与 Remote 侧车区分）。

未授权返回 **`HIGH_RISK_CAPABILITY_NOT_GRANTED`**；对话主路径在 directory 槽位失败时仍 **记日志并回退内置 / Ollama**（既有行为）。

## 实现要点（文件级）

| 区域 | 说明 |
|------|------|
| `crates/oclive_validation/src/plugin_permissions.rs` | 权限枚举、`validate_permissions_list`、`validate_directory_plugin_manifest_permissions`、`manifest_declares_process_spawn`。 |
| `creator-docs/plugin-and-architecture/PLUGIN_V1.md` | §权限规范（中英）。 |
| `src-tauri/src/infrastructure/directory_plugins/manifest.rs` | `permissions` 字段 + 加载时校验。 |
| `src-tauri/src/infrastructure/high_risk_grants.rs` | JSON 键与权限标识一致；读盘兼容旧 snake_case 键。 |
| `src-tauri/src/infrastructure/mcp_client.rs` | `mcp:http` / `mcp:stdio` capability 字符串对齐。 |
| `src-tauri/src/infrastructure/remote_plugin/*` | Remote HTTP 客户端 `network:*` 门禁；目录槽位 RPC 不传 network grant。 |
| `src-tauri/src/api/high_risk.rs` | `grant_*` / `revoke_*` 接受规范 id 与旧别名。 |
| `src-tauri/tests/permission_three_way_consistency.rs` | 三面一致集成测（9 场景）。 |
| 前端 | `AgentDebugPanel.vue`、`tauri-api.ts` grant kind / snapshot 与规范键对齐。 |

## 环境变量

| 变量 | 语义 |
|------|------|
| `OCLIVE_SKIP_HIGH_RISK_GRANTS=1` | 跳过 MCP / 目录 spawn / Remote network 授权检查（**仅 CI / 本地排障**）。 |

## 发版清单对应关系

- **A4.1**：MCP + 目录 process spawn + Remote network 路径可演示「拒绝 → 可见错误码 / 主路径降级」。
- **A4.2**：manifest / 校验 crate / 运行时 **三面一致** — 本文件与 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **A4.2** 已勾选。

## 插件管理收尾（2026-05-20，与 A4 正交）

| 项 | 状态 |
|----|------|
| 架构图 **会话覆盖** vs **写盘** | `set_session_slot_override` / `save_role_slot_registry` / `clear_session_slot_override` 已在 `ArchitectureGraphFlow` + `ArchModuleNode` 接满 |
| **V2 轻量卡片** | 读 `slot_registry_effective`；跳转 V1 架构图（`pluginStore.requestFocusArchSlot`） |
| **V1 生产路径** | 默认仍 `PluginManagerPanel` + 蓝图架构图；实验开关仅切换 V2 预览窗 |
| **A8 插件管理 a11y 切片** | 目录列表键盘导航、弹窗聚焦；见 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) A8 |
