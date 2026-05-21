# 已移除的 CLI 别名（2026-05）

以下顶层/子命令已从 `oclive-cli` 移除；请改用替代命令：

| 旧命令 | 替代 |
|--------|------|
| `oclive publish` | `oclive template pack` |
| `oclive plugin search` | `oclive market search` |
| `oclive plugin update` | `oclive market install <id>` |
| `oclive registry login` | `oclive config set OCLIVE_REGISTRY_URL …` 与 `oclive config set OCLIVE_REGISTRY_TOKEN …` |

`oclive pack publish`（角色包 `.oclivepack`）**保留**，与模板发布无关。
