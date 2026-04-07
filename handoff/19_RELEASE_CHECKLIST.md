# 发布清单（Release Checklist）

面向发布前人工核对；与 `DEVELOPMENT_STANDARDS.md` 中的门禁命令配合使用。

## 1. 构建与门禁

- [ ] `cargo fmt`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`（含 `src-tauri/tests/` 集成测试）
- [ ] `npm run build`
- [ ] `npm run tauri:build`（完整桌面包；产物见 `src-tauri/target/release/bundle/`）

## 2. 版本与 API 说明（0.2.0）

- 应用版本号与 `package.json` / `tauri.conf.json` / `Cargo.toml` 对齐为 **0.2.0**（次版本：新增响应字段，旧客户端可忽略新字段）。
- **`send_message` 响应**新增 **`bot_emotion`**：区分用户输入情绪分析（`emotion` 七维 DTO）与角色回复情绪标签；详见根目录 **`CHANGELOG.md`**。

## 3. 已验证功能（截至文档更新时）

- 对话编排、mock/真实 LLM、事件与记忆持久化（回合内事务见 `DbManager::apply_chat_turn_atomic`）
- 角色包加载：`manifest.json`（门面与契约）+ 可选 **`settings.json`**（引擎向字段合并覆盖）、`load_role` / `get_role_info`
- `user_relation`、`event_impact_factor` 与 `set_user_relation` / `set_evolution_factor`
- 情绪条：`roles/{id}/assets/images/` 下 PNG；缺失或加载失败时 UI 回退 emoji（见 `CharacterInfo.vue`）
- 打包资源：`tauri.conf.json` 中 `bundle.resources` 含 `../roles`；启动时日志 target `oclive_roles` 打印实际角色目录

## 4. 运行环境

- **开发（推荐）**：在**仓库根目录**执行 `npm run tauri:dev`（例如 `d:\oclivenewnew`）。Tauri 的 `beforeDevCommand` 依赖当前目录能解析到 `scripts/tauri-run.cjs`，**不要在盘符根目录（如 `D:\`）执行**，否则 `npm --prefix ..` 类逻辑会指向错误路径。
- **若使用 `cargo tauri dev`**：通常在 `src-tauri` 下执行，此时请**另开终端**在仓库根运行 `npm run dev`，或先执行 `node ../scripts/tauri-run.cjs dev` 再起 Vite；更简单是统一用 **`npm run tauri:dev`**。
- **角色目录**：默认解析为仓库根 `roles/`（或环境变量 `OCLIVE_ROLES_DIR`）
- **调试角色路径**：`RUST_LOG=oclive_roles=info`（或 `debug`）查看 `resolve_roles_dir` / 打包 `resource_dir/roles`
- **Ollama**：需本机可访问；模型名默认读环境变量 `OLLAMA_MODEL`

## 5. 已知限制

- 占位情绪图为 1×1 透明 PNG（保证路径与加载链路正确）；正式发版前应替换为正式立绘
- **消息列表未实现虚拟滚动**；当单角色对话 **超过约 200 条**时可能明显卡顿，计划在后续版本（如 v0.3）优化
- **测试覆盖率 80%** 为团队目标，当前以核心路径与集成测试为主，未接 CI 覆盖率门禁
- 签名与自动更新：未接入时需手工分发安装包

## 6. 版本号

- 与 `package.json` / `src-tauri/tauri.conf.json` 中 `package.version` 保持一致
- 语义：主版本.次版本.修订号（API 或包结构不兼容时升主/次版本）

## 7. 分发（可选）

- Windows：`.msi` / `.exe` 等，路径以 `bundle/` 下实际产物为准
- 代码签名：按团队证书策略在 CI 或本机执行（本文档不绑定具体工具）

## 8. 待优化功能（backlog 摘要，与清单核对同步）

以下为产品/工程上仍值得推进的项（与 §5「已知限制」互补；**非阻塞**日常开发，按优先级自选）。

| 优先级建议 | 项 |
|------------|-----|
| 体验 | 聊天消息列表 **虚拟滚动**（长会话性能）；占位立绘换正式资源 |
| 质量 | **测试覆盖率**目标（如 80%）与可选 **CI 覆盖率门禁**；补充 **e2e**（若需要发布级信心） |
| 分发 | **代码签名**、**自动更新**链路与发布流水线 |
| 文档/创作者 | `roles/manifest.template.json`、`roles/settings.template.json` / `README_MANIFEST.md` 与 **`manifest` + `settings.json` 双文件** 规范对齐（已随仓库更新） |
| 可选增强 | 事件检测侧 **bot 情绪** 与持久化策略的进一步统一；外部/旧 handoff 与 **`04_4.6_PROJECT_TRUTH_CHECKLIST`** 冲突时以源码为准 |
