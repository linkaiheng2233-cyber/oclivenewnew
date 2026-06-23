# AI 剧场发行版（从 0 开发）



本目录为 **AI 剧场（`distro_id=theater`）唯一保留区**。主仓其它路径的剧场代码、profile、角色包、脚本均已清理，从此处重新规划与落地。



| 文件 | 说明 |

|------|------|

| [`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md) | 思路与开发路线 SSOT |

| [`INFORMATION_ARCHITECTURE.md`](INFORMATION_ARCHITECTURE.md) | 信息架构（模式 1 · 舞台壳 / 区域 / 组件 / 状态机 / 开发顺序） |

| [`theater.oclive.toml`](theater.oclive.toml) | Profile 模板（已同步至 `examples/distro-profiles/` 与 `distros/desktop-tauri/resources/distro-profiles/`） |



**策略**：复用 Chat Pro（`desktop`）内核与 store/api，在 profile + `TheaterShell` 前端差分上二次开发。详见路线图。



## 本地启动（模式 1）



```bash

# 剧场壳 + theater profile（Tauri 开发）

npm run tauri:dev:theater



# 仅前端 Vite（无 Tauri IPC；骨架从 public/theater 加载）

# PowerShell:

$env:VITE_OCLIVE_SHELL='theater'; npm run dev

```



**验收**：`html[data-shell="theater"]`、暖橙 accent、头/舞台/戳 Dock/脚注四区可见；开场自动播放默认场景（早餐）`scenes/breakfast.skeleton.json`；左上角可切换四场景。

**手动回归矩阵（四场景）**：

| 场景 | 默认卡司+家人 | 换卡司 | 改关系 | 戳点 |
|------|---------------|--------|--------|------|
| breakfast | 预生成秒开 | AI 重写+forks | AI 重写 | 可用（4 芯片） |
| supermarket | 预生成秒开 | AI 重写 beats-only | AI 重写 | 可用（4 芯片） |
| way_home | 预生成秒开 | AI 重写 beats-only | AI 重写 | 可用（4 芯片） |
| bedtime | 预生成秒开 | AI 重写 beats-only | AI 重写 | 可用（4 芯片） |



## 工程验证



```bash

npm run test:theater:smoke

npm run tauri:build:theater   # 仅打包 mumu + 枫侵月 至 resources/roles

```



`test:e2e:distro-kernel` 的 `theater` scenario 断言 `GET /health` → `distro_id=theater`。

