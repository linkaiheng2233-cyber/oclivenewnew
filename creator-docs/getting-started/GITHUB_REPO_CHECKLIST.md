# GitHub 仓库：已落地的自动化与需在网页上配置的事项

本文说明三件套仓库（`oclivenewnew`、`oclive-pack-editor`、`oclive-launcher`）在 **GitHub 上** 的建议配置；**代码侧** 已包含 CI、Dependabot、PR 模板等，合并到默认分支后即生效。

---

## 已在仓库内配置（推送后生效）

| 项目 | 说明 |
|------|------|
| **CI** | `.github/workflows/ci.yml`：push/PR 时跑测试与构建（详见各仓 workflow）。 |
| **手动重跑** | 支持 **`workflow_dispatch`**：在 Actions 页选 workflow → **Run workflow**。 |
| **Dependabot** | `.github/dependabot.yml`（各仓若有）：每周对 **npm / cargo** 开更新 PR，合并前请确认 CI。 |
| **PR 模板** | `.github/pull_request_template.md`（主仓库）：创建 PR 时自动带出检查清单。 |
| **README 徽章** | 各仓 README 顶部的 **CI 状态徽章**（指向本仓 Actions）。 |

---

## 需在 GitHub **网页上** 完成的项（无法仅靠提交代码）

以下登录 [github.com](https://github.com) → 进入 **对应仓库** → **Settings**：

| 优先级 | 设置项 | 建议 |
|--------|--------|------|
| 建议 | **General → Features** | 按需开启 **Issues**（反馈 bug）、**Discussions**（可选，讨论区）。 |
| 建议 | **Rules → Rulesets** 或 **Branches（经典分支保护）** | 对 `main`：**要求 PR 合并**、**要求通过状态检查（CI）**后再合并（团队多人时尤其重要；单人可暂不强制）。 |
| 可选 | **Actions → General → Fork pull request workflows** | 若接受 fork PR，按需限制；默认即可。 |
| 可选 | **Secrets and variables** | 将来若 CI 要发版、签名、调用 API，在此加 **Repository secrets**（勿把密钥写进仓库）。 |
| 可选 | **Pages** | 仅当要用 GitHub Pages 托管静态站时配置。 |

**Dependabot**：仓库根已有 `dependabot.yml` 并推送后，GitHub 会自动启用版本更新 PR；若 **Insights → Dependency graph → Dependabot** 显示未启用，检查文件是否在默认分支且路径为 `.github/dependabot.yml`。

---

## 三仓库链接（便于设置）

- [oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew) — 运行时  
- [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) — 编写器  
- [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher) — 启动器  

若仓库名或所有者变更，请同步更新各 README 里的 CI 徽章 URL。

---

## 合并 Dependabot PR 时

1. 看 CI 是否全绿。  
2. 大版本升级（如 Vite、Tauri major）建议在本地再跑 `npm run check` / `check:release`。  
3. 若有冲突，在本地合并依赖分支并解决后再推。

---

*与 [CONTRIBUTING.md](../../CONTRIBUTING.md)、[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) 互补。*
