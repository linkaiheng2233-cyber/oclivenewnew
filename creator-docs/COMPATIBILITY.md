# 编写器（oclive-pack-editor）与主程序（oclivenewnew）版本兼容说明

本文档说明 **角色包内 `ui.json`** 与 **主程序** 的兼容关系，避免「编写器导出的字段主程序不认识」或「主程序已支持但编写器未导出」的困惑。

**版本号格式**：两项目均采用 **语义化版本（SemVer）** `MAJOR.MINOR.PATCH`，见各仓库根目录 **`package.json`** 的 **`version`** 字段。

**当前仓库快照（文档更新时 · 与发版审阅对齐）**：

- **oclivenewnew**（主程序 / Tauri 宿主）：**`0.5.0`**（根 `package.json` `version` 与 `distros/desktop-tauri/Cargo.toml` `version` 须一致）
- **oclive_kernel_runtime**（共享契约 crate）：**`0.2.0`**（`kernel/crates/oclive_kernel_runtime/Cargo.toml`；DTO / `API_VERSION` 等见该 crate）
- **oclive-cli**（脚手架 CLI）：**`0.1.0`**（`kernel/crates/oclive-cli/Cargo.toml`；**独立 semver**，不强制与桌面宿主同号；`init --kernel-source` 接主仓时以 path 依赖对齐契约）。**默认构建**仅依赖 `oclive_kernel_runtime` + `oclive_validation`（`cargo tree -p oclive-cli --no-default-features` **无** `libsqlite3-sys` / `axum`）。**`doctor config-resolve`** 默认走 runtime 纯解析；**`--via-host`**（feature `diagnostics-host`）可选 in-memory `AppState` 深度诊断。
- **oclive-pack-editor**（编写器，姊妹仓）：**`0.5.0`**（该仓 `package.json`；与主程序 **0.5.x** 对拍 `ui.json`）
- **oclive-vscode**（VS Code 扩展，姊妹仓）：**`0.4.1`**（独立 semver；spawn/attach 契约对齐主程序 **≥0.4.0**，推荐 **0.5.0**）

---

## 兼容性表

| 编写器版本 | 主程序最低版本 | 新增或强依赖的 `ui.json` 能力 | 备注 |
|------------|----------------|--------------------------------|------|
| **0.2.x** | **0.2.0** | `shell`、`slots`（`chat_toolbar`、`settings_panel`、`role_detail` 等）、基础 `theme` / `layout`（以 schema 为准） | 历史基线 |
| **0.3.x** | **0.3.0** | schema 扩展 **主题/布局** 细分字段（以发版说明为准） | 主程序较低版本可能 **忽略未知字段** |
| **0.4.x** | **0.4.0** | **`sidebar`、`chat.header`** 等插槽在编写器中完整配置时，需主程序 **Directory 插件引导** 已支持对应插槽（见 [DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)） | 插槽名与宿主 `pluginStore` 常量一致 |
| **0.5.x** | **0.5.0** | 立绘 catalog / `visual_presentation` 导出与主程序 `display_metrics`、语音侧通道 `ui.json` 插槽种子对齐 | 见 [CHANGELOG.md](../CHANGELOG.md) `[0.5.0]` |
| **开发版** | **同开发版** | schema 与主程序 `UiConfig` 同分支 | 仅建议开发者本地对拍 |

---

## 升级与降级行为

1. **主程序版本低于编写器目标**  
   - **`ui.json`** 中主程序 **不认识的字段**：若 Rust/TS 模型使用 **`serde` 默认 + 可选字段**，通常 **静默忽略**；若某版本改为 **拒绝未知字段**，以该版本 `CHANGELOG` 为准。  
   - **已声明但宿主未实现的插槽**：该插槽在 UI 中可能 **不显示** 或 **无操作**，需升级主程序。

2. **编写器版本低于主程序**  
   - 主程序 **新插槽 / 新主题键** 可能无法在旧编写器中编辑；可 **手动编辑 `ui.json`** 并参照 [ui.json.schema.json](role-pack/ui.json.schema.json)。

3. **角色包 `settings.json` 与 `plugin_backends`**  
   - 兼容性与 **`min_runtime_version`**、宿主 `load_role` 校验相关，见 [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md)、[CHANGELOG.md](../CHANGELOG.md)。

---

## 仓内模块兼容契约

OCLive 的能力上限取决于整条模块链，而不是某一个组件的最新版。功能更新须同时核对：

```text
角色包/插件资源 → 内核契约与编排 → Tauri/Bridge → distros/shared → Chat Pro/Theater → Vue/iframe/legacy 回退
```

| 边界 | 当前兼容机制 | 限制 / 开发要求 |
|------|--------------|-----------------|
| 角色包 ↔ 内核 | `schema_version`、`min_runtime_version`、`oclive_validation` | 新键优先可选并有默认；Breaking 留至少一个发布周期读兼容 |
| 内核 ↔ 前端 | `api_version`、Rust DTO、`distros/shared/src/api` 镜像、错误码 drift | DTO/命令变化必须同步消费者与契约测，不能只保证 Rust 编译 |
| Tauri ↔ 目录插件 | manifest `schema_version: 1`、插槽名、`bridge.invoke`、事件与 `rpcMethods` 白名单 | 插件 `version` 仅标识插件自身，**不代表宿主兼容范围**；无法表达的新宿主依赖须保留回退或走 Breaking/RFC |
| Chat Pro ↔ 插件 UI | `entry` iframe + 可选 `vueComponent`、共享 `PluginSlotEmbed` | 两种入口都存在时必须同能力；不能只更新 Vue 后让 iframe 落后 |
| Chat Pro 壳 ↔ shared | Fluent / Tool 共用 shared store/composable | 壳特有布局可分叉，契约、状态归属、事件与取消语义不可分叉 |

**结构门禁**：`npm run check:module-compat` 对拍内核与前端插槽注册表、官方插件 manifest、Vue/iframe 文件、RPC timeout 声明和插件索引版本。该门禁不证明 sidecar、音频设备或真实 WebView 行为，相关功能仍须定向集成/烟测。

关联改动与完成声明遵循 [`AI_CHANGE_BOUNDARIES.md`](../handoff/AI_CHANGE_BOUNDARIES.md) G17；破坏性变化遵循 [`BREAKING_CHANGE_PROCESS.md`](../handoff/BREAKING_CHANGE_PROCESS.md)。

---

## 对外兼容一页表（主程序 / 编写器 / 启动器 / 包 / 内核 / CLI）

| 组件 | 版本来源 | 与主程序关系 | 备注 |
|------|----------|----------------|------|
| **oclivenewnew（主程序）** | 根 `package.json` / `distros/desktop-tauri/Cargo.toml` | — | 当前快照 **0.5.0** |
| **oclive_kernel_runtime** | `kernel/crates/oclive_kernel_runtime/Cargo.toml` | 宿主与无头 HTTP **path 依赖**；`SendMessageResponse.api_version`（`API_VERSION` **u32**，当前 **1**）、`RUNTIME_API_VERSION`（字符串 **0.2.0**） | OOCP / 黑盒脚本若断言载荷版本，以 `creator-docs/testing/OOCP_TEST_SUITE.md` 为准 |
| **oclive-cli** | `kernel/crates/oclive-cli/Cargo.toml` | 生成 `kernel_server` / `library` 骨架；**不自带**桌面 `AppState` / SQLite 策略 | 与主程序契约对齐见 [OCLIVE_CLI_GUIDE.md](cli/OCLIVE_CLI_GUIDE.md)、模板 `CONFIG_REFERENCE.md` |
| **oclive-pack-editor（编写器）** | 另仓 `package.json` | 产出 `distros/chat-pro/roles/{id}/`；**`ui.json`** 与主程序见上文「兼容性表」 | `HOST_RUNTIME_VERSION` 应对齐主程序 `version`（编写器 README） |
| **oclive-vscode（VS Code 扩展）** | 另仓 `package.json` | spawn/attach **`kernel_server --api`**；`distro.oclive.toml` 镜像主仓 `examples/distro-profiles/vscode.oclive.toml` | 当前 **0.4.1**；推荐主程序 **0.5.0** |
| **oclive-launcher（启动器）** | 另仓 `package.json` | 注入 **`OCLIVE_ROLES_DIR`**、可选模型名与 zip 安装；**不替代**主程序契约 | [启动器 README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md) |
| **角色包** | `manifest.json`（`schema_version`、`min_runtime_version`） | 低版本主程序可能拒载或降级能力 | [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md)、`RoleStorage::load_role` |
| **宿主 SQLite** | `kernel/crates/oclive_kernel_host/migrations/*.sql` | 仅随 **主程序** 发版迁移；**不可**用旧主程序打开新迁移写过的 DB 再降级（除非 CHANGELOG 明确支持） | 破坏性迁移须在 **CHANGELOG 双语** + 本表「破坏性」段写明 |

破坏性变更时：同步 **`CHANGELOG.md` / `CHANGELOG.en.md`**、上文「兼容性表」、**`oclive_validation`**（若 touched 键）、及姊妹仓 README 中的最低版本说明。

### 发版审阅（维护者自检）

1. 核对本节「快照」三处 semver：**根 `package.json`**、**`distros/desktop-tauri/Cargo.toml`**、**`oclive_kernel_runtime`**（发版 bump 时常需同改）。  
2. 按 [CONTRIBUTING](../CONTRIBUTING.md) 与 [版本规则](development/RELEASE_VERSIONING.md) 更新 **对外说明**：若 bump 了契约或姊妹仓依赖，更新本页表格或快照句。
3. **HTTP / OOCP**：若 `API_VERSION` 或 `RUNTIME_API_VERSION` 变更，必须同步测试套件与文档（见 `creator-docs/testing/OOCP_TEST_SUITE.md`）。

无头 HTTP 的认证属于宿主启动契约：`--api` 默认要求 `OCLIVE_API_TOKEN`，调用方在除 `/health` 外的请求发送 `x-oclive-api-token`；不得把 `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` 用于生产或持久化数据目录。

---

## 如何查看版本

| 产品 | 查看方式 |
|------|----------|
| **主程序** | 应用内 **设置 / 关于**（若有）；或安装包名与仓库 **`package.json`** / **`CHANGELOG.md`** |
| **编写器** | 编写器窗口 **关于**；或仓库 **`package.json`** |

---

## Remote LLM env（指针）

Remote LLM 的 **`OCLIVE_LLM_BACKEND` / `OCLIVE_REMOTE_LLM_*` / `OCLIVE_LLM_CLOUD_API_STYLE` / OpenAI 别名**、JSON-RPC vs OpenAI-compatible 分叉，以及**本机第二本地选型**，以 **[REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §2.0** 为 SSOT；本文不另维护 env 长表。

## 相关文档

- [handoff/A5_CLOSURE_SUMMARY.md](../handoff/A5_CLOSURE_SUMMARY.md)
- [role-pack/ui.json.schema.json](role-pack/ui.json.schema.json)
- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §2.0 — Remote LLM env 矩阵
- [CHANGELOG.md](../CHANGELOG.md)
