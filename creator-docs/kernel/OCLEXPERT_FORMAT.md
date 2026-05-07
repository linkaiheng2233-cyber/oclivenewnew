# `.oclexpert` 专家图分享格式（Module 9）

面向创作者分享「专家模型设施」配置的标准 JSON 包，扩展名建议 **`.oclexpert`**（也可用 `.json` 保存以便编辑器识别）。

## 文件结构（`fileVersion: 1`）

```json
{
  "format": "oclexpert",
  "fileVersion": 1,
  "name": "可选展示名",
  "description": "可选，一两句说明这套配方的特点与效果",
  "author": "可选，创作者署名",
  "graph": {
    "version": 1,
    "nodes": [],
    "edges": []
  },
  "promptStyle": null
}
```

- **`format`**：必须为 `"oclexpert"`。
- **`fileVersion`**：当前仅 **1**；未来若破坏性变更会递增。
- **`name`**、**`description`**、**`author`**：可选字符串，供分享与导入预览；桌面 **专家模型工作台 → 导出 .oclexpert** 会写入；**编写器**侧若需与主程序一致，建议在 `roles/{id}/expert/default.oclexpert` 中同步维护（参见姊妹仓 `oclive-pack-editor` 的 `oclexpertPack` 与高级面板「专家模型」页）。
- **`graph`**：与内核 `ExpertGraph` 一致（`nodes[].type` 为 `snake_case`：`base_model`、`lora_adapter`、`cloud_model`、`event_trigger`、`prompt_style`；节点内字段为 **camelCase**，与桌面应用 `invoke` 载荷一致）。
- **`promptStyle`**：可选，与 `PromptStyleOverride` 一致（camelCase 字段）。

## 兼容导入

应用也接受 **裸 `ExpertGraph` JSON**（仅含 `version` / `nodes` / `edges`），便于从旧版「工作流 JSON」迁移。

## 校验规则（导入时）

- 每个 `nodes[]` 元素必须带已知 **`type`**；未知类型会拒绝导入并提示。
- **`fileVersion`** 高于应用支持范围时拒绝，并提示升级客户端。

## 与「工作流库」的关系

导入 `.oclexpert` 后，前端会将解析出的 `graph` + `promptStyle` 写入 **工作流库**（`expert_workflows_save`），便于在专家工作台继续编辑与版本管理。

## 应用行为提醒

- 点击 **「应用到当前会话」** 时，**会话覆盖 JSON 会先落盘**；侧车 `config_updated` 或云端切换失败时 **不会自动回滚** 已保存的图，但 Run 历史会记录 `applySidecarNotice` / 错误信息，可用 **Rollback** 或 **Run 历史** 手动恢复。
