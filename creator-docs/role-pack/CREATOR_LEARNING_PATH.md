# 角色包创作者学习路径

先完成 [30 分钟黄金路径](../getting-started/CREATOR_GOLDEN_PATH.md)。本页只做进阶导航，不重复 [角色包规范](ROLE_PACK_SPEC.md) 的字段定义。

## 按你正在做的事情阅读

| 目标 | 阅读与验收 |
|------|------------|
| 增加不同地点或情境 | [场景创作指南](CREATOR_SCENE_GUIDE.md)；切换场景后角色仍符合核心人设 |
| 增加世界观资料 | [世界观知识](WORLDVIEW_KNOWLEDGE.md)；重新加载角色后能检索到新知识 |
| 区分不同用户身份 | [用户身份 RFC](../rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md)；默认身份和场景身份不冲突 |
| 添加前置经历 | [角色包规范](ROLE_PACK_SPEC.md) 中的 `memory_seed.json`；确认不包含真实用户数据 |
| 调整核心与可变人设 | [人格档案说明](../../docs/personality-archive-notes.md)；核心正文不被运行时覆盖 |
| 配置七张基础立绘 | [Portable Core](ROLE_PACK_SPEC.md#portable-core)；用 `portable-core` profile 校验 |
| 维护旧包 | [v1 到 v2 迁移](V1_TO_V2_MIGRATION.md)；迁移后只保留一个格式真源 |
| 发布新版本 | [角色包版本管理](PACK_VERSIONING.md)；版本号和最低运行时要求匹配 |

锚点若因规范章节调整而失效，以 [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) 的目录为准。

## 三条边界

1. 创作者拥有角色身份、素材、场景和只读种子记忆。
2. 运行时拥有用户聊天、长期记忆和可变人设；这些数据不随公开角色包分发。
3. 发行版或插件开发者拥有槽位后端、权限和平台增强；基础角色包不依赖这些增强才能工作。

完整职责边界见 [角色包边界](../../handoff/ROLE_PACK_BOUNDARY.md)。

## 每次发布前

```powershell
cargo run -p oclive-cli -- pack validate <角色目录>
cargo run -p oclive-cli -- pack validate <角色目录> --profile portable-core
```

随后在目标发行版里完成实际导入和多轮试聊。CLI 的全部参数以 [OCLive CLI 指南](../cli/OCLIVE_CLI_GUIDE.md) 为准。
