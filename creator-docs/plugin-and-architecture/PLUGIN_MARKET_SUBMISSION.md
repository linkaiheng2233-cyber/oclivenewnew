# 插件市场投稿与分发（链接策展模型）

**适用**：目录插件（`manifest.json` + `type: ocliveplugin`）。**不含**角色包市场、Supabase 社区站上传。

## 设计原则

| 角色 | 做什么 | 不做什么 |
|------|--------|----------|
| **插件作者** | 在 **自己的 Git 仓库** 维护插件源码与文档；向索引提交 **一条链接记录**（`git` + 可选 `gitSubdir`） | 不向 oclive 官方上传 zip、不占用官方存储 |
| **索引维护者**（项目方） | 审核 PR，维护 `plugins.json`（主仓草稿 → [awesome-oclive-plugins](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins)） | 不为每个插件做长期托管；仅策展元数据 |
| **终端用户** | 在桌面 **插件市场** 粘贴信任的 **分享链接**（`plugins.json` 或单个仓库 URL）后浏览/安装 | 默认不自动拉取未信任的公共目录 |

这样 **网络流量小**（索引 JSON 很小；安装时仅 `git clone` 作者仓库），**垃圾插件**难以进入「用户未主动粘贴」的目录；进入官方列表需 **GitHub PR + 人工审核**。

## 用户侧：分享链接

桌面 **插件与后端管理 → 插件市场**：

1. 粘贴创作者提供的链接。
2. 点击 **加载**。
   - **`…/plugins.json`**：加载目录，浏览多条插件。
   - **Git 仓库 HTTPS / SSH**：识别为单插件仓库，显示安装卡片（浅克隆到本机 `{app_data}/distros/chat-pro/plugins/<id>/`）。
3. 安装后于 **插件工作台** 配置槽位；高风险权限按 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) 弹窗授权。

维护者可将审核后的目录 raw 链接发给用户，例如：

`https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-distros/chat-pro/plugins/main/plugins.json`

## 作者侧：投稿流程（GitHub）

1. **准备插件仓库**（独立仓或 monorepo 子路径），满足 [PLUGIN_V1.md](PLUGIN_V1.md) 与 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)。
2. **在插件目录内写清文档**（见下一节 checklist；维护者 PR 可拒收文档不全的条目）。
3. 向 **oclivenewnew** [`data/plugins.json`](../../data/plugins.json) 提 PR，增加一条 `plugins` 元素（字段见 [GITHUB_PLUGIN_INDEX_LINE.md](../../handoff/GITHUB_PLUGIN_INDEX_LINE.md)）。
4. 合并后维护者同步 awesome 仓库的 `plugins.json`。
5. 把 **目录 raw 链接** 或 **你的仓库链接** 写在 README / 发布说明里，供用户粘贴到插件市场。

本地自检：

```bash
node scripts/validate-plugins-index.mjs
```

## 插件包内必须写清的内容（作者责任）

索引里的 `description` 只有一两句；**真实说明必须在插件仓库内**，方便用户与审核者 `git clone` 后直接阅读。

### 1. `README.md`（必填）

建议包含以下小节（可用中文或英文，但须完整）：

| 小节 | 内容 |
|------|------|
| **功能** | 插件解决什么问题、提供哪些 `provides` / 插槽 |
| **环境要求** | Node/Python 版本、系统依赖、是否需要 GPU/Ollama 等 |
| **安装** | 手动复制到 `distros/chat-pro/plugins/` 的路径说明；或说明「仅通过插件市场 / `git` 安装」 |
| **配置** | `plugin_state`、环境变量、与 `plugin_backends` 的对应关系 |
| **权限说明** | `manifest.json` 里每一项 `permissions` / `shell.bridge.invoke` **为何需要** |
| **兼容版本** | 测试过的 oclive / 主应用版本（如 `0.2.x`） |
| **支持/反馈** | Issue 链接、邮箱或讨论区；**勿**只写「联系作者」无 URL |

### 2. `manifest.json`（必填且诚实）

- `id`：稳定反向域名，与索引条 `id` **完全一致**。
- `version`：semver，与索引条 `version` **一致**。
- `permissions` / `process` / `shell.bridge`：**最小权限**；新增权限须在 README 解释。
- 可选：`description` 字段（一句话）、`author`（见下）。

### 3. 可选但推荐

| 文件 | 用途 |
|------|------|
| `CHANGELOG.md` | 版本变更 |
| `LICENSE` | 许可证（作者自选；主仓为 Apache-2.0，插件可用 MIT / Apache-2.0 等） |
| `author.json` | 作者展示名、推荐后端（见 [AUTHOR_JSON.md](../role-pack/AUTHOR_JSON.md) 若复用形状） |

### 4. 索引条（`plugins.json`）与仓库一致

| 字段 | 要求 |
|------|------|
| `git` | 可 `git clone --depth 1` 的 URL |
| `gitSubdir` | monorepo 时指向含 `manifest.json` 的子目录 |
| `description` | 列表摘要，须与 README 首段不矛盾 |
| `permissions` | 展示用列表，应与 manifest 一致 |
| `dependencies` | 若依赖其它插件 id，写明 semver 范围 |

## 维护者审核清单（防垃圾）

PR 合并前建议核对：

- [ ] 仓库可访问，且为 **插件源码**（非空壳、非纯广告页）。
- [ ] `manifest.json` 与索引 `id` / `version` / `git`(+`gitSubdir`) 一致。
- [ ] README 含上表必填小节；权限有解释。
- [ ] `permissions` 无无故的 `network:*`、`process:spawn` 等高风险项。
- [ ] `validate-plugins-index.mjs` 通过。
- [ ] 非重复 `id`；`description` / `tags` 无误导。

拒收示例：无 README、权限过宽无说明、索引指向非插件仓库、仿冒官方 `com.oclive.*` id。

## 与其它文档的关系

| 文档 | 内容 |
|------|------|
| [PLUGIN_V1.md](PLUGIN_V1.md) | 契约与 RPC |
| [PLUGIN_AUTHOR_LEARNING_PATH.md](PLUGIN_AUTHOR_LEARNING_PATH.md) | 开发入门 |
| [GITHUB_PLUGIN_INDEX_LINE.md](../../handoff/GITHUB_PLUGIN_INDEX_LINE.md) | 索引字段、环境变量、安装语义 |
| [ERROR_CODES.md](../getting-started/ERROR_CODES.md) | 市场离线/缓存排障 |

---

[English](../../creator-docs-en/plugin-and-architecture/PLUGIN_MARKET_SUBMISSION.md)（待镜像时可从本页翻译）
