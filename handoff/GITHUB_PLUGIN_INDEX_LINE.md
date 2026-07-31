# GitHub 目录插件索引线（阶段 A）

**范围**：`plugins.json` → 桌面插件市场（粘贴分享链接加载）/ 安装、CLI `oclive market`。**不含**角色包 `catalog.json`、Supabase 社区站、向官方上传 zip。

**分发模型**：索引仅保存 **元数据 + `git` 链接**；源码与下载由 **作者仓库** 承担；官方 PR 策展防垃圾。见 **[PLUGIN_MARKET_SUBMISSION.md](../creator-docs/plugin-and-architecture/PLUGIN_MARKET_SUBMISSION.md)**。

## SSOT 与线上 URL

| 用途 | 路径 / URL |
|------|------------|
| 主仓草稿（PR 改这里） | [`data/plugins.json`](../data/plugins.json) |
| 线上默认（桌面 + CLI） | `https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-distros/chat-pro/plugins/main/plugins.json` |
| 开发镜像 | `https://raw.githubusercontent.com/linkaiheng2233-cyber/oclivenewnew/main/data/plugins.json` |

## 创作者：如何通过 PR 加入索引

1. **插件仓库**：每个目录插件须为可 `git clone` 的独立仓库，或 monorepo 子路径（见 `gitSubdir`）。**README 与 manifest 须写清功能、环境、权限原因**（维护者可拒收文档不全条目），见 [PLUGIN_MARKET_SUBMISSION.md](../creator-docs/plugin-and-architecture/PLUGIN_MARKET_SUBMISSION.md)。
2. **自检**：`manifest.json` 的 `id` / `version` 与索引条一致；`node scripts/validate-plugins-index.mjs` 通过（对 monorepo 示例会核对子路径 manifest）。
3. **改主仓草稿**：在 **oclivenewnew** 向 `data/plugins.json` 提 PR，增加一条 `plugins` 数组元素。
4. **同步 awesome**：合并后维护者运行  
   `node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-distros/chat-pro/plugins/plugins.json`  
   并在 [awesome-oclive-plugins](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins) 提交 `plugins.json`。
5. **勿重复字段**：每条仅保留 camelCase **`gitSubdir`**，不要同时写 `git_subdir`。
6. **分享给用户**：提供审核后目录的 **raw `plugins.json` 链接**，或单插件 **仓库 URL**；用户在桌面插件市场 **粘贴 → 加载**。

PR 说明建议附上：插件 id、测试过的 oclive 版本、所需 `permissions`、是否依赖其它插件 id、README 是否已含安装与权限说明。

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

桌面端在插件市场 **粘贴目录链接并加载成功** 时也会写入上述 app_data 缓存。

## 安装语义

- **`git`**：浅克隆到临时目录，再按 `manifest.id` 移到 `{app_data}/distros/chat-pro/plugins/<id>/`。
- **`gitSubdir`**：克隆后进入子目录再校验 manifest 并移动（2026-05-20 起，桌面 + CLI 一致）。

## 维护命令

```bash
node scripts/validate-plugins-index.mjs
node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-distros/chat-pro/plugins/plugins.json
```

## 验收（本地）

1. 可选：`$env:OCLIVE_LOCAL_MONOREPO = "D:\oclivenewnew"`（GitHub 不可达时）
2. 桌面：插件市场 → 粘贴主仓 `data/plugins.json` 的 raw 链接（或 awesome 默认目录）→ **加载** → 安装 `com.oclive.example.minimal`
3. CLI：`cargo run -p oclive-cli -- --experimental market search minimal` → `--experimental market install com.oclive.example.minimal`
4. 已安装扫描：`cargo run -p oclive-cli -- plugin search --provides llm -o <plugins-dir>`

## 代码锚点

- `kernel/crates/oclive_kernel_host/src/infrastructure/plugin_installer.rs` — `DEFAULT_PLUGIN_INDEX_URL`、`git_subdir`、本地回退
- `distros/desktop-tauri/src/api/plugin_index.rs` — `sync_plugin_index_command` / `install_plugin_from_market`
- `kernel/crates/oclive-cli/src/market_index.rs` — 默认 URL、空 awesome 回退主仓
- `kernel/crates/oclive-cli/src/plugin_search.rs` — `oclive plugin search --provides`

## 后续（P1+）

- 桌面 SimplePluginManager「浏览市场」入口深化（见路线图）
- awesome 仓库 PR 模板、第一批社区插件上架
- `oclive plugin update`（索引版本比对 + pull）
