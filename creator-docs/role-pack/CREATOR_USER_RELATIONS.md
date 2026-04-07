# 创作者说明：用户身份与初始好感

角色包「用户自定义」总览教学见 [《角色包用户自定义创作者教学》](./CREATOR_ROLE_PACK_CUSTOMIZATION.md)。

本文说明角色包 `manifest.json` 里 **`user_relations`**（用户身份）相关字段，以及加载时的校验规则。

## 关系键 `id` 与展示名 `display_name`

- **`user_relations` 的键**（如 `friend`、`classmate`）是程序内部使用的 **英文标识**，用于存档、API、默认关系等，请保持稳定、勿随意改名。
- **`display_name`**（可选）：界面下拉框、关系预览等处展示的 **中文或其它展示文案**。若省略或留空，则展示名与键相同（即显示英文键）。
- 导出角色时，若某身份的展示名与键不同，会写出 `display_name` 字段；相同则省略，保持 JSON 简洁。

示例：

```json
"user_relations": {
  "friend": {
    "display_name": "好友",
    "prompt_hint": "你们是好朋友，说话随意亲密",
    "favor_multiplier": 1.0,
    "initial_favorability": 45
  }
}
```

## `default_relation`

- 必须对应 **`user_relations` 中存在的键**（若填写了非空字符串）。
- 用于新对话或未指定关系时的默认身份。

## `favor_multiplier` 与 `initial_favorability`

- **`favor_multiplier`**：好感变化倍率，须为 **有限且大于 0** 的正数。
- **`initial_favorability`**：该身份下、**首次建立用户—角色关系**时的初始好感（0～100）。须为有限数字；加载时会再约束到合法区间。

## `memory_config.topic_weights` 与场景

- `topic_weights` 的 **顶层键必须是场景 id**，且该场景须出现在以下至少一处：
  - `manifest.json` 顶层 **`scenes`** 数组，或
  - `roles/{角色id}/scenes/` 下 **子目录名**（与 manifest 顶层 `scenes` 合并、去重后的场景 id 列表一致）。
- 否则会加载失败，并返回 **中文错误说明**（便于修正 manifest）。

## 校验时机

在从目录加载角色（读取 `manifest.json` 并转为运行时 `Role`）时执行上述校验；校验失败时不会静默忽略，请根据提示修改包内配置。
