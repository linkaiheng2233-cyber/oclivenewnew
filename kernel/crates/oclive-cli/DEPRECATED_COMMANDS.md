# 已移除的 CLI 别名（2026-05）

以下顶层/子命令已从 `oclive-cli` 移除；请改用替代命令：

| 旧命令 | 替代 |
|--------|------|
| `oclive publish` | `oclive template pack` |
| `oclive plugin search`（在线索引） | `oclive market search`；**已安装**扫描仍用 `oclive plugin search [--provides]` |
| `oclive plugin update` | `oclive market install <id>` |
| `oclive registry login` | `oclive config set OCLIVE_REGISTRY_URL …` 与 `oclive config set OCLIVE_REGISTRY_TOKEN …` |

`oclive pack publish`（角色包 `.oclivepack`）**保留**，与模板发布无关。

## Shell 补全（2026-05-20 复核）

`oclive completions bash` 由 **`clap_complete::generate` + `Cli::command()`** 生成，覆盖当前全部顶层子命令；上表已移除命令**不会**出现在补全脚本中。
