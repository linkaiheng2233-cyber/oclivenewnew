# 从 v2 升级到 v3 蓝图

**目标读者**：已使用 `schema_version: 2` 的 `pipeline.ocblueprint`、需要 **`runtime_config`** 或可选 **双核** 的蓝图作者。按本文手动升级，**约 10 分钟**可完成校验与试聊。

**权威格式**：[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · 校验：`oclive_validation::blueprint_v3` · 双核：[DEVELOPER_GUIDE.md](../dual-core/DEVELOPER_GUIDE.md)

[English](../../creator-docs-en/role-pack/V2_TO_V3_MIGRATION.md)

---

## 1. v2 与 v3 核心差异

| 维度 | v2 | v3 |
|------|----|----|
| `schema_version` | `2` | `3` |
| 引擎 / 系统配置 | 多在 `meta.*`（过渡期宿主仍兼容） | 顶层 **`runtime_config`**（SSOT 目标） |
| 双核 | 无正式字段 | `runtime_config.dual_core` + 可选 `pipeline.stable` / `pipeline.experimental` |
| 槽位归属 | 无 `zone` | 可选 `slot_registry.*.zone`（`stable` / `experimental`） |
| 默认运行时 | `process_message` → `co_present` | **双核关闭时与 v2 行为一致**（零 diff） |

> **批量 CLI 迁移（Q18）仍延后**；当前推荐复制 v2 包 → 按下列步骤改 JSON → `pack validate`。

---

## 2. 手动升级步骤（10 分钟）

### 步骤 1：备份

```powershell
Copy-Item -Recurse roles\my_role roles\my_role.v2.bak
```

### 步骤 2：改 `schema_version`

在 `pipeline.ocblueprint` 根对象：

```json
"schema_version": 3
```

### 步骤 3：添加 `runtime_config`

将原 `meta` 中的**系统字段**迁入 `runtime_config`（v3 校验会读此段；v2 文件若含 `runtime_config` 仅警告）：

```json
"runtime_config": {
  "interaction_mode": "immersive",
  "memory_config": { },
  "reply_quality_anchor": null,
  "remote_fallback_to_builtin": true,
  "dual_core": { "enabled": false }
}
```

字段含义见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) 与 [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md) §3.3。

**最小 v3（不开双核）**：只需 `"runtime_config": { "dual_core": { "enabled": false } }` 或省略 `dual_core`（默认关）。

### 步骤 4：（可选）双核与 `pipeline`

仅当集成方显式开启双核时添加：

```json
"runtime_config": {
  "dual_core": { "enabled": true }
},
"pipeline": {
  "stable": [],
  "experimental": [
    { "action": "slot.emotion.analyze", "depends_on": [] }
  ]
}
```

- Stable 核今日仍走宿主 **`co_present`**；`pipeline.stable` 主要为文档与 Monolith 焊接。
- Experimental 步骤的 `action` 须能解析到 `slot_registry` 键；P4 运行时仅 **PluginHost 七种 type** 可执行。

### 步骤 5：（可选）`zone` 标记

```json
"slot_registry": {
  "llm": {
    "type": "llm",
    "zone": "stable",
    "position": 0,
    ...
  }
}
```

### 步骤 6：校验

```powershell
cargo run -p oclive-cli -- pack validate roles\my_role
cargo run -p oclive-cli -- doctor
```

`doctor` 对 v3 包输出 **`blueprint_v3_file_format`**、**`slot_registry_v3_llm`**、**`slot_position_v3_unique`**（不再误报 schema 2 不匹配）。

### 步骤 7：试聊

主应用加载角色或 HTTP `--api` 发一轮 `/chat`，确认 `reply` 非空。

---

## 3. 字段映射表（v2 → v3）

| v2 位置 | v3 位置 | 说明 |
|---------|---------|------|
| `meta.interaction_mode` | `runtime_config.interaction_mode` | 建议迁移；`meta` 残留仅兼容读 |
| `meta.memory_config` | `runtime_config.memory_config` | 同上 |
| `meta.reply_quality_anchor` | `runtime_config.reply_quality_anchor` | 同上 |
| `meta.remote_fallback_to_builtin` | `runtime_config.remote_fallback_to_builtin` | 同上 |
| `meta.remote_presence` 等 | 仍可在 `meta`（创作者/剧情向） | 非引擎 SSOT |
| `slot_registry` | 同名 + 可选 `zone` | 结构不变 |
| （无） | `pipeline.stable` / `pipeline.experimental` | 双核可选 |
| （无） | `runtime_config.dual_core.enabled` | 默认 `false` |

---

## 4. 双核可选性（重要）

| 配置 | 行为 |
|------|------|
| `schema_version: 3` 且 `dual_core.enabled: false`（或省略） | 与 v2 **同路径**编排，OOCP S0–S12 无回归 |
| `dual_core.enabled: true` 且 `pipeline.experimental` 非空 | 走 `DualPipelineRunner`；失败静默降级 Stable |
| 创作者分发包 | **不得**单独开启双核（见 [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md) §5.1） |

脚手架示例：`cargo run -p oclive-cli -- init --dual-core -o ./my-kernel`

---

## 5. FAQ

**Q：必须升到 v3 吗？**  
A：否。Stable 新包应使用 v4；只有明确启用双核 Beta 时才升 v3。现有 v2 包继续兼容且不会自动改写。

**Q：v2 里的 `meta.memory_config` 不迁会怎样？**  
A：v3 校验优先 `runtime_config`；仅留 `meta` 时宿主可能仍兼容读，但 `pack validate` 与编写器目标视图会以 `runtime_config` 为准。

**Q：没有 `migrate-v2-v3` 命令？**  
A：Q18 批量工具延后；用手动步骤或 `init --dual-core` 生成 v3 模板再合并 `meta` / 资源。

**Q：Monolith 工程怎么配？**  
A：`oclive init --monolith --dual-core`；见 [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)。

**Q：`pack validate --profile creator` 还要用吗？**  
A：纯创作者子集校验不变；完整 v3 包用默认 `pack validate`。勿用 `distros/chat-pro/roles/mumu` 测 creator profile（完整示例包）。

---

## 6. 相关链接

- v1 → v2：[V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md)
- 学习路径：[CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)
- 双核注册表：[METHOD_REGISTRY.md](../dual-core/METHOD_REGISTRY.md)
