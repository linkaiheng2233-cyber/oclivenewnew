# 编写器（oclive-pack-editor）与主程序（oclivenewnew）版本兼容说明

本文档说明 **角色包内 `ui.json`** 与 **主程序** 的兼容关系，避免「编写器导出的字段主程序不认识」或「主程序已支持但编写器未导出」的困惑。

**版本号格式**：两项目均采用 **语义化版本（SemVer）** `MAJOR.MINOR.PATCH`，见各仓库根目录 **`package.json`** 的 **`version`** 字段。

**当前仓库快照（文档更新时 · 与发版审阅对齐）**：

- **oclivenewnew**（主程序 / Tauri 宿主）：**`0.5.2`**（根 `package.json` `version` 与 `distros/desktop-tauri/Cargo.toml` `version` 须一致）
- **oclive_kernel_runtime**（共享契约 crate）：**`0.2.0`**（`kernel/crates/oclive_kernel_runtime/Cargo.toml`；DTO / `API_VERSION` 等见该 crate）
- **oclive_kernel_host**（完整宿主与进程内门面 crate）：**`0.2.0`**；受支持的 Rust 集成入口为 `OcliveKernel`，内部 `AppState` 不是兼容契约。
- **oclive-cli**（脚手架 CLI）：**`0.1.0`**（`kernel/crates/oclive-cli/Cargo.toml`；**独立 semver**，不强制与桌面宿主同号；`init --kernel-source` 接主仓时以 path 依赖对齐契约）。**默认构建**仅依赖 `oclive_kernel_runtime` + `oclive_validation`（`cargo tree -p oclive-cli --no-default-features` **无** `libsqlite3-sys` / `axum`）。**`doctor config-resolve`** 默认走 runtime 纯解析；**`--via-host`**（feature `diagnostics-host`）可选 in-memory `AppState` 深度诊断。
- **oclive-pack-editor**（编写器，姊妹仓）：**`0.5.1`**（该仓 `package.json`；与主程序 **0.5.x** 对拍 `ui.json`）
- **oclive-vscode**（VS Code 扩展，姊妹仓）：**`0.5.0`**（独立 semver；spawn/attach 契约对齐主程序 **≥0.4.0**，推荐 **0.5.2**）

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

3. **角色包蓝图与旧配置兼容层**
   - 当前 Stable v4 以 `pipeline.ocblueprint` 的 `slot_registry`（及可选 `runtime_config` / `extensions`）为磁盘真源；`settings.json` / `plugin_backends` 仅是 legacy 迁移输入。兼容性与 **`min_runtime_version`**、宿主 `load_role` 校验相关，见 [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md)、[CHANGELOG.md](../CHANGELOG.md)。

---

## 仓内模块兼容契约

OCLive 的能力上限取决于整条模块链，而不是某一个组件的最新版。功能更新须同时核对：

```text
角色包/插件资源 → 内核契约与编排 → Tauri/Bridge → distros/shared → Chat Pro/Theater → Vue/iframe/legacy 回退
```

| 边界 | 当前兼容机制 | 限制 / 开发要求 |
|------|--------------|-----------------|
| 角色包 ↔ 内核 | `schema_version`、`min_runtime_version`、`oclive_validation` | 新键优先可选并有默认；Breaking 留至少一个发布周期读兼容 |
| Rust 宿主 ↔ 内核 | `oclive_kernel_host::OcliveKernel` + `KernelErrorBody` code | 稳定性指 Rust 源码级门面，不是 C ABI；新增入口向后兼容，破坏性签名按宿主 semver / CHANGELOG 处理；禁止把内部 `AppState` 当公开合同 |
| 内核 ↔ 前端 | `api_version`、Rust DTO、`distros/shared/src/api` 镜像、错误码 drift | DTO/命令变化必须同步消费者与契约测，不能只保证 Rust 编译 |
| Tauri ↔ 目录插件 | manifest `schema_version: 1`、插槽名、`bridge.invoke`、事件与 `rpcMethods` 白名单 | 插件 `version` 仅标识插件自身，**不代表宿主兼容范围**；无法表达的新宿主依赖须保留回退或走 Breaking/RFC |
| Chat Pro ↔ 插件 UI | `entry` iframe + 可选 `vueComponent`、共享 `PluginSlotEmbed` | 两种入口都存在时必须同能力；不能只更新 Vue 后让 iframe 落后 |
| Chat Pro 壳 ↔ shared | Fluent / Tool 共用 shared store/composable | 壳特有布局可分叉，契约、状态归属、事件与取消语义不可分叉 |

**结构门禁**：`npm run check:module-compat` 对拍内核与前端插槽注册表、官方插件 manifest、Vue/iframe 文件、RPC timeout 声明和插件索引版本。该门禁不证明 sidecar、音频设备或真实 WebView 行为，相关功能仍须定向集成/烟测。

关联改动与完成声明遵循 [`AI_CHANGE_BOUNDARIES.md`](../handoff/AI_CHANGE_BOUNDARIES.md) G17；破坏性变化遵循 [`BREAKING_CHANGE_PROCESS.md`](../handoff/BREAKING_CHANGE_PROCESS.md)。

---

<a id="six-slot-minimal-compatibility-draft"></a>

## 六槽 Base / 最小逻辑角色的兼容审阅范围（已采纳 · 2026-10-09）

**效力**：维护者于 2026-10-09 审核并采纳本节的有限兼容审阅规则，用于现有公共层的变更判断；这不构成新的稳定版本保证。受检实现基线为 `d9847b70c8c6164ab4715813f7e55bcfb6bb20ed`；本节没有改 Rust API、校验器、版本或运行语义。模块职责和能力语义只引用 [MODULE_MAP §0.4–0.9](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension)，最小角色定义只引用 [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md#01-已确认的最小逻辑-contract)。下表圈定审阅对象，不将所在 crate 的全部导出纳入承诺。

### 有限公共清单

| 审阅对象 | 现有公共入口与依据 | 限定范围 |
|---|---|---|
| 六槽调用 | `oclive_kernel_contracts::{MemoryBase, EmotionBase, EventBase, PromptBase, LlmBase, AgentBase}`；[调用绑定](../kernel/crates/oclive_kernel_contracts/src/slot_base.rs) | 现有单方法的名字、签名、借用关系、对应正常结果及已确认能力语义；不要求依固定顺序调用六槽 |
| 异步绑定 | 同 crate 的 `BaseCallFuture`；同上源码 | 现有可 `dyn` 使用的 boxed local future；不强制实现或 future 为 `Send` / `Sync` / `'static`，不推导取消、回滚或重试安全 |
| 请求与失败载体 | `oclive_kernel_types::{MemoryBaseRequest, EmotionBaseRequest, EventBaseRequest, PromptBaseRequest, LlmBaseRequest, AgentBaseRequest, BaseCallError, BaseCallErrorKind}`；[数据绑定](../kernel/crates/oclive_kernel_types/src/slot_base.rs) | 现有字段/类型、材料与用途区别，以及正常结果与调用失败的区别；不冻结 HTTP/SSE、权限句柄或产品终态 |
| 最小逻辑定义 | `oclive_validation::MinimalRoleDefinition`，由 `oclive_kernel_types::MinimalRoleDefinition` 重导出；[定义](../kernel/crates/oclive_validation/src/minimal_role.rs) | 非空 `persona_prompt` 与至少一项、每项非空的 `visual_assets` 引用；正文与资产顺序原样保留，不指定七张表情图、文件名、URI 方案或完整 `Role` |
| 逻辑校验与 JSON 投影 | `oclive_validation::{validate_minimal_role_definition, parse_minimal_role_definition}`；同上源码 | 当前逻辑合法性及投影的接受/拒绝边界；当前未知字段被忽略而非保留。通过不证明资源存在、安全或可渲染，不定义跨发行版磁盘包 |

完整参考 Host 的旧端口、`AppState`、最小角色本地加载/会话 DTO、共享消费者的具体装配、丰富角色格式、目录插件协议、网络桥与存储均不因本表成为小 Kernel 的稳定 API。共享运行库继续承担最小适配，发行版承担资源、生命周期和扩展接线；这不阻止它们独立制定自己的兼容规则。高级情绪驱动长期记忆沿[已确认的可选扩展决定](architecture/DESIGN_DECISIONS.md#emotion-memory-extension-deferred)，不成为 Base 必修项。

### 相容与破坏性变化：审阅判读

| 变化例子 | 判读与所需核对 |
|---|---|
| 替换内部检索/解码算法或修基础缺陷，保留公开输入、结果和错误承诺 | 可作为保持合同的内部改动；按实际消费者与原问题回归，不把算法改动一概视为相容或保证模型质量 |
| 给 Base 增加必做方法/超 trait，强化 `Send` / `Sync` / `'static`，或把 local future 改为必须跨线程 | 原合法实现可能无法继续编译；按源代码 Breaking 处理，不作为内部重构直接合入 |
| 给当前公开请求 struct 增加一个 `Option` 字段 | 外部 struct literal 仍可能不能编译；Rust 源码兼容须单独核，不能套用“JSON 可选字段通常兼容”结论 |
| 更名/删除现有字段、改变借用/返回形状、把正常空结果解释成失败或未调用 | 分别核源码与行为 Breaking；`Ok`、空文本和能力返回仍不直接证明产品成功或 invocation terminal |
| 给 `#[non_exhaustive] BaseCallErrorKind` 增加原因 | 当前绑定允许外部保留未知分支；仍须核行为和适配。未知原因继续表示未正常完成，不自动等于可重试、无副作用或已停止；不把 `detail` / Display 文案升级为机器协议 |
| 在最小逻辑投影新增必填丰富角色字段，拒绝当前可接受的未知字段，或注入关系/七维人格默认值 | 改变最小输入或校验边界，按数据/行为 Breaking 审阅；当前忽略未知字段不承诺无损保存丰富包 |
| 增加独立可选增强 | 先明确增强的输入、输出、关联、授权与所需版本；原 Base-only 实现不因此承担新增义务。必需增强不可静默降成“已经满足”，未选择的增强不隐式启用 |

“相容”须注明 **Rust 源码 / 逻辑数据 / 行为 / 具体传输** 哪一层；同一改动可在一层相容、另一层破坏。当前请求/错误的 Rust 绑定不自带 serde wire，也不因跨发行版最小定义存在而形成 C ABI 或统一网络协议。

### 审阅、迁移与有限验证

沿用 [Breaking 流程](../handoff/BREAKING_CHANGE_PROCESS.md)，不另造审批系统：变更提出者列受影响符号、旧/新行为、下游及迁移；Host/模块作者按实际使用面改适配和测试；角色转换器作者负责本发行版格式到逻辑定义的映射与资源检查。兼容层是否需要、能否实现及保留多久按具体 Breaking 由维护者审核；既有发布周期读兼容规则只在其实际适用的数据面使用，不假造 Rust trait 的运行时兼容层。

复用已有[Base-only fixture](../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs)、[请求/错误单测](../kernel/crates/oclive_kernel_types/src/slot_base.rs)、[最小逻辑校验](../kernel/crates/oclive_validation/src/minimal_role.rs)与[可替换六槽消费案例](../kernel/crates/oclive_kernel_runtime/tests/minimal_role_six_slots.rs)。实际改公共 Rust API 时跑受影响回归及 G8 的 workspace doctest，选明确使用旧面的消费者证明原问题；JSON/校验变化另核接受和拒绝样例。单次已有全 CI 或文件存在不等于所有未来第三方组合已验证；证据足以识别本次兼容影响后停止，不穷尽 Host/算法/设备组合。

**版本与效力**：保持当前各产物的[独立版本规则](development/RELEASE_VERSIONING.md)，不把 crate `0.2.0`、设计稿编号、`API_VERSION` 或角色包 schema 互相代用，不新增六槽协议协商或宣布 1.0。本清单及判读已获采纳；每次实际变更仍须列出影响面并通过适用审阅与验证。真实版本 bump、旧接口撤销或新执行语义在对应实质变更中单独确定；本规则不能替代它们的批准，不证明稳定发布或所有下游已验，也不关闭 K-CORE-BOUNDARY-01 的其它剩余面。

---

## 对外兼容一页表（主程序 / 编写器 / 启动器 / 包 / 内核 / CLI）

| 组件 | 版本来源 | 与主程序关系 | 备注 |
|------|----------|----------------|------|
| **oclivenewnew（主程序）** | 根 `package.json` / `distros/desktop-tauri/Cargo.toml` | — | 当前快照 **0.5.2** |
| **oclive_kernel_runtime** | `kernel/crates/oclive_kernel_runtime/Cargo.toml` | 宿主与无头 HTTP **path 依赖**；`SendMessageResponse.api_version`（`API_VERSION` **u32**，当前 **1**）、`RUNTIME_API_VERSION`（字符串 **0.2.0**） | OOCP / 黑盒脚本若断言载荷版本，以 `creator-docs/testing/OOCP_TEST_SUITE.md` 为准 |
| **oclive_kernel_host** | `kernel/crates/oclive_kernel_host/Cargo.toml` | HTTP、Tauri 与 `OcliveKernel` 共用完整 `process_message` / SQLite / 插件 / Event Ring 编排 | 当前 **0.2.0**；对外用 `OcliveKernel`，不直接装配 `AppState` |
| **oclive-cli** | `kernel/crates/oclive-cli/Cargo.toml` | 无 `--kernel-source` 生成 serde 占位；带该参数的 `library` 生成完整 `OcliveKernel` path 依赖与门面 | 与主程序契约对齐见 [OCLIVE_CLI_GUIDE.md](cli/OCLIVE_CLI_GUIDE.md)、模板 `CONFIG_REFERENCE.md` |
| **oclive-pack-editor（编写器）** | 另仓 `package.json` | 产出 `distros/chat-pro/roles/{id}/`；**`ui.json`** 与主程序见上文「兼容性表」 | `HOST_RUNTIME_VERSION` 应对齐主程序 `version`（编写器 README） |
| **oclive-vscode（VS Code 扩展）** | 另仓 `package.json` | spawn/attach **`kernel_server --api`**；`distro.oclive.toml` 镜像主仓 `examples/distro-profiles/vscode.oclive.toml` | 当前 **0.5.0**；推荐主程序 **0.5.2** |
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

- [历史验收证据：A5_CLOSURE_SUMMARY.md](../handoff/archive/A5_CLOSURE_SUMMARY.md)
- [role-pack/ui.json.schema.json](role-pack/ui.json.schema.json)
- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §2.0 — Remote LLM env 矩阵
- [CHANGELOG.md](../CHANGELOG.md)
