# 蓝图 v2 实施计划

| 项 | 值 |
|----|-----|
| 状态 | **已确认 / 可以落实**（决议见 [BLUEPRINT_V2_DECISIONS.md](BLUEPRINT_V2_DECISIONS.md)） |
| RFC | [RFC_ROLE_BLUEPRINT_V2.md](RFC_ROLE_BLUEPRINT_V2.md)（**Accepted**） |
| 当前阶段 | **P0–P8 已收口**（2026-05-20）；远程 CI 绿线见 [§8](#8-p8--ci-与验收) |

---

## 1. 范围与边界

### 1.1 本计划要做

- 新角色包格式：`pipeline.ocblueprint` `schema_version: 2`
- `slot_registry` 开放多实例 + 七类 `type` 合并策略（**P4 起生效**）
- `meta` 合并原 manifest/settings 字段；**七维人格仅搬家**（Q1）
- 会话覆盖：内存 registry + 架构图交互（§RFC §6）
- `oclive_validation` + JSON Schema + CLI `blueprint validate` / `pack validate` 适配
- 迁移工具 + `roles/*` 改写（P6∥P2）
- 架构图蓝图驱动（P5）

### 1.2 本计划不做（或延后）

- **插件 UX 简化（2026-05）**：主应用默认 **SimplePluginManager**（已安装列表）；`slot_attachment` + `oclive plugin install --role` 自动装配；架构图/多槽位默认迁至 **`oclive plugin manage`**（`--tui`）。见 [PLUGIN_V1.md](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)「简洁插件管理」。

- VAD / Big Five 人格改造（未来 RFC）
- `co_present` 动态多实例（**P4**，确认前不启动）
- pack-editor / launcher UI（文档预留）
- 用户可新增第 8 种 `type`
- 蓝图 DSL `steps[]` 恢复为主路径

### 1.3 与预研代码的处置

| 已有提交/代码 | 处置 |
|---------------|------|
| Vue Flow 架构图、暗色网点 | **保留**画布 |
| `useArchitectureGraphConnections` 手拖连线 | **P5 删除主路径** |
| `useArchitectureGraphModel` 六槽固定拓扑 | **P2/P5 改为读 registry** |
| `archGraphConnections.ts` 固定拓扑校验 | **P5 改为 registry 规则或删除** |

---

## 2. 数据契约摘要

### 2.1 `meta.personality`（Q1）

```json
"personality": {
  "stubbornness": 0.5,
  "clinginess": 0.5,
  "sensitivity": 0.5,
  "assertiveness": 0.5,
  "forgiveness": 0.5,
  "talkativeness": 0.5,
  "warmth": 0.5
}
```

或 `"personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]`（迁移用）。

加载时映射到现有 `PersonalityDefaults` → `PersonalityVector`，**无公式变更**。

### 2.2 `slot_registry` 实例键

- 键：用户定义（`memory_short_term`、`emotion`、`llm`…）
- 值：`type`, `label`, `backend`, `position`, 可选 `plugin` / `plugins` / `model` / `url` / `local_memory_provider_id`

### 2.3 校验规则（P1）

- `schema_version === 2`
- `meta.id` 非空；建议与目录名一致（WARN/ERR 可配置）
- `meta.personality` 七维合法
- `meta.relations` / `default_relation` 结构合法
- `slot_registry` 至少一个 `type: llm`
- 每个槽位 `type` / `backend` 枚举合法；directory 必填 `plugin` 或 `plugins`
- **`module_relations` 禁止出现在文件中**（B3=C）；仅运行时由 `slot_registry` 派生

---

## 3. 分阶段路线图

```
P0  Accepted RFC + 本计划确认     [x]
P1  Schema + oclive_validation + CLI validate（不动 co_present）  [x]
P2  蓝图加载 → Role + SessionSlotRegistry  [x]
P3  PluginHost 按槽实例解析  [x]
P4  co_present 多实例执行器  [x]
P5  架构图 + 会话覆盖 + 写盘工具栏  [x]
P6  roles 迁移（与 P2 同迭代）  [x]
P7  文档 + CHANGELOG Breaking  [x]
P8  CI（pack validate 默认 v2、OOCP、全量测）  [x]
```

### P5 交付摘要（架构图写盘）

| 能力 | 说明 |
|------|------|
| `save_role_slot_registry` | Tauri 命令；经 `oclive_validation::write_role_pack_blueprint_slot_registry` 写回 `pipeline.ocblueprint` → `slot_registry` |
| 工具栏 | 添加槽位（`ArchAddSlotDialog`）、删除槽位；**至少保留一个 `type: llm`**，最后一个 llm 不可删 |
| 前端 | `ArchitectureGraphFlow.vue` + `src/lib/slotRegistry.ts`；Vitest `src/__tests__/slotRegistry.test.ts` |
| 缓存 | 写盘后 `invalidate_role_cache` + `load_role` 刷新 |
| **会话覆盖 UI（2026-05-20 收尾）** | 蓝图节点：**「仅本次覆盖」** → `set_session_slot_override`；**「设为包默认」** → `save_role_slot_registry`；**「清除覆盖」** → `clear_session_slot_override`；虚线框 + 文案已接满 |
| **V2 轻量卡片** | `usePluginManagerV2` 读 `slot_registry_effective`；完整写盘/增删在 V1 架构图 |

### P8 — CI 与验收

| 检查项 | 说明 |
|--------|------|
| `pack validate` 默认 | **BlueprintV2**；`--profile legacy` 仅旧包 |
| OOCP 黄金包 | `roles/mumu`（v2 `pipeline.ocblueprint`）；CI **`oocp-test-suite`** |
| Tauri 1.5 Linux | **`rust` / `oocp-test-suite`** 使用 **`ubuntu-22.04`** + `libwebkit2gtk-4.0-dev`（非 4.1） |
| `generate_context!` | **`rust`** job 在 clippy/test 前先 **`npm ci && npm run build`**（需 `dist/`） |
| 阻塞 job | `rust`×2、`frontend`、`oocp-test-suite`、`cli`、`cli-bench`；`cargo-audit` / `remote-plugin-demo` 可失败 |

维护者本地：`cargo test -p oclive_validation`、`cargo test -p oclivenewnew-tauri`、`npm run test:unit`、`pack validate ./roles/mumu`。

### P1 — 首批代码（确认后唯一自动开工项）

| 任务 | 产出 |
|------|------|
| `crates/oclive_validation/src/blueprint_v2.rs` | Rust 校验 |
| `crates/oclive-cli/schemas/pipeline.ocblueprint.v2.schema.json` | JSON Schema |
| `oclive-cli blueprint validate` | 仅接受 v2；移除/废弃 steps 校验路径 |
| `oclive-cli pack validate` | **默认** v2（`pipeline.ocblueprint`）；`--profile legacy` 保留 manifest/settings 路径 |
| 单测 | 合法/非法样例 JSON |

**明确不做**：`role_pack.rs` 加载、`co_present`、`PluginHost` 改造。

### P2 — 加载与会话覆盖（与 P6 并行）

| 任务 | 产出 |
|------|------|
| `BlueprintLoader` | 解析 v2 → `RoleBlueprint` |
| `Role` 构建 | `meta` → 原 `Role` 字段（含 `default_personality` 从 `meta.personality`） |
| `SessionSlotRegistry` | `HashMap<slot_key, SlotEntry>` + overlay |
| API | `set_session_slot_backend(slot_key, …)` 替代/包装 `set_session_plugin_backend` |
| `get_role_info` | `effective_slots` + `sources` + per-key 覆盖标记 |

### P3 — 解析与插件

| 任务 | 产出 |
|------|------|
| `SlotResolver` | 按实例解析 `Arc<dyn MemoryRetrieval>` 等 |
| `CompositeAgentProvider` | 多 `plugins[]` + 多 agent 槽合并 tools |
| `ComplexEmotionProvider` | directory / remote 分支 |
| `provides` 校验 | 含 `complex_emotion` |

### P4 — 编排

| 任务 | 产出 |
|------|------|
| `SlotRunner` per type | 实现 §RFC 合并表 |
| `co_present` 改造 | 阶段表内调用 Runner |
| 集成测 | 双 memory、双 llm last-wins 等 |

### P5 — 架构图

| 任务 | 产出 |
|------|------|
| 节点 | 每 registry 键一节点；颜色 by `type` |
| 边 | 派生 + 固定内核→总线示意（可选简化） |
| 工具栏 | 添加槽位 / 添加插件 / 多对一 |
| 节点点击 | 后端面板 + 本次覆盖 + 重置默认 |
| 移除 | 手拖 `connect`、自由 `edges-change` 写拓扑 |

### P6 — 迁移

| 任务 | 产出 |
|------|------|
| `oclive-cli pack migrate-to-blueprint` | manifest+settings → v2 |
| 改写 `roles/mumu` 等 | CI 黄金包 |

### P7 — 文档

- `ROLE_PACK_SPEC.md` v2、`PLUGIN_V1.md`、`OCLIVE_ARCHITECTURE_OVERVIEW.md`
- `SETTINGS_REFERENCE.md` → 指向蓝图
- `CHANGELOG.md` Breaking 条目

### P8 — CI

- `cargo test -p oclive_validation`
- `cargo test -p oclivenewnew-tauri`
- OOCP + `invoke_hotpath_matrix` 在新黄金包上通过

---

## 4. 会话覆盖 — API 与前端（RFC §6 落地）

### 4.1 后端

| 现 API | 新 API（草案） |
|--------|----------------|
| `set_session_plugin_backend(role_id, module: CoreModule, backend)` | `set_session_slot_override(role_id, slot_key, patch: SlotOverridePatch)` |
| `plugin_backends_effective` | `slot_registry_effective` + `slot_overrides: Record<key, bool>` |

`SlotOverridePatch`：`backend?`, `plugin?`, `plugins?`, `model?` …

### 4.2 前端（插件工作台 · 架构图）

- [x] 节点 `data`：`slotKey`, `type`, `backend`, `sessionOverridden`
- [x] 蓝图节点内按钮：**仅本次覆盖** / **设为包默认** / **清除覆盖**（`ArchModuleNode.vue` + `set_session_slot_override`）
- [x] 视觉：虚线边框 + **「本次覆盖」**
- 独立 `ArchSlotBackendPanel.vue`：未拆文件；逻辑在节点 + `archGraphActionsKey`

### 4.3 模块节点下拉（过渡期）

现有 `ArchModuleNode` 按 **六槽 enum** 的下拉在 P5 改为按 **实例键** 配置；与蓝图工具栏并存。

---

## 5. 风险与依赖

| 风险 | 缓解 |
|------|------|
| P2 改动面大（加载链） | P1 Schema 先冻结；样例蓝图进仓库 |
| 多 llm last-wins 行为变化 | OOCP 用新黄金包重录预期 |
| `set_session_plugin_backend` 调用方多 | 保留薄包装层映射 slot_key（六槽默认键名） |
| 编写器未适配 | 先 CLI + 主应用架构图；pack-editor 里程碑独立 |

---

## 6. R1–R4（已关闭）

见 [BLUEPRINT_V2_DECISIONS.md](BLUEPRINT_V2_DECISIONS.md)。

---

## 7. 确认清单

- [x] RFC Accepted 与 Q1–Q8  
- [x] P1 范围（仅 Schema/validation）  
- [x] P2–P8 顺序（P6∥P2）  
- [x] 会话覆盖交互（§4）  
- [x] P5 移除手拖连线  
- [x] R1–R4  

### Git 提交提醒

| 顺序 | 建议 commit message | 内容 |
|------|---------------------|------|
| 1 | `docs(handoff): blueprint v2 decisions frozen` | 决议 + RFC/计划状态 |
| 2 | `feat(validation): pipeline.ocblueprint v2 schema and CLI validate` | `blueprint_v2.rs`、Schema、CLI、`pipeline.ocblueprint.template`、测试 |

**门禁**：`cargo test -p oclive_validation`、`cargo test -p oclive-cli`。

---

## 附录 A — `meta` 字段迁移对照（P2/P7）

| 原位置 | 新位置 `meta.*` |
|--------|-----------------|
| `manifest.id` | `id` |
| `manifest.name` | `name` |
| `manifest.version` | `version` |
| `manifest.author` | `author` |
| `manifest.description` | `description` |
| `manifest.default_personality` | `personality`（Q1） |
| `manifest.user_relations` | `relations` |
| `manifest.default_relation` | `default_relation` |
| `manifest.scenes` | `scenes` |
| `manifest.evolution` | `evolution` |
| `manifest.memory_config` | `memory_config` |
| `manifest.life_schedule` 等 | 同名并入 |
| `settings.schema_version` | 由蓝图 `schema_version: 2` 取代 |
| `settings.plugin_backends` | `slot_registry` |
| `settings.interaction_mode` | `interaction_mode` |
| `settings.ollama_model` | `ollama_model` |

`settings.plugin_backends.directory_plugins` → 各 directory 槽的 `plugin` / `plugins`。
