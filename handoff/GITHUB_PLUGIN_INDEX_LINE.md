# GitHub 目录插件索引线（阶段 A）

**范围**：`plugins.json` → 桌面同步/安装、CLI `oclive market`。**不含**角色包 `catalog.json`、Supabase 社区站。

## SSOT 与线上 URL

| 用途 | 路径 / URL |
|------|------------|
| 主仓草稿（PR 改这里） | `data/plugins.json` |
| 线上默认（桌面 + CLI） | `https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugins/main/plugins.json` |
| 开发镜像（含 `gitSubdir` 官方示例） | `https://raw.githubusercontent.com/linkaiheng2233-cyber/oclivenewnew/main/data/plugins.json` |

## 维护命令

```bash
node scripts/validate-plugins-index.mjs
node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-plugins/plugins.json
# 在 awesome-oclive-plugins 仓库 commit + push
```

## 安装语义

- 索引项 **`git`**：浅克隆 URL。
- **`gitSubdir`**（可选）：单仓多插件时的相对路径；宿主与 CLI 均支持（2026-05-20 起）。

## 验收（本地）

1. `OCLIVE_PLUGIN_INDEX_URL=https://raw.githubusercontent.com/.../oclivenewnew/main/data/plugins.json`
2. 桌面：插件工作台 → 社区索引 → **同步在线索引** → 列表 ≥4 条 → 安装 `com.oclive.example.minimal`
3. CLI：`cargo run -p oclive-cli -- market search minimal` → `market install com.oclive.example.minimal`

## 代码锚点

- `src-tauri/src/infrastructure/plugin_installer.rs` — `DEFAULT_PLUGIN_INDEX_URL`、`PluginIndexEntry.git_subdir`
- `src-tauri/src/api/plugin_index.rs` — `sync_plugin_index_command` / `install_plugin_from_market`
- `crates/oclive-cli/src/market_index.rs` — 默认 URL、空 awesome 列表时回退主仓 `data/plugins.json`
