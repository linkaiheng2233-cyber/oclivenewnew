# VS Code 发行版（扩展契约）

**实现仓库**：`oclive-vscode`（与主应用同级目录）。  
**产品战略 SSOT**：[`oclive-vscode/docs/STRATEGY.md`](../../../oclive-vscode/docs/STRATEGY.md)（以角色为基点 · 非 Cursor · 渗透 vs Agent 正交）。  
**跨宿主记忆**：[`CROSS_HOST_MEMORY.md`](CROSS_HOST_MEMORY.md)。

---

## 定位

| 项 | 约定 |
|----|------|
| **北极星** | **角色住在开发者的工程里** — 顺滑聊天 + IDE 渗透 + 可深度定制（**不是** Cursor / Cline 式默认编程 Agent） |
| 聊天底座 | 体验对齐基础聊天发行版的流畅易用；内核默认 **`pure_chat`** |
| 场景 | `scene_id=vscode`（须在包 `meta.scenes` 中声明） |
| 记忆 / 好感 | 与桌面 **共用 `app.db`**（单内核写库） |
| **数据目录** | spawn 时设 `OCLIVE_APP_DATA` → `%LOCALAPPDATA%/OCLive/data`（与 `OCLive/runtime` 并列） |
| 默认 profile | `examples/distro-profiles/vscode.oclive.toml`（`pure_chat`、`allow_mode_switch=false`、`skip_agent`） |
| 互动模式 | VS Code **永久 pure_chat**；不承载 `immersive`（决策门 B=A，见姊妹仓 [`GATE_DECISIONS.md`](../../../oclive-vscode/docs/GATE_DECISIONS.md)） |
| 渗透能力 | 0.3.1：`oclive.penetration.*`、`.oclive/{roleId}/` — 见姊妹仓 [`ROADMAP.md`](../../../oclive-vscode/ROADMAP.md) · [`VSCODE_DISTRIBUTION.md`](../../../oclive-vscode/docs/VSCODE_DISTRIBUTION.md) |

桌面 **沉浸模式** UI **不在** VS Code 实现；跨宿主记忆仍经 `OCLIVE_APP_DATA`。

---

## 扩展本分（Phase 1.x）

- 侧栏：**顶栏立绘**（`assets/images/` + `portrait_emotion` / `bot_emotion`，无图则 emoji）
- **对话** + 可选 **编辑器上下文**（当前文件 / 选区）
- 内核：`GET /health` + `POST /chat`（`8420`；**policy-first**：共享 Rust 策略 `resolve_kernel_action` 决定 attach / spawn / replace，见 [`DISTRO_KERNEL_LIFECYCLE.md`](../kernel/DISTRO_KERNEL_LIFECYCLE.md)）
- 可选：`POST /chat/stream`（需发行版能力 `oclive.chat.streaming`）

---

## 角色包要求（vscode-lite，编写器分级导出目标）

| 路径 | vscode-lite |
|------|-------------|
| `pipeline.ocblueprint` | 必填 |
| `scenes/vscode/` | 推荐 |
| `prompts/` | 推荐 |
| `assets/images/` | 可选（7 标签 PNG 可换皮） |
| `life_schedule` 大段 | 可裁剪 |
| 全量 `knowledge/` | 按需 |

---

## 编写器 backlog（未实现）

见 [`../../handoff/PACK_EDITOR_ROADMAP.md`](../../handoff/PACK_EDITOR_ROADMAP.md)：

- **情绪图片可视化编辑**（当前：简单创作页多文件上传）
- **情绪族扩展**（多族 × 多标签，需内核 `Emotion` / 立绘 `ALLOWED` 同步）
- **分级导出** `full` | `vscode-lite` | `headless`

---

## 测试

- CLI：`oclive-vscode` 下 `npm run smoke`
- 协议：主仓 `examples/oocp-test-suite/run.mjs`（Codex 轨道 A）
- 跨宿主数据：主仓 `node scripts/e2e-cross-host-memory.mjs`（canonical `OCLIVE_APP_DATA` smoke）
- **离线 / bundled kernel**：扩展仓库 [`oclive-vscode/bin/README.md`](../../../oclive-vscode/bin/README.md)（与扩展 `README.md` §Offline 一致）
