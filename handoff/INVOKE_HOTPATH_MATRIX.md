# 高流量 `invoke` 对照表（A1.2）

**用途**：把 Tauri **`generate_handler!`** 中的高频命令与 **Rust 侧入口**、**烟测** 对齐，便于回归与后续扩写 golden。**不替代**各 `*_impl` 的单元语义；完整 IPC 仍由 Tauri 运行时绑定。

**权威命令表**：[`../distros/desktop-tauri/src/lib.rs`](../distros/desktop-tauri/src/lib.rs)（`invoke_handler` 列表）。

**集成测**：[`../distros/desktop-tauri/tests/invoke_hotpath_matrix.rs`](../distros/desktop-tauri/tests/invoke_hotpath_matrix.rs)  
命令：`cargo test -p oclivenewnew-tauri --test invoke_hotpath_matrix`

---

## 宿主热路径（已由 `invoke_hotpath_matrix` 单测串联）

| `invoke` 名（camelCase 见前端） | Rust 命令 / impl | 说明 |
|--------------------------------|------------------|------|
| `list_roles` | `list_roles_impl` | 角色列表 |
| `load_role` | `load_role_impl` | 加载角色 |
| `get_role_info` | `get_role_info_impl` | 角色信息 |
| `get_display_metrics` | `get_display_metrics_impl` | 只读 affect 快照 + 置 `radar_deep_pending` |
| `get_time_state` | `get_time_state_impl` | 虚拟时间 |
| `send_message` | `process_message` | 主编排 |
| `query_memories` | `query_memories_impl` | 记忆查询 |
| `get_directory_plugin_catalog` | `get_directory_plugin_catalog_impl` | 目录插件 catalog（含 5s 指纹缓存） |
| `get_plugin_state` | `get_plugin_state_impl` | `plugin_state.json` 角色态 + 全局默认 |
| `get_hotkey_bindings` | `get_hotkey_bindings_impl` | `hotkey_bindings.json`（与 `HotkeyBindingsFile::load` 缺省行为一致） |
| `set_session_slot_override` | `set_session_slot_override_impl` | 蓝图 v2 会话槽覆盖（`slot_registry_effective`） |
| `save_role_slot_registry` | `save_role_slot_registry_impl` | 架构图写盘 `pipeline.ocblueprint`（集成测见 `save_role_slot_registry.rs`） |
| `switch_scene` | `switch_scene_impl` | 切换 `user_presence_scene` |
| `list_high_risk_grants` | `list_high_risk_grants_impl` | 高风险授权快照 |
| `grant_high_risk_capability` / `revoke_high_risk_capability` | `*_impl` | 授权 / 撤销 |

---

## 仍属后续增强（不挡 A1.2 本切片收口）

| 方向 | 说明 |
|------|------|
| **全命令表** | `lib.rs` 中其余 `invoke` 按需再挂矩阵行 |
| **golden / 对照 JSON** | 对关键 DTO 做固定快照时另开任务，避免与目录 mtime 指纹 flake 纠缠 |
| **IPC 层** | 真 Tauri 窗 / WebDriver / 安装包全屋 E2E 归入发版表 **A1.1c**（另立项） |

---

## 相关

- [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) §四 **A1.2**  
- [week3_004_api.rs](../distros/desktop-tauri/tests/week3_004_api.rs)（更宽 API 覆盖）  
- [plugin_backends_v2_resolve.rs](../distros/desktop-tauri/tests/plugin_backends_v2_resolve.rs)
