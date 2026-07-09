# AI 深读索引（Agent Reading Index）

> **读者**：Cursor / Codex / 自动化 Agent / 维护者用 AI 改代码。  
> **GitHub 首页 [`README.md`](../README.md) 面向人类**；本文是 **AI 专用分类目录**——链 SSOT、**禁止复制长表**（G14）。  
> **快速约束**：[`AGENTS.md`](../AGENTS.md) · **人类阶梯**：[`human-docs/README.md`](../human-docs/README.md)

---

## 怎么用本文

1. **每次改代码前**：先读 [§0 门禁](#0-改代码前门禁必读)（约 5 分钟）。  
2. **按任务选路径**：跳 [§9 场景阅读路径](#9-按任务选阅读路径)。  
3. **需要细节**：只打开对应 SSOT 全文，**勿**在对话里复述整表。  
4. **带数字的汇报**：须过 [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md)。  
5. **新建/大改文档前**：读 [`handoff/README.md`](./README.md) §文档分责（G10–G16）。

**效率源于限制**：索引越薄、SSOT 越单点，Agent 越不易 drift。

---

## 0. 改代码前门禁（必读）

| 序 | 文档 | 何时 |
|----|------|------|
| 1 | [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) | **G1–G16** · 代码 + 文档纪律 |
| 2 | [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) | 六槽 / 设施 / 独立通道 **定义与关系** |
| 3 | [`NAMING_CONVENTIONS.md`](../creator-docs/NAMING_CONVENTIONS.md) §4.2 | canonical import · 禁止别名 |
| 4 | [`BUS_FACTOR_NOTES.md`](./BUS_FACTOR_NOTES.md) | `process_message` · DB · 错误码 **文件锚点** |
| 5 | [`.cursor/rules/oclivenewnew.mdc`](../.cursor/rules/oclivenewnew.mdc) | 7 条硬约束镜像 |

**必背常量（勿查错字段）**：回复 DTO 字段 **`reply`**（不是 `response`）· 六槽键 `plugin_backends` / `slot_registry` · 蓝图 **`steps[]` 不参与首轮调度**。

---

## 1. 仓库身份与物理布局

| 主题 | SSOT |
|------|------|
| 产品定位（组装—契约—分发层） | [`OCLIVE_POSITIONING_DIFFERENTIATION.md`](./OCLIVE_POSITIONING_DIFFERENTIATION.md) |
| 发版版本 | 根 `package.json` · [`CHANGELOG.md`](../CHANGELOG.md) · [`PROJECT_CURRENT_STATUS.md`](../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md) |
| Crate 地图 | [`kernel/crates/README.md`](../kernel/crates/README.md) |
| 仓库布局速记 | [`AGENTS.md`](../AGENTS.md) §仓库布局 |
| Cargo target（仓外） | [`../oclive-dev-artifacts/oclivenewnew-cargo-target/`](../.cargo/config.toml) |
| 姊妹仓 | `oclive-pack-editor` · `oclive-launcher` · `oclive-plugin-market` · `oclive-vscode`（各仓 `AGENTS.md` 指回本索引） |

---

## 2. 架构 · 模块 · 正交分层

| 主题 | SSOT |
|------|------|
| **模块注册表**（四大类 · 逐槽关系 · 改动约束） | [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) |
| 对外架构叙述 · 模块编号 | [`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| 解耦全景 · 插件清单（非 MODULE_MAP 双写） | [`human-docs/team/ARCHITECTURE_DECOUPLING_PANORAMA.md`](../human-docs/team/ARCHITECTURE_DECOUPLING_PANORAMA.md) |
| 单核双态（外核 / Monolith 宏核） | [`RFC_OCLIVE_MONOLITH_MODE.md`](../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| 实验核 dual_core（**默认关**） | [`RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md`](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · [`TECHNICAL_DEBT_INVENTORY.md §2`](./TECHNICAL_DEBT_INVENTORY.md) |
| 独立通道（voice.asr 等 · **非六槽**） | [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) |
| 角色包 vs 蓝图 vs 发行版 | [`ROLE_PACK_BOUNDARY.md`](./ROLE_PACK_BOUNDARY.md) · [`BLUEPRINT_FOLDER_LAYOUT.md`](./BLUEPRINT_FOLDER_LAYOUT.md) |
| 三发行版 HostProfile | [`THREE_DISTRO_KERNEL_CLOSURE.md`](./THREE_DISTRO_KERNEL_CLOSURE.md) · [`DISTRO_CAPABILITY_PROFILE.md`](../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) |
| Turn Thinking（Fast/Deep · **非第七槽**） | [`RFC_TURN_THINKING_PERSISTENCE.md`](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) |
| domain ↔ infrastructure 分层 | [`ARCHITECTURE_LAYERING.md`](./ARCHITECTURE_LAYERING.md) · `node scripts/check-domain-layering.mjs` |

---

## 3. 主编排与代码锚点（不可绕开）

```
Vue invoke / HTTP --api
  → distros/desktop-tauri/src/api/*.rs
  → oclive_kernel_host::process_message
  → co_present → turn_pipeline (pre → Event → Prompt → LLM → post)
  → PluginHost 六槽
```

| 锚点 | 路径 |
|------|------|
| 主编排 SSOT | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` |
| 回合流水线 | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/` |
| Prompt 公式 | `kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/mod.rs`（**不是 Result**） |
| DTO | `kernel/crates/oclive_kernel_types/src/models/dto.rs` |
| DB 迁移 SSOT | `kernel/crates/oclive_kernel_host/migrations/001_init.sql` |
| Tauri 命令注册 | `distros/desktop-tauri/src/lib.rs`（仅 `generate_handler!`） |
| 前端 invoke 封装 | `distros/shared/src/api/`（**camelCase**） |

详情：[`BUS_FACTOR_NOTES.md`](./BUS_FACTOR_NOTES.md) · [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md)（**13** 条热路径）

---

## 4. 契约 · 六槽 · 插件 · 角色包

| 主题 | SSOT |
|------|------|
| 六槽 DTO · `send_message` 顺序 · backend 枚举 | [`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 六槽 backend 24 格真值 | [`SLOT_BACKEND_REALITY_MATRIX.md`](./SLOT_BACKEND_REALITY_MATRIX.md) |
| 角色包磁盘格式 | [`ROLE_PACK_SPEC.md`](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 蓝图 settings / `slot_registry` | [`SETTINGS_REFERENCE.md`](../creator-docs/cli/SETTINGS_REFERENCE.md) |
| Remote JSON-RPC | [`REMOTE_PLUGIN_PROTOCOL.md`](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| 目录插件 · bridge · 权限 | [`DIRECTORY_PLUGINS.md`](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) · [`BRIDGE_API_REFERENCE.md`](../creator-docs/plugin-and-architecture/BRIDGE_API_REFERENCE.md) |
| 插件放置决策 | [`PLUGIN_PLACEMENT_GUIDE.md`](../creator-docs/plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md) |
| 跨宿主 L1/L2/L3 | [`CROSS_HOST_MEMORY.md`](../creator-docs/role-pack/CROSS_HOST_MEMORY.md) |
| 聊天 vs 记忆三套存储 | [`CHAT_STORAGE_ARCHITECTURE.md`](./CHAT_STORAGE_ARCHITECTURE.md) |
| 错误码 | [`KERNEL_ERROR_CODE_CONVENTION.md`](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) · [`ERROR_CODES.md`](../creator-docs/getting-started/ERROR_CODES.md) |
| 校验 crate（native + WASM） | `kernel/crates/oclive_validation` |

**学习路径（深读）**：[`PLUGIN_AUTHOR_LEARNING_PATH.md`](../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) · [`CREATOR_LEARNING_PATH.md`](../creator-docs/role-pack/CREATOR_LEARNING_PATH.md) · [`KERNEL_INTEGRATOR_LEARNING_PATH.md`](../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)

---

## 5. 发行版 · 内核生命周期 · 集成

| 主题 | SSOT |
|------|------|
| Chat Pro（`desktop`） | `distros/chat-pro/` · profile `distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml` |
| VS Code Flash | [`handoff/vscode/`](./vscode/) · 姊妹仓 `oclive-vscode` |
| AI Theater | [`handoff/theater/`](./theater/) · `distros/theater/` |
| 内核 attach/spawn | [`DISTRO_KERNEL_LIFECYCLE.md`](../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) · [`KERNEL_SCHEDULER_RESCOPE.md`](./KERNEL_SCHEDULER_RESCOPE.md) |
| 无头 HTTP | `examples/headless-kernel-minimal/` · `--api` → `:8420` |
| oclive-cli 工厂 | [`OCLIVE_CLI_GUIDE.md`](../creator-docs/cli/OCLIVE_CLI_GUIDE.md) · [`KERNEL_FACTORY_VISION.md`](../creator-docs/getting-started/KERNEL_FACTORY_VISION.md) |
| OOCP 黑盒 S0–S12 | [`OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md) · `examples/oocp-test-suite/` |

---

## 6. 测试 · CI · 供应链

| 主题 | SSOT / 命令 |
|------|-------------|
| 日常门禁 | `npm run check` · `npm run check:rust`（**不含 doctest**） |
| 发版门禁 | `npm run check:release`（**含 doctest**） |
| Dimension 5 | **15** 项注册 / **14** 项 CI · `node scripts/dimension5-acceptance.mjs --ci` |
| 分层 ratchet | `node scripts/check-domain-layering.mjs` |
| 文档 registry | `node scripts/check-doc-registry.mjs` |
| 路径/stale 别名 | `node scripts/check-stale-paths.mjs` |
| cargo audit | [`KNOWN_VULNERABILITIES.md`](../creator-docs/security/KNOWN_VULNERABILITIES.md) |
| Breaking 流程 | [`BREAKING_CHANGE_PROCESS.md`](./BREAKING_CHANGE_PROCESS.md) |

---

## 7. handoff 活跃文档（维护者深读）

完整列表与分责：[`handoff/README.md`](./README.md) §活跃文件 · §文档分责。

| 类别 | 代表文档 |
|------|----------|
| **模块 / 边界** | MODULE_MAP · ROLE_PACK_BOUNDARY · CHAT_STORAGE · AI_CHANGE_BOUNDARIES |
| **关键路径** | BUS_FACTOR · INVOKE_HOTPATH · BLUEPRINT_FOLDER_LAYOUT |
| **技术债 / 冻结** | TECHNICAL_DEBT_INVENTORY · PERF_PHASES · TTFT_BENCHMARK · DEEP_PROMPT_DISTILLATION |
| **产品 / 定位** | OCLIVE_POSITIONING · THREE_DISTRO_KERNEL_CLOSURE · PRODUCT_LINE_TASK_BUCKETS |
| **发行版子目录** | [theater/](./theater/) · [vscode/](./vscode/) · [launcher/](./launcher/) · [pack-editor/](./pack-editor/) |

**禁止当 truth**：[`handoff/archive/`](../handoff/archive/) · [`04_4.6_PROJECT_TRUTH_CHECKLIST.md`](./04_4.6_PROJECT_TRUTH_CHECKLIST.md)（G3）

---

## 8. creator-docs 分类入口

总索引：[`DOCUMENTATION_INDEX.md`](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

| 目录 | 内容 |
|------|------|
| [`getting-started/`](../creator-docs/getting-started/) | 架构总览 · 学习路径 · 错误码 · 项目状态 |
| [`plugin-and-architecture/`](../creator-docs/plugin-and-architecture/) | PLUGIN_V1 · Remote · 目录插件 · 作者路径 |
| [`role-pack/`](../creator-docs/role-pack/) | ROLE_PACK_SPEC · 迁移 · 跨宿主 |
| [`kernel/`](../creator-docs/kernel/) | HostProfile · 生命周期 · APP_DATA |
| [`cli/`](../creator-docs/cli/) | oclive-cli · SETTINGS_REFERENCE |
| [`rfc/`](../creator-docs/rfc/) | 草案（**未默开项勿当已发布**） |
| [`roadmap/`](../creator-docs/roadmap/) | 愿景 · 按月路线（中文为主） |
| [`testing/`](../creator-docs/testing/) | OOCP · 验收说明 |
| [`security/`](../creator-docs/security/) | 已知漏洞 · 审计范围 |

**示例代码**：[`examples/`](../examples/)（directory-plugin-* · voice-loop-minimal · headless-kernel-minimal · oocp-test-suite）

---

## 9. 按任务选阅读路径

### 只改角色包（G1：不动六槽）

1. [`ROLE_PACK_BOUNDARY.md`](./ROLE_PACK_BOUNDARY.md)  
2. [`ROLE_PACK_SPEC.md`](../creator-docs/role-pack/ROLE_PACK_SPEC.md)  
3. [`CREATOR_GOLDEN_PATH.md`](../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md)  
4. 示例：`distros/chat-pro/roles/mumu/`

### 改聊天 / 编排 / Prompt

1. §0 门禁 → [`BUS_FACTOR_NOTES.md`](./BUS_FACTOR_NOTES.md)  
2. [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) §编排  
3. `process_message.rs` · `turn_pipeline/` · `prompt_builder/`  
4. [`CHAT_STORAGE_ARCHITECTURE.md`](./CHAT_STORAGE_ARCHITECTURE.md)（若动持久化）

### 新增 / 修改插件或侧通道

1. [`PLUGIN_PLACEMENT_GUIDE.md`](../creator-docs/plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md)  
2. [`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [`DIRECTORY_PLUGINS.md`](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)  
3. [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)（若不进六槽）  
4. 示例：`examples/directory-plugin-minimal/` · `distros/chat-pro/plugins/com.oclive.voice.asr/`

### 改 Tauri / 前端 Chat Pro

1. [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md)  
2. `distros/desktop-tauri/src/api/*.rs`（薄封装）  
3. `distros/shared/src/stores/` · `distros/chat-pro/src/shells/`  
4. [`human-docs/modules/surfaces/frontend-chat-pro.md`](../human-docs/modules/surfaces/frontend-chat-pro.md)

### 内核集成 / 新发行版

1. [`KERNEL_INTEGRATOR_LEARNING_PATH.md`](../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)  
2. [`DISTRO_CAPABILITY_PROFILE.md`](../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)  
3. [`DISTRO_KERNEL_LIFECYCLE.md`](../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md)  
4. [`THREE_DISTRO_KERNEL_CLOSURE.md`](./THREE_DISTRO_KERNEL_CLOSURE.md)

### 改文档

1. [`handoff/README.md`](./README.md) §文档分责 — **查是否已有 SSOT**  
2. [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G10–G16  
3. 模块关系 **只**改 MODULE_MAP；**链接代替复制**

---

## 10. 模块化开工包（按需深读）

人类排版 · Agent 可精读：`human-docs/modules/README.md`

| 类别 | 路径 |
|------|------|
| 六槽 | [`human-docs/modules/slots/`](../human-docs/modules/slots/) |
| 设施 | [`human-docs/modules/facilities/`](../human-docs/modules/facilities/) |
| 独立通道 | [`human-docs/modules/side-channels/`](../human-docs/modules/side-channels/) |
| 编排策略 | [`human-docs/modules/orchestration/`](../human-docs/modules/orchestration/) |
| 角色包 / 宿主面 | [`human-docs/modules/packs/`](../human-docs/modules/packs/) · [`surfaces/`](../human-docs/modules/surfaces/) |

---

## 11. 常用命令

```bash
npm run tauri:dev
npm run check
npm run check:release
npm run test:unit
cargo run -p oclive-cli -- dev
node scripts/dimension5-acceptance.mjs --ci
```

---

*入口精简版：[`AGENTS.md`](../AGENTS.md) · 人类文档包：[`human-docs/ai-package/README.md`](../human-docs/ai-package/README.md) · 文档总索引：[`DOCUMENTATION_INDEX.md`](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)*
