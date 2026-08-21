# 发版版本号策略（独立发版）

**结论**：**桌面应用、CLI、内核 crate 保持独立 SemVer**，首次对外发布不强行统一到同一数字。

| 产物 | 当前版本（`main`，2026-08-22） | 发版节奏 | 说明 |
|------|-------------------|----------|------|
| **桌面 Tauri**（`package.json` / `distros/desktop-tauri`） | **0.5.1** | 用户可见功能与安装包 | 变更写入 [CHANGELOG.md](../../CHANGELOG.md) `[0.5.1]` |
| **`oclive-cli`** | **0.1.0** | 脚手架与工具链 | CLI Breaking 见 [DEPRECATED_COMMANDS.md](../../kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md) |
| **`oclive_kernel_runtime`** | **0.2.0**（crate） | 与 HTTP/`--api` 契约 | 见 [COMPATIBILITY.md](../COMPATIBILITY.md) |
| **`oclive_validation`** | **0.1.0** | 角色包 / 蓝图校验 | 与编写器 wasm 对齐 |

## 为何不同步 bump

- 桌面发版含前端与安装器；CLI 可单独 `cargo install` 更新。
- 内核 crate 被多宿主（Tauri、`kernel_server`、未来启动器）引用，**契约变更**应独立于 UI 发版。
- [COMPATIBILITY.md](../COMPATIBILITY.md) 已用 **`min_runtime_version`** / `API_VERSION` 表达跨产物兼容，而非单一全局版本。

## 当前补丁发布

1. [CHANGELOG.md](../../CHANGELOG.md) **`[0.5.1] - 2026-08-22`** 已整理；发版日打 tag **`oclivenewnew-v0.5.1`**。
2. 桌面补丁 tag：**`oclivenewnew-v0.5.1`**；`oclive-cli` 未变更，不重复打 tag。
3. Breaking 角色包：必须链 [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md)。

## 后续

- 仅当 **`oclive_kernel_runtime` 主版本** 升级时，同步检查 `min_runtime_version` 与 OOCP 套件。
- Monolith / 嵌入式交付物版本跟随**生成该二进制的 CLI + 模板**版本记录于 `bench_history.json`（本地，不提交）。

[English](../COMPATIBILITY.md) · [DOCUMENTATION_INDEX](../getting-started/DOCUMENTATION_INDEX.md)
