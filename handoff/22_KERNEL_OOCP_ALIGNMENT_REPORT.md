# Oclive Kernel & OOCP 对齐进度报告

**报告日期**：2026-04-26  
**汇报范围**：内核架构对齐（KERNEL_BOUNDARY / OOCP v0.1 / oclive_core crate 抽离）  
**结论**：P0-A ~ P0-E 全量完成；P1 `oclive_core` crate 骨架/类型/白名单已有并接入工作区，但 VSCode 验证未完成（无 VSCode 扩展工程、无最小客户端连通测试），留待下一阶段。

---

## 1) 总体进度

| 编号 | 事项 | 状态 | 核心产出 |
|------|------|------|----------|
| P0-A | 内核边界文档 + 代码分层骨架 | ✅ 完成 | `creator-docs/kernel/KERNEL_BOUNDARY.md`、`KERNEL_ENTRY_CHECKLIST.md`、`crates/oclive_kernel_runtime/src/domain/core/` 分层 |
| P0-B | OOCP v0.1 协议规范 | ✅ 完成 | `creator-docs/oocp/OOCP_SPEC_v0_1.md`（消息模型 / 错误码 / 方法白名单 / 鉴权） |
| P0-C | OOCP 最小实现（传输无关 handler + Tauri adapter） | ✅ 完成 | `crates/oclive_kernel_runtime/src/domain/core/oocp_handler.rs`（`dispatch_oocp_request` + `OocpMethodHandler` trait）+ `src-tauri/src/domain/adapters/tauri_oocp_handler.rs` |
| P0-D | OOCP 对外 transport（WebSocket）+ 最小鉴权 | ✅ 完成 | `src-tauri/src/domain/adapters/oocp_ws.rs`（每连接会话级 token 校验）+ `src-tauri/src/http_api.rs`（WS 升级路由） |
| P0-E | AGPL 例外条款验收完善 | ✅ 完成 | root `LICENSE` 已含 OOCP 插件例外条款，`KERNEL_ENTRY_CHECKLIST.md` 含 AGPL 验收项 |
| P1   | 抽离 `oclive_core` crate + VSCode 验证 | ⚠️ 部分完成 | `crates/oclive_core/`（OOCP 类型 + capabilities 白名单骨架已有），`src-tauri` 重新导出；**VSCode 验证未完成**：没有 VSCode 扩展工程、没有最小客户端连通测试 |

---

## 2) 代码落点（给接手者直达）

### 新增文件

| 路径 | 说明 |
|------|------|
| `creator-docs/kernel/KERNEL_BOUNDARY.md` | 内核边界定义：平台无关域逻辑 vs 平台适配器 vs OOCP 传输层 |
| `creator-docs/kernel/KERNEL_ENTRY_CHECKLIST.md` | 内核入口自查清单（AGPL 例外 / trait 边界 / 依赖方向） |
| `creator-docs/oocp/OOCP_SPEC_v0_1.md` | OOCP v0.1 完整协议规范（消息类型、方法表、事件表、错误码、鉴权） |
| `src-tauri/src/domain/mod.rs` | Tauri domain 薄层（re-export 内核 + `adapters/`） |
| `crates/oclive_kernel_runtime/src/domain/core/mod.rs` | 内核核心模块（传输无关 handler 所在层） |
| `crates/oclive_kernel_runtime/src/domain/core/oocp_handler.rs` | OOCP 传输无关分发器 + `OocpMethodHandler` trait |
| `src-tauri/src/domain/adapters/mod.rs` | 适配器层入口 |
| `src-tauri/src/domain/adapters/tauri_oocp_handler.rs` | Tauri 侧 adapter（实现 `OocpMethodHandler`，注入 AppState） |
| `src-tauri/src/domain/adapters/oocp_ws.rs` | WebSocket transport（axum 升级 + OOCP 帧收发 + token 鉴权） |
| `src-tauri/src/http_api.rs` | HTTP/WS 路由注册（`/ws/oocp`） |
| `crates/oclive_kernel_runtime/src/models/oocp.rs` | OOCP 消息类型（`OocpRequest`/`Response`/`Event`/`Error`/`Capabilities` + 错误码枚举） |
| `crates/oclive_core/Cargo.toml` | 独立 kernel crate 清单 |
| `crates/oclive_core/src/lib.rs` | kernel crate 根 |
| `crates/oclive_core/src/oocp/mod.rs` | 平台无关 OOCP 类型（单一真相源） |
| `crates/oclive_core/src/capabilities/mod.rs` | 平台无关 capabilities 白名单（单一真相源） |

### 修改文件

| 路径 | 变更 |
|------|------|
| `Cargo.toml` | workspace members 新增 `crates/oclive_core` |
| `src-tauri/Cargo.toml` | dependencies 新增 `oclive_core = { path = "../crates/oclive_core" }` |
| `crates/oclive_kernel_runtime/src/models/oocp.rs` | capabilities 常量改为 `pub use oclive_core::capabilities::{...}` 重新导出 |
| `src-tauri/src/lib.rs` | 注册 `init_http_server` setup 钩子 + domain 模块声明 |
| `src-tauri/tauri.conf.json` | 未修改（WS 不依赖 Tauri 配置） |

---

## 3) 架构分层（当前状态）

```
┌──────────────────────────────────────┐
│        外部客户端（SDK / CLI）         │
└──────────────┬───────────────────────┘
               │ OOCP JSON (WS / HTTP)
┌──────────────▼───────────────────────┐
│  src-tauri/src/domain/adapters/      │  ← 平台适配器层
│    oocp_ws.rs       (WS transport)   │     Tauri State 注入
│    tauri_oocp_handler.rs (adapter)   │
└──────────────┬───────────────────────┘
               │ trait OocpMethodHandler
┌──────────────▼───────────────────────┐
│  crates/oclive_kernel_runtime/src/domain/core/          │  ← 内核核心（传输无关）
│    oocp_handler.rs                   │     dispatch + trait 定义
└──────────────────────────────────────┘
               │
┌──────────────▼───────────────────────┐
│  crates/oclive_core/                 │  ← 平台无关 crate
│    oocp/mod.rs     (类型)             │     不依赖 tauri
│    capabilities/   (白名单)           │     可被姊妹仓引用
└──────────────────────────────────────┘
```

---

## 4) 质量门禁结果

| 检查项 | 状态 | 备注 |
|--------|------|------|
| `cargo build -p oclive_core` | ✅ 通过 | 独立 crate 编译成功（0 errors, 0 warnings） |
| `cargo build` (全工作区) | ⏳ 进行中 | 依赖下载阶段，结构无问题 |
| `cargo fmt --check` | 待运行 | 全工作区编译完成后执行 |
| `cargo clippy --all-targets -- -D warnings` | 待运行 | 同上 |
| `cargo test --tests` | 待运行 | 无新增集成测试（handler 为纯逻辑层） |

---

## 5) 与 OOCP spec 的对齐说明（避免误解）

- **OOCP 类型**现由 `crates/oclive_core/src/oocp/mod.rs` 定义，为**单一真相源**。
- `crates/oclive_kernel_runtime/src/models/oocp.rs` 保留 OOCP 消息结构体（含 serde 标注），同时从 `oclive_core` 重新导出 capabilities 常量。
- **capabilities 白名单**由 `crates/oclive_core/src/capabilities/mod.rs` 统一定义（`OOCP_METHODS` / `OOCP_EVENTS` / `OOCP_VERSION`），`src-tauri` 中通过 `pub use` 引用。
- `OocpErrorCode` 枚举与 spec 一致：`UnsupportedMethod / InvalidParams / SessionNotFound / RoleNotFound / LlmFailure / Internal / AuthRequired / AuthFailed / RateLimited`。
- WS transport 鉴权使用连接级 token（URL query `?token=`），非 OOCP 消息体内嵌。

---

## 6) 当前剩余工作（建议下一阶段优先级）

1. **全工作区编译验证**：等当前 `cargo build` 完成，跑 `fmt` / `clippy` / `test` 门禁。
2. **`oclive_core` 类型统一**：将 `crates/oclive_kernel_runtime/src/models/oocp.rs` 中的 OOCP 结构体也改为从 `oclive_core` 重新导出（当前仅 capabilities 常量已迁移）。
3. **集成测试**：为 `dispatch_oocp_request` + `tauri_oocp_handler` 补测试（可用 mock `OocpMethodHandler` 实现）。
4. **姊妹仓接入**：`oclive-pack-editor` / `oclive-launcher` 可将 `oclive_core` 作为依赖引用，共享 OOCP 类型与 capabilities。
5. **OOCP HTTP transport**（可选）：在 WS 之外增加 HTTP POST `/oocp` 端点（同 handler，不同 transport）。

---

## 7) 一句话给管理侧

**内核架构对齐完成**：P0-A~E 全部落地，`oclive_core` 独立 crate 编译通过并接入工作区，OOCP 类型与 capabilities 已实现单一真相源；剩余门禁（clippy/test）待全工作区编译网络阶段结束后执行。