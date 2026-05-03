# oclive_kernel_runtime 开发者 SDK（v0.1）

面向 **库模式**（嵌入 `KernelAppState`）与 **服务模式**（`oclive_kernel_server`）的快速索引。契约以 **`models/dto.rs`**、**`KERNEL_ENTRY_CHECKLIST`**、**[OOCP_SPEC_COMPLETE_REFERENCE.md](../oocp/OOCP_SPEC_COMPLETE_REFERENCE.md)** 为准（叙述性草案见 [OOCP_SPEC_v0_1.md](../oocp/OOCP_SPEC_v0_1.md)）。

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
| crates.io / 发布顺序 / 镜像 / 桌面 | [KERNEL_AND_OOCP_RELEASE_PATH.md](../distributions/KERNEL_AND_OOCP_RELEASE_PATH.md) |
| crates.io / 发布准备 | `oclive_kernel_runtime/README.md` §crates.io |

---

## 5. 容器（无头 `kernel_server`）

- **镜像**：仓库根 **`Dockerfile.kernel-server`**（多阶段构建 `oclive_kernel_server`）。  
- **Compose**：**`docker-compose.kernel-server.yml`**（默认映射 **`48888`**，环境变量 **`OOCP_API_PORT`**）。  
- **忽略上下文**：根目录 **`.dockerignore`** 减小构建上传体积（不含前端 `src/`、随包 `roles/` 等）。

## 6. crates.io 与 `cargo publish --dry-run`

当前 **`oclive_kernel_runtime`** 依赖 **`oclive_core` / `oclive_validation` 的 path 版本**，在未为它们声明 **crates.io 版本** 并发布前，**`cargo publish -p oclive_kernel_runtime --dry-run` 会失败**（Cargo 要求发布包的所有依赖带版本约束）。CI 以 **`cargo doc`** 与 **`invoke_lists` 对齐脚本** 作为 P2 子集门禁；全量 dry-run 留在依赖链发布就绪后执行。
