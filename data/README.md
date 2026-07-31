# OCLive 版本化数据目录

本目录只存放需要由运行时、开发工具或 CI 共同读取的**版本化数据**，不存模型、运行日志或用户状态。

| 路径 | 用途 | 权威边界 |
|------|------|----------|
| `plugins.json` | 社区目录插件索引草稿 | 插件市场清单；线上发布流程见下文 |
| `ci/impact-map.v1.json` | changed path → 直接模块、中央强制影响边与高风险规则 | OCLive 维护者拥有；第三方不能缩小范围 |
| `ci/validation-catalog.v1.json` | policy、profile、validator、workflow job 与受信命令坐标 | 只登记命令，不由模块自定义执行 |
| `ci/modules/*.oclive.module.json` | Stage 1 仓内领域描述 | 脚手架当前不生成；稳定后只作为生成/预检目标 |

领域感知 CI 的契约、第三方隔离和影子阶段见 [`SOMEDAY_TOOLCHAIN_CI.md`](../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)。`data/ci` 的改动属于中央控制面高风险变更，规划器会 fail-safe 到当前 policy 全量；Stage 1 仍不会跳过任何现有 job。

## 社区目录插件索引（GitHub 线 SSOT）

**`plugins.json`** 为桌面端与 CLI 的**权威清单草稿**；发布到线上索引仓库：

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

角色包、`catalog.json`、Supabase 社区站不在 `data/` 维护。
