# GitHub 目录插件索引线（阶段 A）

**范围**：`plugins.json` → 桌面同步/安装、CLI `oclive market`。**不含**角色包 `catalog.json`、Supabase 社区站。

## SSOT 与线上 URL

| 用途 | 路径 / URL |
|------|------------|
| 主仓草稿（PR 改这里） | [`data/plugins.json`](../data/plugins.json) |
| 线上默认（桌面 + CLI） | `https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugins/main/plugins.json` |
| 开发镜像 | `https://raw.githubusercontent.com/linkaiheng2233-cyber/oclivenewnew/main/data/plugins.json` |

## 创作者：如何通过 PR 加入索引

1. **插件仓库**：每个目录插件须为可 `git clone` 的独立仓库，或 monorepo 子路径（见 `gitSubdir`）。
2. **自检**：`manifest.json` 的 `id` / `version` 与索引条一致；`node scripts/validate-plugins-index.mjs` 通过（对 monorepo 示例会核对子路径 manifest）。
3. **改主仓草稿**：在 **oclivenewnew** 向 `data/plugins.json` 提 PR，增加一条 `plugins` 数组元素。
4. **同步 awesome**：合并后维护者运行  
   `node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-plugins/plugins.json`  
   并在 [awesome-oclive-plugins](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins) 提交 `plugins.json`。
5. **勿重复字段**：每条仅保留 camelCase **`gitSubdir`**，不要同时写 `git_subdir`。

PR 说明建议附上：插件 id、测试过的 oclive 版本、所需 `permissions`、是否依赖其它插件 id。

## `plugins.json` 字段说明

| 字段 | 必填 | 说明 |
|------|------|------|
| `id` | 是 | 与插件根目录 `manifest.json` 的 `id` 一致 |
| `name` | 是 | 市场列表展示名 |
| `version` | 是 | 与 manifest `version` 一致（semver） |
| `git` | 是 | `git clone --depth 1` 用的 HTTPS（或 SSH）URL |
| `gitSubdir` | 否 | 单仓多插件时的相对路径，如 `examples/directory-plugin-minimal` |
| `description` / `author` / `tags` | 否 | 列表与搜索 |
| `category` / `source` | 否 | 分类与来源标签（如 `official`） |
| `permissions` | 否 | 展示用；安装不自动授权，见 DIRECTORY_PLUGINS |
| `dependencies` | 否 | 对象：`依赖插件 id` → semver 范围 |

根级可选：`generatedAt`（ISO 时间）。awesome 仓库另用 `version` + `generated_at` 包装，由同步脚本生成。

## 环境变量

| 变量 | 作用 |
|------|------|
| `OCLIVE_PLUGIN_INDEX_URL` | 覆盖在线 `plugins.json` URL（镜像、主仓 raw、本地调试） |
| `OCLIVE_MARKET_INDEX_URL` | CLI `oclive market` 同义覆盖（与上一项二选一即可） |
| `OCLIVE_LOCAL_MONOREPO` | **本地回退**：HTTPS 连不上 GitHub 时，对索引中指向 `oclivenewnew` 的 `git` 改用 `file:///<本机主仓路径>` 浅克隆（需已 clone 主仓） |

### 本机缓存路径

| 产物 | 路径 |
|------|------|
| 桌面（Tauri） | `{app_data}/plugin_index_cache.json`（与 `app.db` 同级） |
| CLI | `%USERPROFILE%\.oclive\plugin_index_cache.json`（Unix：`~/.oclive/`） |

刷新缓存（离线开发）：

```powershell
Copy-Item D:\oclivenewnew\data\plugins.json $env:USERPROFILE\.oclive\plugin_index_cache.json -Force
```

桌面端在「插件市场 → GitHub 插件索引 → 同步在线索引」成功时也会写入上述 app_data 缓存。

## 安装语义

- **`git`**：浅克隆到临时目录，再按 `manifest.id` 移到 `{app_data}/plugins/<id>/`。
- **`gitSubdir`**：克隆后进入子目录再校验 manifest 并移动（2026-05-20 起，桌面 + CLI 一致）。

## 维护命令

```bash
node scripts/validate-plugins-index.mjs
node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-plugins/plugins.json
```

## 验收（本地）

1. 可选：`$env:OCLIVE_LOCAL_MONOREPO = "D:\oclivenewnew"`（GitHub 不可达时）
2. 桌面：插件工作台 → 插件市场 → **同步在线索引** → 安装 `com.oclive.example.minimal`
3. CLI：`cargo run -p oclive-cli -- market search minimal` → `market install com.oclive.example.minimal`
4. 已安装扫描：`cargo run -p oclive-cli -- plugin search --provides llm -o <plugins-dir>`

## 代码锚点

- `src-tauri/src/infrastructure/plugin_installer.rs` — `DEFAULT_PLUGIN_INDEX_URL`、`git_subdir`、本地回退
- `src-tauri/src/api/plugin_index.rs` — `sync_plugin_index_command` / `install_plugin_from_market`
- `crates/oclive-cli/src/market_index.rs` — 默认 URL、空 awesome 回退主仓
- `crates/oclive-cli/src/plugin_search.rs` — `oclive plugin search --provides`

## 后续（P1+）

- 桌面 SimplePluginManager「浏览市场」入口深化（见路线图）
- awesome 仓库 PR 模板、第一批社区插件上架
- `oclive plugin update`（索引版本比对 + pull）
