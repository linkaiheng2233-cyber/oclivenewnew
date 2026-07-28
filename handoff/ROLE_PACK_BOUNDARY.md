# 角色包与蓝图 · 职责边界（SSOT）

**读者**：创作者、宿主集成方、Cursor / Agent。  
**状态**：与 **Stable v4 扩展外壳已交付** 对齐；v2 保持兼容，**v3 双核**见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)（Opt-in Beta，默认关）。

| 文档 | 用途 |
|------|------|
| 角色包（入门） | [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 蓝图 / 系统配置 | [SETTINGS_REFERENCE.md](../creator-docs/cli/SETTINGS_REFERENCE.md) |
| **蓝图目录 `blueprint/`（拉取式、本体保持瘦）** | **[BLUEPRINT_FOLDER_LAYOUT.md](./BLUEPRINT_FOLDER_LAYOUT.md)** |
| **蓝图扩展外壳 / 资源协调** | **[RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)** |
| 双核对齐 | [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) |

---

## 1. 一句话划分

| 组件 | 职责 | 面向 |
|------|------|------|
| **角色包** | 角色身份、人格、关系、提示词与场景**内容** | **初级创作者** |
| **蓝图** | 槽位实例、后端路由、模型名、交互/记忆/远程策略、双核开关等**系统配置** | **高级开发者 / 宿主管理员** |

**物理落盘（今日）**：v2/v3/v4 均以 **`distros/chat-pro/roles/{id}/pipeline.ocblueprint`** 为宿主加载入口；新包 canonical 格式为 **Stable v4**。**逻辑上**分责；外置片段、扩展载荷、专家修订与说明放入 **`distros/chat-pro/roles/{id}/blueprint/`**，经 `includes` 或 v4 `extensions.*.config_ref` 引用（见 [BLUEPRINT_FOLDER_LAYOUT.md](./BLUEPRINT_FOLDER_LAYOUT.md)），**禁止**把长文与向导结果搅进蓝图 JSON。

**legacy**：`manifest.json` + `settings.json` 已废弃，**不得**与 `pipeline.ocblueprint` 并存；引擎字段应视为**蓝图侧**，非「角色门面」。

---

## 2. 角色包可编辑（创作者）

### 2.1 `meta` 创作者子集（v2/v4）

| 字段 | 说明 |
|------|------|
| `id` | 角色 id（与目录名一致） |
| `name` | 展示名 |
| `version` | 包版本 |
| `author` | 作者 |
| `description` | 简介 |
| `personality` | 七维人格（对象或 7 元数组） |
| `relations` | 用户关系定义 |
| `default_relation` | 默认关系 id |
| `scenes` | 场景 id 列表（与 `scenes/` 目录一致） |

可选创作者向 **`meta`**（剧情/人设，非引擎路由）：

| 字段 | 说明 |
|------|------|
| `life_trajectory` / `life_schedule` | 异地/人生轨迹文案（见 README_MANIFEST） |
| `evolution.personality_source` | **仅 v2 兼容落点**；Stable v4 由高级运行时视图写入 `runtime_config.evolution.personality_source` |

### 2.2 目录与文件（非 JSON 槽位）

| 路径 | 说明 |
|------|------|
| **`core_personality.txt`** | **Tier0 人设唯一真源**（`PromptBuilder` 只读此文件 + 蓝图 `meta` 元数据；**不**接入 `prompts/system.md`） |
| `memory_seed.json` | 可选、创作者维护的只读前置记忆；与用户运行时 LTM、STM、聊天记录分离，详见 [`ROLE_PACK_SPEC`](../creator-docs/role-pack/ROLE_PACK_SPEC.md#persona--memory-独立迁移契约) |
| `prompts/` | **可选创作辅助**：`reply_quality_anchor.md` 人类可读镜像（Stable v4 运行时 SSOT 为 `runtime_config.reply_quality_anchor`；v2 兼容 `meta`；否则用内核默认）、creator profile 校验目录；**非** Tier0 人设来源 |
| `scenes/{id}/` | 场景 `scene.json`、`description.txt` 等 |
| `knowledge/` | 世界观 Markdown（内容向） |
| `assets/` | 立绘、头像等；**v0.4+ 草案**：`config.json` → `portrait_catalog` 指向 `assets/images/` 等路径（见 [RFC_PORTRAIT_FACILITY.md](../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md)） |
| `config.json` | 可选引擎参数：`memory` / `relation` / `turn_thinking`（Wave F 路由 + ephemeral，见 [ROLE_PACK_SPEC §9.11](../creator-docs/role-pack/ROLE_PACK_SPEC.md#911-turn_thinkingwave-f-co-present-路由)） |
| `ui.json` | **前端布局**（非后端；见 CONFIGURATION_FILES） |
| `author.json` | 作者元数据、推荐插件（须用户确认才生效） |

### 2.3 创作者不应直接改（属蓝图）

v2 兼容包可能把系统配置写在 **`meta`**；Stable v4 必须只写 **`runtime_config`**，宿主对 `meta.*` 的读取仅用于旧包回退：

- 已迁至 **`runtime_config.*`**（见 §3.3）：`interaction_mode`、`memory_config`、`reply_quality_anchor`、`remote_fallback_to_builtin`（包级建议）、`dual_core` 等
- 过渡期仍可能出现在 **`meta.*`**（宿主只读兼容）

**禁止**创作者包内单独开启双核（见 §5.1）。

---

## 3. 蓝图专属（系统配置）

### 3.1 `pipeline.ocblueprint` 蓝图段

| 键 / 段 | 说明 |
|---------|------|
| `slot_registry` | 多实例槽：`type`、`backend`、`plugin`、`model`、`url`、`position`… |
| `groups` | 架构图分组（可选） |
| `pipeline` | 双核 RFC：`stable` / `experimental` + `depends_on`（**schema v3 · Proposed**） |
| `slot_registry.*.zone` | 双核归属（**Proposed**） |

**禁止落盘**：`module_relations`、`steps`、`entry`（校验报错；运行时派生）。

**`blueprint/` 卫星目录**（可选）：`includes/`、`overlays/`、`revisions/`、`docs/` — **不**替代 `pipeline.ocblueprint` 路径；专家文档放此处**不影响**默认蓝图校验（详见 [BLUEPRINT_FOLDER_LAYOUT.md](./BLUEPRINT_FOLDER_LAYOUT.md)）。

### 3.2 通用蓝图扩展外壳（Stable v4）

通用扩展沿用“底座归 OCLive、载荷归扩展作者”的原则，但不把第三方字段不断追加到蓝图根：

| OCLive 维护 | 扩展作者维护 |
|-------------|--------------|
| `extensions` 容器、实例 ID、`capability`、可选 `provider`、`required`、安全 `config_ref`、缺失/降级语义 | `config_ref` 指向的载荷 schema、实现、UI、迁移、许可证、文档与支持 |

- **角色内容扩展**（例如 Chat Pro `adult_extension.json`）与**蓝图能力扩展**是两种契约；可以使用同一分责原则，但不得互相冒充。
- 蓝图只声明能力意图；宿主把蓝图、`HostProfile`、用户设置和能力注册表编译为进程内 `ExecutionPlan`。
- 使用共享 GPU/内存/进程的能力另接 Resource Adapter；纯文本或纯配置扩展不需要资源适配器。
- 未知可选扩展须保留并可见降级；未知必需扩展允许查看角色以修复，但不得激活该蓝图。
- v4 已实现外壳、路径安全、required/optional 与编写器 round-trip；v2/v3 仍严格拒绝该字段。
- Capability Registry 尚未落地：可选声明保留但暂不执行；必需声明阻止激活。结构化可见降级仍是后续工作。

完整边界与接入闭环只维护于 [蓝图扩展与资源协调 RFC](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)，本文不复制其字段和资源协议。

### 3.3 `runtime_config`（Stable v4 SSOT；v3 双核 Beta 兼容）

| 子字段 | 说明 |
|--------|------|
| `interaction_mode` | `immersive` \| `pure_chat` |
| `memory_config` | 记忆权重与场景策略 |
| `reply_quality_anchor` | 回复质量锚点全文 |
| `remote_fallback_to_builtin` | 包级 Remote 降级建议（宿主全局仍以 `app_settings` 为准） |
| `dual_core.enabled` | 双核开关，默认 **`false`** |
| `identity_binding` / `evolution` / `ollama_model` / `remote_presence` / `autonomous_scene` | 引擎策略（可选） |

v2 文件若含 `runtime_config`：`pack validate` **警告并忽略**；稳定蓝图请升 **`schema_version: 4`**，只有双核 Beta 使用 v3。

### 3.4 自 `settings.json` 剥离的引擎字段（legacy → 蓝图）

| legacy `settings.json` | canonical 蓝图落点 |
|------------------------|-------------|
| `plugin_backends` | `slot_registry` |
| `interaction_mode` | **`runtime_config.interaction_mode`**；v2 仅兼容 `meta.interaction_mode` |
| `memory_config` | **`runtime_config.memory_config`** |
| `evolution`（引擎参数） | **`runtime_config.evolution`** |
| `remote_presence` / `autonomous_scene` | **`runtime_config.*`** |
| `ollama_model` | `slot_registry` 中 `type: llm` 的 `model` 或 `runtime_config.ollama_model` |

### 3.3 不属于角色包、也不属于角色蓝图文件

| 配置 | 落点 |
|------|------|
| `remote_fallback_to_builtin` | 宿主 **`app_settings`** / `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` |
| Monolith `weld_modules` | 工程根 **`monolith.toml`**（不随角色包分发） |
| 目录插件 **`permissions`** | 插件 **`manifest.json`** + 用户 **`high_risk_grants.json`** |
| MCP server | `{app_data}/mcp-servers/*.json` + 用户授权 |
| GPU/内存预算、租约与抢占 | 宿主 Resource Coordinator + `HostProfile` / 用户本机策略；蓝图只声明能力和降级意图 |

---

## 4. 迁移与校验（路线图）

| 项 | 今日 | 目标 |
|----|------|------|
| 文件 | 单文件 `pipeline.ocblueprint` | 可选拆 `role.meta.json` + `pipeline.ocblueprint`（未排期） |
| 引擎字段 | v2 兼容读取 `meta.*` | v4 顶层 **`runtime_config`**，禁止与 `meta` 双写 |
| CLI | `pack validate` 全量 v2/v3/v4 | **`--profile creator`** 已实现（§2 子集 + `prompts/`；**不**校验 `slot_registry` / `pipeline`） |
| 编写器 | 新建 v4；导入 v2 后无损保持 v2 | 默认「角色」视图 / 高级「蓝图」视图 |

**`--profile creator` 与完整示例包**：`distros/chat-pro/roles/mumu` 等**完整示例包**含 evolution、`slot_registry` 与引擎向字段，应用**默认** `pack validate`（全量 v2/v3/v4）。对 **`--profile creator`** 会失败 — **不是 bug**，说明该包超出「纯创作者子集」。验证 creator profile 请用 `pack create` 生成的最小包或仅含 §2 字段的包。

**v2 / v3 / v4 并存**：宿主不自动改写旧包；编写器导入 v2 后仍以 v2 导出，新建包默认 v4。

---

## 5. 双核与角色包

### 5.1 双核启用条件

| 决议 | 说明 |
|------|------|
| **归属** | **蓝图** `runtime_config.dual_core.enabled`，**非**角色包字段 |
| **默认** | **`false`**；与 Remote 降级一样对终端用户**静默** |
| **创作者** | **不得**在面向初级创作者的分发包中单独置 `enabled: true` |
| **开启方** | 宿主管理员、`oclive init --dual-core` 工程模板、集成方蓝图 |
| **legacy** | **`settings.json` 不含** `dual_core` |

### 5.2 Experimental 核与角色包

| 项 | 说明 |
|----|------|
| **角色包** | 只提供 Stable 灵魂（`meta` 子集、`prompts/`、`scenes/` 内容） |
| **Experimental** | `pipeline.experimental` + 开放 `type` 由**开发者蓝图**配置，非入门创作者职责 |
| **P4 运行时** | 仅 **`PluginHost` 七种 type** 可执行；其余 type 校验可过、运行时报未实现（Q20） |
| **省略 `pipeline.stable`** | Stable 仍走 **`co_present` 硬编码**（Q19） |

详见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md)。

---

## 6. 加载链（供 Bus factor）

```text
distros/chat-pro/roles/{id}/pipeline.ocblueprint
  ├─ meta（创作者子集 + 过渡期引擎字段）
  ├─ runtime_config（v4 Stable 系统配置 SSOT；v3 双核 Beta 兼容）
  ├─ slot_registry（蓝图）
  ├─ groups / includes（蓝图）
  ├─ pipeline（仅 v3 双核 Beta）
  └─ extensions（仅 v4；可选保留、必需声明在当前阶段阻止激活）
        ↓
Capability Registry / Plan Compiler（目标）→ SlotResolver / PluginHost → process_message
```

会话 **`set_session_plugin_backend`** 覆盖槽位枚举，**不写回**角色包；高危能力仍走 **插件 manifest + grants**。

---

[English summary](../creator-docs-en/role-pack/ROLE_PACK_SPEC.md#0-role-pack-vs-blueprint-boundary)
