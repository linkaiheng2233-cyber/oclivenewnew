# 发版版本号策略（独立发版）

**结论**：**桌面应用、CLI、内核 crate 保持独立 SemVer**，首次对外发布不强行统一到同一数字。

**SSOT 范围**：独立版本与候选版映射规则。当前数值、源码 manifest 和验收状态只在 [PROJECT_CURRENT_STATUS](../getting-started/PROJECT_CURRENT_STATUS.md) 维护。

| 产物 | 发版节奏 | 说明 |
|------|----------|------|
| **桌面 Tauri** | 用户可见功能与安装包 | 未发行变更留在 [CHANGELOG](../../CHANGELOG.md) `[Unreleased]` |
| **`oclive-cli`** | 脚手架与工具链 | Breaking 见 [DEPRECATED_COMMANDS](../../kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md) |
| **`oclive_kernel_runtime`** | runtime 与 HTTP/`--api` 契约 | 见 [COMPATIBILITY](../COMPATIBILITY.md) |
| **`oclive_validation`** | 角色包 / 蓝图校验 | 与编写器 wasm 校验能力对齐 |

## 为何不同步 bump

- 桌面发版含前端与安装器；CLI 可单独 `cargo install` 更新。
- 内核 crate 被多宿主（Tauri、`kernel_server`、未来启动器）引用，**契约变更**应独立于 UI 发版。
- [COMPATIBILITY.md](../COMPATIBILITY.md) 已用 **`min_runtime_version`** / `API_VERSION` 表达跨产物兼容，而非单一全局版本。

## 当前发布候选版

1. 候选版编号、基线与是否公开只查 [现状快照](../getting-started/PROJECT_CURRENT_STATUS.md)。本地回滚标签不等于公开发行；发布前冻结目标提交并完成适用门禁和该提交的远程 CI。
2. Windows MSI 使用 Tauri 配置中的数值版本，桌面使用对应 SemVer；映射数值在现状快照登记。CLI 与内核不随桌面版本重复升级，不能因去掉预发行后缀而把 MSI 数值回退。
3. Breaking 角色包：必须链 [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md)。

## 后续

- 仅当 **`oclive_kernel_runtime` 主版本** 升级时，同步检查 `min_runtime_version` 与 OOCP 套件。
- Monolith / 嵌入式交付物版本跟随**生成该二进制的 CLI + 模板**版本记录于 `bench_history.json`（本地，不提交）。

[English](../../creator-docs-en/development/RELEASE_VERSIONING.md) · [DOCUMENTATION_INDEX](../getting-started/DOCUMENTATION_INDEX.md)
