# `roles/{id}/blueprint/` — 蓝图卫星目录

角色包根目录的 **`pipeline.ocblueprint`** 保持**瘦**：只放 `schema_version`、`meta`、`slot_registry`、`groups`、`runtime_config` 与 **`includes`** 拉取清单。

本目录存放蓝图相关、但**不应塞进本体 JSON** 的材料：

| 子路径 | 用途 |
|--------|------|
| `includes/` | 可被 `includes[].path` 引用的 JSON 片段 |
| `overlays/` | 专家设施 UI 定义（如 `expert.facility.blueprint`） |
| `revisions/` | 向导/专家 apply 的修订与降级快照 |
| `docs/` | 给人看的说明（宿主加载可忽略） |

角色文案仍在包根 **`prompts/`**、**`scenes/`**、**`core_personality.txt`**。

完整约定见 [handoff/BLUEPRINT_FOLDER_LAYOUT.md](../../handoff/BLUEPRINT_FOLDER_LAYOUT.md)。
