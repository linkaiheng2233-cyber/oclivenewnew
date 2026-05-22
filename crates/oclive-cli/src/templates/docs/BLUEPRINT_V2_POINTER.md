# pipeline.ocblueprint（v2）

角色包 **SSOT** 为 `pipeline.ocblueprint`（`schema_version: 2` · `meta` · `slot_registry`）。

权威文档（oclivenewnew 仓库）：

- [ROLE_PACK_SPEC.md](https://github.com/oclive-app/oclivenewnew/blob/main/creator-docs/role-pack/ROLE_PACK_SPEC.md)
- [V1_TO_V2_MIGRATION.md](https://github.com/oclive-app/oclivenewnew/blob/main/creator-docs/role-pack/V1_TO_V2_MIGRATION.md)

校验：`cargo run -p oclive-cli -- pack validate <角色根>`（默认 v2 profile）。

编排运行时以宿主 `process_message` 为准，**不**使用蓝图 `steps[]` DSL。
