# oclive_kernel_runtime 开发者 SDK（v0.1）

面向 **库模式**（嵌入 `KernelAppState`）与 **服务模式**（`oclive_kernel_server`）的快速索引。契约以 **`models/dto.rs`**、**`KERNEL_ENTRY_CHECKLIST`**、**OOCP_SPEC** 为准。

---

## 1. 库模式：持有 `KernelAppState`

- **构造**：`KernelAppState::new(db_path, roles_dir, app_data_dir).await`（SQLite 路径可为 `":memory:"`）。  
- **内存 + Mock LLM（测试）**：`KernelAppState::new_in_memory_with_llm(Arc<dyn LlmClient>, roles_dir).await`。  
- **对话入口**：`domain::chat_engine::process_message(&state, &SendMessageRequest).await` → `SendMessageResponse`（字段 **`reply`**）。  
- **特性裁剪**：见 [`LIGHTWEIGHT_PROFILE.md`](./LIGHTWEIGHT_PROFILE.md)（`default-features = false` 与子 feature）。

---

## 2. 服务模式：`oclive_kernel_server`

- **源码**：[`crates/oclive_kernel_server`](../../crates/oclive_kernel_server)。  
- **入口**：`oclive_kernel_runtime::http_api::serve_api_with_options`（`kernel-http-api`）。  
- **端口**：环境变量 **`OOCP_API_PORT`**（默认见 `kernel_server` 的 `main.rs`）。  
- **本地运行**：仓库根执行 **`scripts/run_kernel_server.sh`** 或 **`scripts/run_kernel_server.ps1`**（可选参数：端口）。

---

## 3. 错误与可诊断性

- **类型**：`error::AppError`，`code()` 与 `to_frontend_error()`（`[CODE]` 前缀）。  
- **字典**：[`handoff/10_ERROR_CODE_DICTIONARY.md`](../../handoff/10_ERROR_CODE_DICTIONARY.md)。  
- **工程路线**：[`handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md`](../../handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md)。

---

## 4. 进一步阅读

| 主题 | 文档 |
|------|------|
| 边界与模块 | [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md) |
| Tauri 命令 ↔ 实现 | [KERNEL_API_IMPLEMENTATION_MATRIX.md](./KERNEL_API_IMPLEMENTATION_MATRIX.md) |
| crates.io / 发布准备 | `oclive_kernel_runtime/README.md` §crates.io |

---

## 5. 容器（无头 `kernel_server`）

- **镜像**：仓库根 **`Dockerfile.kernel-server`**（多阶段构建 `oclive_kernel_server`）。  
- **Compose**：**`docker-compose.kernel-server.yml`**（默认映射 **`48888`**，环境变量 **`OOCP_API_PORT`**）。  
- **忽略上下文**：根目录 **`.dockerignore`** 减小构建上传体积（不含前端 `src/`、随包 `roles/` 等）。

## 6. crates.io 与 `cargo publish --dry-run`

当前 **`oclive_kernel_runtime`** 依赖 **`oclive_core` / `oclive_validation` 的 path 版本**，在未为它们声明 **crates.io 版本** 并发布前，**`cargo publish -p oclive_kernel_runtime --dry-run` 会失败**（Cargo 要求发布包的所有依赖带版本约束）。CI 以 **`cargo doc`** 与 **`invoke_lists` 对齐脚本** 作为 P2 子集门禁；全量 dry-run 留在依赖链发布就绪后执行。

---

## 7. 嵌入式最小可运行示例（复制即用）

工作区包 **`kernel_embed_minimal`**（[`examples/kernel_embed_minimal/`](../../examples/kernel_embed_minimal/)）演示：

1. `KernelAppState::new_in_memory_with_llm` + 仓库内 `roles/` 目录  
2. `SendMessageRequest`（`role_id` / `user_message` / 可选 `session_id`）  
3. `domain::chat_engine::process_message` → 打印 **`reply`**

在仓库根执行：

```bash
cargo run -p kernel_embed_minimal
```

依赖与契约：本包 `Cargo.toml` 以 **path** 引用 `oclive_kernel_runtime`；对外 DTO 与错误码仍以 **`models/dto.rs`**、[`handoff/10_ERROR_CODE_DICTIONARY.md`](../../handoff/10_ERROR_CODE_DICTIONARY.md) 为准；OOCP 见 [OOCP_SPEC_COMPLETE_REFERENCE.md](../oocp/OOCP_SPEC_COMPLETE_REFERENCE.md)。

---

## 8. HTTP 远程试聊示例（`kernel_remote_simple`）

独立脚本通过 **`GET /health`**、**`POST /chat`** 调用 **`oclive_kernel_server`**（默认端口由 **`OOCP_API_PORT`** 决定，常为 **`48888`**），无需嵌入 Rust。

- 路径：[examples/kernel_remote_simple/](../../examples/kernel_remote_simple/)  
- 说明：先 **`cargo run -p oclive_kernel_server`**，再运行 **`client.py`** 或 **`client.mjs`**（需本机 **Ollama** 与合法 **`role_path`**）。

契约与 `http_api` 一致，见 [`crates/oclive_kernel_server/README.md`](../../crates/oclive_kernel_server/README.md)。

---

## 9. 目录插件 SDK 极简示例（`kernel_directory_plugin_simple`）

最小 **manifest + Node JSON-RPC 侧车**：实现 **`echo.ping`**（手工 curl）与 **`memory.rank`**（将角色 **memory** 设为 **`directory`** 时由内核调用）。

- 路径：[examples/kernel_directory_plugin_simple/](../../examples/kernel_directory_plugin_simple/)  
- 与 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)、[REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) 联调。
