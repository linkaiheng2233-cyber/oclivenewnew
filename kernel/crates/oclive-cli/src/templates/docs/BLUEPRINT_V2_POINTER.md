# pipeline.ocblueprint（v2/v3/v4）

角色包 **SSOT** 为 `pipeline.ocblueprint`；新 Stable 包用 `schema_version: 4`，v2 保持兼容，v3 仅为冻结的双核 Beta。

权威文档（oclivenewnew 仓库）：

- [ROLE_PACK_SPEC.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/role-pack/ROLE_PACK_SPEC.md)
- [V1_TO_V2_MIGRATION.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/role-pack/V1_TO_V2_MIGRATION.md)

校验：`cargo run -p oclive-cli -- pack validate <角色根>`（按声明版本精确分派）。

编排运行时以宿主 `process_message` 为准，**不**使用蓝图 `steps[]` DSL。
