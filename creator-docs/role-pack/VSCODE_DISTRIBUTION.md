# VS Code 发行版（扩展契约）

**实现仓库**：`oclive-vscode`（与主应用同级目录）。  
**跨宿主记忆**：[`CROSS_HOST_MEMORY.md`](CROSS_HOST_MEMORY.md)。

---

## 定位

| 项 | 约定 |
|----|------|
| 对标内核模式 | **`pure_chat`**（建议蓝图 `runtime_config.interaction_mode` 或包级默认） |
| 场景 | `scene_id=vscode`（须在包 `meta.scenes` 中声明） |
| 记忆 / 好感 | 与桌面 **共用 `app.db`**（单内核写库） |
| **数据目录** | spawn 时设 `OCLIVE_APP_DATA` → `%LOCALAPPDATA%/OCLive/data`（与 `OCLive/runtime` 并列） |
| 渗透能力 | **扩展设置默认关闭**（信、心声、idle 聚焦、终端展示行） |

桌面 **沉浸模式**（`immersive`）的虚拟时间、位移、异地心声 UI **不要求** VS Code 侧实现。

---

## 扩展本分（Phase 1.x）

- 侧栏：**顶栏立绘**（`assets/images/` + `portrait_emotion` / `bot_emotion`，无图则 emoji）
- **对话** + 可选 **编辑器上下文**（当前文件 / 选区）
- 内核：`GET /health` + `POST /chat`（`8420`，attach 优先）

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
