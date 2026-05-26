# Changelog

> **English mirror**: [CHANGELOG.en.md](CHANGELOG.en.md) — 用户可见变更请与中英两份同步维护。

## [Unreleased]

### Breaking

- **`high_risk_grants.json`**：仅接受规范权限键（`mcp:http`、`mcp:stdio`、`process:spawn`、`network:*`）。旧版 `mcp_http` / `directory_plugin_process_spawn` 等别名不再读取；请手动迁移文件后重授。

### Changed

- **实验性双核运行时**：`oclivenewnew-tauri` 新增 Cargo feature **`dual_core`**（默认关闭）。启用后编译 `dual_pipeline*` 并在 `role.dual_core_gated()` 时走实验核路径；`cargo build -p oclivenewnew-tauri --features dual_core`。

### Added

- **遗忘曲线与关系演化（`config.json`）**：艾宾浩斯长期记忆衰减（`memory.decay_halflife_days`）；重复提及强化（`mention_count` + `reinforcement_factor`）；沉浸模式下亲密值疏远与关系阶段降级（`relation.*`）；虚拟时间流速（`time.speed`）与首次沉浸对齐 `life_schedule` 起点；强化记忆微幅推动七维人格 / 可变档案「记忆塑造」。规范见 [ROLE_PACK_SPEC §9](creator-docs/role-pack/ROLE_PACK_SPEC.md)。

（下一发版条目写在此处。）

---

## [0.2.0] - 2026-05-22

**桌面宿主 `0.2.0`** · **`oclive-cli` `0.1.0`** · **`oclive_kernel_runtime` `0.2.0`**（独立 SemVer，见 [RELEASE_VERSIONING.md](creator-docs/development/RELEASE_VERSIONING.md)）。

### Breaking

- **角色包 v2**：新包以 **`pipeline.ocblueprint`**（`schema_version: 2`）为唯一配置中枢；`oclive pack validate` **默认 v2**。旧包迁移：[V1_TO_V2_MIGRATION.md](creator-docs/role-pack/V1_TO_V2_MIGRATION.md)。
- **CLI**：移除顶层 `publish`、`plugin search/update`、`registry login`（见 [DEPRECATED_COMMANDS.md](crates/oclive-cli/DEPRECATED_COMMANDS.md)）。

### Added

- **蓝图 v2 与架构图**：`slot_registry` / 会话 `set_session_slot_override`、写盘 **`save_role_slot_registry`**；黄金包 **`roles/mumu`** 等已迁 v2。
- **双核（Dual-core）**：`runtime_config.dual_core` + `pipeline.experimental` 实验步，失败静默降级稳定核 `co_present`（默认关）。
- **`oclive-cli` 工具链**（22 个顶层子命令）：`init`（含 **`--monolith`**）、`build`、`bench`（`--matrix` / `--cold-start` / `--soak` / `--save`）、`dev`、`pack`、`doctor`、`test --oocp`、`explain` 等；见 [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md)。
- **Monolith 焊接模式**：`init --monolith` → `build` → 双二进制 **`bench`**；[RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)。
- **HTTP `--api`**：`GET /health`、`POST /chat`；CI **OOCP 黑盒 S0–S11** + 进程重启烟测。
- **Agent / MCP**、目录插件高风险授权、插件 HTML **`OclivePluginBridge`**、市场索引安装。
- **启动自检** `startup_health`；**`oclive explain`** 全量 `AppError` 词条；**`oclive doctor`** 蓝图三项检查。
- **编排**：`TurnContext` 收敛回合参数；`AppStateBuilder` + 策略注册表拆分；滚动文件日志（`OCLIVE_LOG_DIR` / `--api`）。

### Changed

- **主编排**：Tauri 与 HTTP 均经 **`process_message`**；入口蓝图 **不再**作首轮 DSL 调度。
- **角色包格式**：`pack validate` 默认 v2（`--profile legacy` 保留旧包）；manifest/settings 顶层键白名单收紧。
- **Tauri**：`generate_handler!` 按域分组注释；移除 `reqwest` `blocking` 与 `@tauri-apps/api/fs` 直连（改自定义 command）；插件 bridge 脚本外置为前端 IIFE 资源。
- **架构图 v2**：移除手拖连线 composable（边由 `slot_registry` 派生）。
- **前端**：i18n 域拆分、`tauri-api` 模块化、Vite vendor chunk 拆分；`App.vue` 顶栏面板抽取。

### Fixed

- **错误处理**：统一 **`AppError` / `KernelErrorBody` JSON** + 前端 **`apiErrors`** 映射（含 invoke 与 HTTP 同形）。
- **SQLite**：WAL + 连接池（`sqlite_pool.rs`）；Release profile 调优（`opt-level=3`、`codegen-units=1`）。
- **并发**：内存 **`Cache`** 读锁优先 + 容量上限；角色冷加载 **`DashMap` inflight**（不再依赖 `Arc::strong_count`）。
- 插件事件订阅竞态、自定义事件被 `bridge.events` 误拦、Remote 未配置 URL 时的可见警告等。

### Performance

- Release 二进制采样约 **12 MiB PE / 7.6 MiB .text**（见 [PERFORMANCE.md](creator-docs/getting-started/PERFORMANCE.md)）。
- 目录插件 IPC in-flight 合并（catalog / bootstrap / plugin_state）；`pluginStore` 刷新与 slot memo 优化。

### Engineering

- 工作区 **`cargo clippy -D warnings`** 与 CI 对齐；共享 **`oclive_validation`**；**`invoke` 热路径**集成测 11 条。
- **`npm run check:release`** 发版闸门；Playwright **`vite preview`** 首屏（Ubuntu CI）。

### Documentation

- [COMPATIBILITY.md](creator-docs/COMPATIBILITY.md)、[PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md)、双语 **creator-docs-en** 镜像与蓝图 v2 文档收口。

---

## [0.2.0] — 2026-04-02

（0.2.x 周期内较早合入项；已包含在上列 **0.2.0** 发版说明中。）

### Added

- 大角色包导入进度：后端 `import_progress` 事件 + 前端导入进度条模态框。
- 角色包导入前预览（`manifest.json` peek）与冲突处理：当角色 ID 已存在时弹出“覆盖/取消”确认。
- 角色包导入支持 **`.zip`**（与 `.ocpak` 相同容器）以及 **已解压目录**（与 `roles/{角色id}/` 布局一致）；见 `roles/README_MANIFEST.md`。
- 场景切换欢迎语：`switch_scene` 成功后读取 `scene.json` 的 `welcome_message`（或稳定随机 monologue）并自动插入聊天区人设消息。
- 关系阶段升级提示：`send_message` 响应增加 `relation_state`，前端在“升级”时插入系统消息。

### Changed

- 虚拟滚动策略：`ChatMessageList` 在有消息时始终启用虚拟滚动（减少 DOM 压力）。
- 角色包导出命名：导出文件默认改为 `{role_name}_{version}.ocpak`（安全化文件名）。

### API

- `send_message` 响应新增 `relation_state`；`emotion` 仍表示用户输入侧七维分析。

---

## [0.1.0]

- 初始公开基线（以仓库内首次标记版本为准）。
