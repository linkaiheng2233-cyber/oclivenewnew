# 迁入内核收尾说明（Kernel migration — complete）

> **状态**：共享机制在 `crates/oclive_kernel_runtime`；桌面仅 invoke、路径、`AppState`、`rescan` / 进程生命周期等。  
> **对照表**：[creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md](../creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md)  
> **轻量 profile（可选特性、OOCP/`invoke`、依赖与 `http_api` 拟定）**：[creator-docs/kernel/LIGHTWEIGHT_PROFILE.md](../creator-docs/kernel/LIGHTWEIGHT_PROFILE.md)；本地自检亦可运行根目录 `scripts/check_kernel_runtime_minimal.sh` 或 `scripts/check_kernel_runtime_minimal.ps1`。  
> **说明**：角色市场索引缓存写入为 **`fs::write` 且错误上抛**（非静默忽略）。

## 内核模块一览

| 领域 | 内核位置 | 桌面残留 |
|------|-----------|----------|
| 角色包 ZIP、导入、市场直链 | `infrastructure::role_pack_archive` | `role_pack.rs` / `role_market.rs` 薄封装 |
| `roles.json` | `models::role_market_index`、`role_market_index_sync` | — |
| `reviews.json` | `models::plugin_reviews_index`、`plugin_reviews_index_sync` | `plugin_reviews.rs` 薄封装 |
| 插件索引 URL | `plugin_index_sync::resolve_plugin_index_url` | — |
| 插件安装 | `infrastructure::plugin_install` | `plugin_installer.rs`：`rescan` 等 |
| `.oclive_install.json` 写 | `directory_plugins::write_plugin_install_meta` | — |
| 卸载清 `plugin_state` | `PluginStateStore::remove_plugin_references` | — |

## 自检（仓库根）

```bash
cargo fmt --all --check
cargo test -p oclive_kernel_runtime role_pack_archive
cargo check -p oclivenewnew-tauri
cargo check -p oclive_kernel_runtime --no-default-features
```

## 保留在发行版

`deep_link`、`hotkey` 全局注册、`directory_plugins/watcher`、`directory_plugin_invoke` 进程侧等。
