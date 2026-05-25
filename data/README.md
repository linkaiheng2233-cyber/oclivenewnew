# 社区目录插件索引（GitHub 线 SSOT）

本目录 **`plugins.json`** 为桌面端与 CLI 的**权威清单草稿**；发布到线上索引仓库：

- 仓库：[awesome-oclive-plugins](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins)
- 线上 URL：`https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugins/main/plugins.json`

## 字段（与 `PluginIndexEntry` 对齐）

| 字段 | 必填 | 说明 |
|------|------|------|
| `id` | 是 | 与插件根目录 `manifest.json` 的 `id` 一致 |
| `name` | 是 | 展示名 |
| `version` | 是 | 与 manifest `version` 一致 |
| `git` | 是 | `git clone` 用的 HTTPS URL |
| `gitSubdir` | 否 | 单仓多插件时，仓库内相对路径（如 `examples/directory-plugin-minimal`） |
| `description` / `author` / `tags` | 否 | 市场列表展示 |
| `dependencies` | 否 | `id` → semver 范围 |
| `permissions` | 否 | 展示用，安装不自动授权 |

## 维护流程

1. 改本文件后本地校验：`node scripts/validate-plugins-index.mjs`
2. 同步到 awesome 仓库根目录 `plugins.json`：`node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-plugins/plugins.json`（路径按本机克隆调整）
3. 在 awesome 仓库提交并 push；桌面端默认 URL 即会拉到新列表。

开发/CI 可设 `OCLIVE_PLUGIN_INDEX_URL` 指向主仓镜像：

`https://raw.githubusercontent.com/linkaiheng2233-cyber/oclivenewnew/main/data/plugins.json`

角色包、`catalog.json`、Supabase 社区站不在此目录维护。
