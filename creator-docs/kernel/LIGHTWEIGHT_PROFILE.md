# 轻量场景 × Cargo features × OOCP / invoke 对照

> 维护：与 `KERNEL_BOUNDARY.md`、`KERNEL_API_IMPLEMENTATION_MATRIX.md` 交叉引用；文档总索引见 [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](../getting-started/DOCUMENTATION_INDEX.md)。  
> 目的：嵌入式宿主、侧车 / pack-editor、官方桌面三种形态下，明确 **runtime 特性**、**OOCP 方法**与 **Tauri invoke** 的可用边界。

---

## 1. 场景总览

| 场景 | 典型宿主 | `oclive_kernel_runtime` | HTTP / Axum | ZIP 角色包与插件归档 | 市场索引同步 | Agent / MCP |
|------|-----------|-------------------------|-------------|----------------------|--------------|-------------|
| **官方桌面** | `src-tauri`（默认依赖） | `full`（默认） | 开（`kernel-http-api`） | 开（`role-pack-zip`） | 开（`market-sync`） | 开（`kernel-agent`） |
| **kernel_server / pack-editor 试聊** | `oclive_kernel_server` | `full`（默认） | 开 | 开 | 开 | 开 |
| **嵌入式 lib / 玩偶侧车** | 自建进程，仅需 OOCP+编排 | `default-features = false` + 按需子特性 | 常关 | 常关 | 常关 | 常关 |

启用 **`tauri_invoke`** 仅在使用 `oclive_kernel_runtime::error::tauri_invoke` 将 `AppError` 映射到 `tauri::InvokeError` 的 **桌面 crate** 上需要；纯 lib / server **不要** 开启。

---

## 2. `oclive_kernel_runtime` 特性（聚合）

定义见 crate 根目录 `Cargo.toml`。

| Feature | 作用 |
|---------|------|
| **`full`**（默认） | `kernel-http-api` + `role-pack-zip` + `market-sync` + `kernel-agent` |
| **`kernel-http-api`** | Axum HTTP + OOCP WebSocket（`http_api` 模块） |
| **`role-pack-zip`** | `zip` 依赖；`plugin_archive`、`role_pack_archive`；插件 / 角色包归档安装路径 |
| **`market-sync`** | `plugin_index_sync`、`plugin_reviews_index_sync`、`role_market_index_sync` |
| **`kernel-agent`** | ReAct Agent、MCP 客户端实现、`RemoteAgentHttp`、目录 Agent HTTP 槽 |
| **`tauri_invoke`** | 可选 `tauri` 依赖，用于桌面错误映射 |

**注意**：关闭 `role-pack-zip` 时，`plugin_install` 中带解压的实现会返回明确错误；关闭 `market-sync` 时，同步函数所在模块不参与编译，由宿主（如 `src-tauri` 的 `plugin_installer` / `role_market`）保证不与该组合链接。

---

## 3. OOCP（`oclive_core::oocp_handler`）

capabilities 中的方法列表见 `OOCP_METHODS`。以下能力与 runtime 特性关系：

| 能力 | 关闭 `kernel-agent` 时的行为 |
|------|------------------------------|
| `agent.call_mcp_tool` | 适配层应返回错误（例如「Agent / MCP 未编译」）；协议仍可在 capabilities 中出现，由宿主决定是否收窄握手 |

其余 chat / role / time 等方法不依赖上述可选模块。

---

## 4. Tauri `invoke` 与前端契约

完整命令列表由 `src-tauri/src/lib.rs` 的 `tauri::generate_handler!` 注册（见 `KERNEL_API_IMPLEMENTATION_MATRIX.md`）。

### 4.1 按主题的 invoke 分组（便于 SKU / 前端对齐）

下列分组与 `src-tauri/invoke_lists/*.txt` 及 Cargo `invoke-*` 特性一致；**默认 `invoke-full` 仍为全量注册**。若本地执行了 `cargo check -p oclivenewnew-tauri --no-default-features --features tauri-app,custom-protocol`，`build.rs` 会把 `src/gen/tauri-invoke-capabilities.ts` 写成全 `false`；恢复官方前端契约可再跑一次默认的 `cargo check -p oclivenewnew-tauri`（或从版本库还原该文件）。

| 分组 | 命令示例（Rust 路径 → 前端 camelCase 参数） |
|------|---------------------------------------------|
| **Agent / MCP** | `api::agent::*` → `listMcpServers`、`callMcpTool` 等 |
| **角色包 I/O** | `api::role_pack::*`、`preview_local_plugin_archive_command`、`install_local_plugin_archive_command` |
| **角色 / 插件市场** | `api::role_market::*`、`api::plugin_index::*`、`api::plugin_reviews::*`、`api::plugin_update::*` |
| **创作者工具链** | `api::plugin_scaffold::*`、`api::plugin_pack::*`、`api::plugin_debug::*` |

### 4.2 桌面 `invoke-*` 特性与 `generate_handler!` 裁剪（已实现）

`tauri::generate_handler!` 过程宏**不能**在参数里嵌套展开子 `macro_rules!()` 片段；做法是在 `src-tauri/src/invoke_registry.rs` 的 **`oclive_invoke_handler!` 单条列表** 上，对可选命令逐条写 `#[cfg(feature = "invoke-…")]`（cfg 在过程宏之前剥离）。

- **Cargo**：`oclivenewnew-tauri` 的 `default` 包含 `invoke-full`；后者聚合 `invoke-agent`、`invoke-expert-models`、`invoke-role-market`、`invoke-plugin-market`、`invoke-plugin-creator`。极简 SKU 示例：`cargo build -p oclivenewnew-tauri --no-default-features --features tauri-app,custom-protocol`（仅核心 invoke；需同步前端能力文件，见下）。
- **前端契约**：`src-tauri/build.rs` 在带 `tauri-app` 的 `cargo build`/`check` 时重写 `src/gen/tauri-invoke-capabilities.ts`；`src/lib/tauriInvokeCapabilities.ts` 维护命令名到分组的映射；`src/utils/tauri-api.ts` 在 `invoke` 前对缺省分组给出友好错误。新增可选分组命令时须同时改 **Rust 宏列表** 与 **`COMMAND_CAPABILITY` 映射**。

---

## 5. `src-tauri` 与 kernel 重复依赖及 `http_api` 双轨（审计结论）

以下条目作为 **独立去重 PR** 的拟定说明（不在此 PR 强制删除依赖，以免牵连链接与版本对齐）。

### 5.1 重复的直接依赖

`src-tauri/Cargo.toml` 与 `oclive_kernel_runtime` 均声明（或间接固定）例如：`sqlx`、`zip`、`axum`、`tower-http`、`reqwest`、`ed25519-dalek` 等。长期方向：

- 桌面逻辑优先通过 **`oclive_kernel_runtime::...` 公开 API** 访问存储与市场 / 归档能力，避免在 `src-tauri` 再挂一层同类 crate。
- 若仍须在壳层保留 **notify / sysinfo / tauri 插件** 等内核没有的依赖，单独列出「壳层独有」清单，其余逐项核对是否可删除。

### 5.2 `http_api` 双轨

- **Runtime**：`crates/oclive_kernel_runtime/src/http_api`（由 `kernel-http-api` 控制）。
- **桌面**：`src-tauri/src/http_api`（`run_api_server` 等）。

拟定合并方向：**单一实现源在 runtime**；桌面仅保留入口函数（端口解析、`KernelAppState` 构造委托），删除或变薄重复路由层；变更需单独评审并与 pack-editor 试聊路径回归。

---

## 6. 校验命令（亦见 CI）

```bash
cargo check -p oclive_kernel_runtime --no-default-features
```

可选组合示例：

```bash
cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api
cargo check -p oclive_kernel_runtime --no-default-features --features kernel-http-api,kernel-agent
```

仓库脚本：`scripts/check_kernel_runtime_minimal.sh`、Windows：`scripts/check_kernel_runtime_minimal.ps1`。

---

## 7. 与 `KERNEL_BOUNDARY.md` 的关系

发行版专属内容（深链、快捷键、目录插件 watcher、`directory_plugin_invoke` 生命周期等）仍按 `KERNEL_BOUNDARY.md` §3；轻量配置不改变「不进内核」清单，仅增加 **runtime 可选编译单元** 的取舍维度。
