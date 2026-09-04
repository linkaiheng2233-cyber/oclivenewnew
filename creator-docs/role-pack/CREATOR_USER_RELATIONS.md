# 创作者说明：用户关系、身份模板与初始好感

角色包「用户自定义」总览见 [《角色包定制指南》](CREATOR_ROLE_PACK_CUSTOMIZATION.md)，完整格式以 [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) 为准。

这三个概念需要分开：

- **关系**：`pipeline.ocblueprint` → `meta.relations`，描述互动关系、好感初值与倍率；
- **用户身份模板**：可选 `user_identities/`，描述“用户是谁”，并可映射到一个关系；
- **当前选择**：宿主把全局/场景身份 id 持久化在 SQLite 角色运行态，不写回角色包，也不是六槽 SessionCache 覆盖。

legacy `manifest.json.user_relations` 只是 `meta.relations` 的旧名字，以下示例以当前参考宿主蓝图格式族的新 Stable 写法为准；这不规定其它发行版的产品包布局。

## 关系键与展示名

`relations` 的键（如 `friend`、`classmate`）是存档、API 和映射使用的稳定 id。发布后不要随意改名。`display_name` 是可选的界面文案；省略或留空时，界面可回退显示 id。

```json
{
  "schema_version": 4,
  "meta": {
    "relations": {
      "friend": {
        "display_name": "好友",
        "prompt_hint": "你们是好朋友，说话随意亲密",
        "favor_multiplier": 1.0,
        "initial_favorability": 45
      }
    },
    "default_relation": "friend"
  }
}
```

这里省略了蓝图其它必填字段；可复制完整 v4 样例再编辑。

## `default_relation`

- 非空时必须引用 `meta.relations` 中存在的键。
- 用于尚无更具体关系映射时的默认关系。
- 它不是 `user_identities/index.json.default_identity_id`：前者选关系，后者选用户身份模板。

## 好感字段

- `favor_multiplier`：好感变化倍率，必须是有限且大于 0 的数。
- `initial_favorability`：首次建立该用户—角色关系时的初始好感，必须在 0～100。

## 让身份映射到关系

需要更完整的“用户是谁”描述时，在 `user_identities/index.json` 的身份条目中写 `maps_to_relation_id`：

```json
{
  "schema_version": 1,
  "default_identity_id": "classmate_user",
  "identities": {
    "classmate_user": {
      "display_name": "同班同学",
      "template_file": "classmate.md",
      "maps_to_relation_id": "classmate"
    }
  }
}
```

模板正文放在 `user_identities/classmate.md`。无 `user_identities/` 时，宿主可回退使用关系里的 `prompt_hint`；这只是兼容行为，不代表关系与身份是同一个概念。当前选择与发行版限制见 [用户身份 RFC](../rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md)。

## `memory_config.topic_weights` 与场景

`topic_weights` 的顶层键必须是已声明的场景 id。场景集合来自蓝图 `meta.scenes` 与角色包 `scenes/<scene_id>/` 子目录的合并结果，否则角色加载会失败。

- Stable v4：写在 `runtime_config.memory_config.topic_weights`；
- v2 兼容包：仍可从 `meta.memory_config.topic_weights` 读取；
- legacy 包：按 [V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md) 迁移，不要与蓝图双写。

## 校验

```powershell
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<角色 id>
```

默认校验会按蓝图声明的 v2/v3/v4 精确分派。关系 id、默认关系、数值范围、场景引用和身份模板文件错误都会被报告；校验失败不会被静默忽略。

[English](../../creator-docs-en/role-pack/CREATOR_USER_RELATIONS.md)
