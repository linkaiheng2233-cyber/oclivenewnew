# OOCP 规范 — 完整参考索引

> **定位**：对外链与 CI/测试文档的稳定锚点（「完整参考」入口）。**正文权威**仍以下列文件与 Rust 类型为准；本页不重复定义字段，仅做导航与一致性提示。

## 主规范（正文）

| 文档 | 说明 |
|------|------|
| **[OOCP_SPEC_v0_1.md](./OOCP_SPEC_v0_1.md)** | 消息信封、`capabilities`、v0.1 方法清单、错误模型、示例 JSON |
| **[OOCP_TRANSPORTS.md](./OOCP_TRANSPORTS.md)** | WebSocket / HTTP 等传输注意点 |
| **[OOCP_FREEZE_POLICY.md](./OOCP_FREEZE_POLICY.md)** | 版本冻结与兼容策略 |

## 与实现对齐（单一真相源）

- **方法名与参数解析**：`crates/oclive_core/src/oocp_handler.rs`（`handle_method`）
- **capabilities 白名单**：`crates/oclive_core/src/capabilities/mod.rs`（`OOCP_METHODS` / `OOCP_EVENTS` / `OOCP_VERSION`）
- **对话等业务 DTO**：`crates/oclive_kernel_runtime/src/models/dto.rs`（例如 `SendMessageResponse` 的 **`reply`** 字段）

## 协议级测试

| 文档 | 说明 |
|------|------|
| **[OOCP_TEST_SUITE.md](./OOCP_TEST_SUITE.md)** | 标准化测试场景（HTTP 探活 + OOCP WebSocket 方法） |
| **参考实现**：[`examples/oocp-test-suite/README.md`](../../examples/oocp-test-suite/README.md) | Node.js 可执行脚本（CI 与本地） |

## 无头内核入口

- **`oclive_kernel_server`**：`GET /health`、`GET /oocp`（WebSocket）、REST 试聊等 — 见 [`crates/oclive_kernel_server/README.md`](../../crates/oclive_kernel_server/README.md)
