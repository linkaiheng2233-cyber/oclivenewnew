# 世界观知识（角色包资源）

本页说明 **共景主对话** 如何加载 `roles/{role_id}/knowledge/` 下的 Markdown、注入 Prompt，以及 **`event_hints`** 如何补充 [`EventDetector`](../../src-tauri/src/domain/event_detector.rs) 的关键词（与 Remote 插件 `plugin_backends` **无关**）。

## 目录与启用规则

- **目录名**：固定为 **`knowledge/`**（位于角色包根目录旁，与 `manifest.json` 同级）。
- **自动启用**：`manifest.json` **未写** `knowledge` 字段时，若存在 `knowledge/` 目录，则加载其下所有 `.md`（递归）。
- **显式关闭**：在 manifest 或 `settings.json` 中写 `"knowledge": { "enabled": false }`，即使存在目录也不加载。
- **显式启用且无文件**：若 `enabled: true` 但 glob 下没有 `.md`，加载失败（避免半包静默）。

## `manifest.json` / `settings.json` 可选块

```json
"knowledge": {
  "enabled": true,
  "glob": "knowledge/**/*.md"
}
```

- **`glob`**：须以 **`knowledge/`** 开头；当前实现递归枚举 `knowledge/` 下全部 `.md`（与 `**/*.md` 约定一致）。
- **`settings.json`** 中的 `knowledge` 会 **覆盖** manifest 合并后的同名字段（见 [`DiskRoleSettings::apply_to_manifest`](../../src-tauri/src/models/role_settings_disk.rs)）。

## Markdown 与 YAML front matter

每个 `.md` 文件 **必须** 以 front matter 开头，否则加载报错。

```markdown
---
id: lore_city
tags: [雾城, 主线]
scenes: [home]
weight: 1.0
event_hints:
  quarrel:
    keywords: ["决裂", "分手"]
  praise:
    keywords: ["神作"]
---

正文：与设定相关的叙述，参与检索与 Prompt 拼接。
```

| 字段 | 必填 | 说明 |
|------|------|------|
| `id` | 是 | 包内唯一；用于 Prompt 中标注来源。 |
| `tags` | 否 | 参与检索加分。 |
| `scenes` | 否 | 省略或空表示全场景；否则仅当当前 `scene_id` 命中列表时参与检索。 |
| `weight` | 否 | 默认 `1.0`，与检索分数相乘后排序。 |
| `event_hints` | 否 | 键为事件类型：`quarrel` / `apology` / `praise` / `complaint` / `confession` / `joke` / `ignore`；值为 `keywords` 数组（可选 `weight` 预留）。 |

## 运行时行为（摘要）

1. **加载**：[`RoleStorage::load_role_from_dir`](../../src-tauri/src/infrastructure/storage.rs) 在校验通过后解析知识，写入 **`Role::knowledge_index`**（`Arc`，仅内存）。
2. **检索**：对用户句做轻量重合打分 + `scenes` 过滤，取 Top-K；拼接为 **【世界观设定】** 段。**共景**见 [`PromptBuilder::build_prompt`](../../src-tauri/src/domain/prompt_builder.rs)（位于日程推断之后、长期记忆之前）。**异地心声**（`remote_life`）见 [`build_remote_life_prompt`](../../src-tauri/src/domain/remote_life_prompt.rs)：检索时以**角色所在场景** `character_scene_id` 过滤，与共景下用当前同场景 `scene_id` 的规则对称。
3. **事件**：检索到的块合并为 [`KnowledgeEventAugment`](../../src-tauri/src/models/knowledge.rs)，传入 [`EventDetector::detect_with_augment`](../../src-tauri/src/domain/event_detector.rs) 与 `estimate_event_impact` 的规则回退路径（B1：补充关键词，不替换内置情绪门控逻辑）。异地心声路径固定 `Ignore` 事件估计，不因知识块再跑一轮事件管线。

## 调试建议

- 加载失败时错误信息包含文件路径与解析原因（front matter / 重复 `id` / 未知 `event_hints` 键等）。
- 关闭 LLM 事件估计（`OCLIVE_EVENT_IMPACT_LLM=0`）时较易观察纯规则 + `event_hints` 的差异。

## 相关文档

- [PACK_VERSIONING.md](./PACK_VERSIONING.md) — 版本与 `knowledge` 契约  
- [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — 可替换子系统（世界观知识 **不是** 插件后端）
