# AI 剧场发行版（从 0 开发）

本目录为 **AI 剧场（`distro_id=theater`）唯一保留区**。主仓其它路径的剧场代码、profile、角色包、脚本均已清理，从此处重新规划与落地。

| 文件 | 说明 |
|------|------|
| [`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md) | 思路与开发路线 SSOT |
| [`theater.oclive.toml`](theater.oclive.toml) | Profile 模板（未接入构建；开工时复制到 `examples/distro-profiles/` 与 `src-tauri/resources/distro-profiles/`） |

**策略**：复用 Chat Pro（`desktop`）内核与 `ToolShell`，在 profile / 编排 / 前端差分上二次开发。详见路线图。
