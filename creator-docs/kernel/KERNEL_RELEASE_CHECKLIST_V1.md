# Kernel 发布前检查清单（对齐 KERNEL_BASELINE_V1.md）

本清单以 **[KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)** 冻结范围为基准，用于 **oclive_kernel_runtime** 与依赖它的宿主（Tauri / kernel_server）发版前自检。勾选责任在 **Release Owner**。

---

## A. API 与契约稳定性

- [ ] **OOCP**：方法清单与 payload 与文档一致；`schema_version` / `capabilities.version` 变更已记入 CHANGELOG 并评估兼容性。  
- [ ] **HTTP 试聊 API**（若启用 `kernel-http-api`）：`POST /chat` 契约未破坏（空消息 400、`reply` 字段、`session_id` 隔离）。  
- [ ] **DTO**：`src-tauri/src/models/dto.rs` 与 `oclive_kernel_runtime::models::dto` 对齐；用户可见回复字段仍为 **`reply`**。  
- [ ] **Remote JSON-RPC**：`REMOTE_PLUGIN_PROTOCOL.md` 与实现一致；新增方法已更新协议文档。  
- [ ] **manifest / settings 白名单**：`oclive_validation::json_keys` 与 `roles/README_MANIFEST.md` 同步。

---

## B. 异步化与运行时纪律

- [ ] **Tokio**：不在 async 任务中长时间阻塞；目录探测 / 重 IO 已走 `spawn_blocking` 或等价路径（与现有 handoff 约定一致）。  
- [ ] **remote_plugin_call_async**：宿主若直接调用远程 JSON-RPC，须使用合适 `reqwest` 超时与 `connect_timeout`。  
- [ ] **SQLite**：连接池与迁移在冷启动路径可重复执行且无竞态。

---

## C. 错误体系与可诊断性

- [ ] **`AppError::code()`** 与 `to_frontend_error()` 的 **`[CODE]`** 前缀一致（参见 `handoff/10_ERROR_CODE_DICTIONARY.md` 与 `tests/public_api_error_contract.rs`）。  
- [ ] **远程失败**：HTTP 状态、超时、畸形 JSON-RPC 有明确日志目标 `oclive_plugin`，且不导致 panic。  
- [ ] **Tauri 映射**（若变更）：`AppError -> InvokeError` 仍注册于 `generate_handler!`。

---

## D. 测试与质量门禁

- [ ] **本地**：`cargo fmt --all --check`、`cargo clippy --workspace -- -D warnings`、`cargo test --workspace`。  
- [ ] **文档**：`cargo doc -p oclive_kernel_runtime --all-features --no-deps` + `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links`（与 CI 一致）。  
- [ ] **关键集成测试已绿**：远程 HTTP 异常（`c_remote_plugin_http_failures`）、消息链路状态（`c_message_chain_state`）、错误信封（`public_api_error_contract`）、目录插件与权限（`p2_*` / `p3_*` 等）。  
- [ ] **手动烟测**（按需）：真实 `OCLIVE_REMOTE_PLUGIN_URL` 侧车一条 `send_message` 闭环。

---

## E. CI 与产物

- [ ] **GitHub Actions**（或等价）：Ubuntu + Windows 矩阵通过。  
- [ ] **内核 Server / Docker**（若发版）：`Dockerfile.kernel-server` 构建与 README 端口说明仍有效。  
- [ ] **许可证**：AGPL 与依赖 NOTICES 无回归（若发二进制）。

---

## F. 文档与变更日志

- [ ] **CHANGELOG.md**（根目录）：本次版本条目含 **Added / Changed / Fixed** 中与内核相关的用户可见说明。  
- [ ] **本基线**：若语义变更，已更新 `KERNEL_BASELINE_V1.md` 版本号/冻结说明或启动 v1.1 草案。  
- [ ] **DOCUMENTATION_INDEX.md**：新内核文档已链入。

---

**签核**：Release Owner _________ 日期 _________  
