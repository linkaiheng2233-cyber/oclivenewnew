# VS Code 发行版（扩展契约）

**实现仓库**：`oclive-vscode`（与主应用同级目录）。  
**产品战略 SSOT**：[`oclive-vscode/docs/STRATEGY.md`](../../../oclive-vscode/docs/STRATEGY.md)（以角色为基点 · **渗透插件化**）。  
**渗透模型**：[`PENETRATION_PLUGIN_MODEL.md`](../../../oclive-vscode/docs/PENETRATION_PLUGIN_MODEL.md)  
**跨宿主记忆**：[`CROSS_HOST_MEMORY.md`](../../creator-docs/role-pack/CROSS_HOST_MEMORY.md)。

---

## 定位

| 项 | 约定 |
|----|------|
| **北极星** | **轻量化 Chat Pro 住进开发者工程** — 角色陪伴为主体，代码/编译上下文做差异化（不是 Cursor） |
| **核心扩展** | 聊天 · 内核 attach · 角色 · 身份 · 记忆 · 编辑器上下文 · 诊断/选区只读分析 · 宿主 API（0.4+） |
| **渗透** | **可选插件**（日记+信+idle 等合并一包）；0.3.x 内置为 **过渡** |
| **效率边界** | 0.5 默认只读分析；运行编译/测试或写盘必须使用独立权限与显式用户动作 |
| 场景 | `scene_id=vscode` |
| 记忆 / 好感 | 与桌面 **共用 `app.db`** |
| 默认 profile | `examples/distro-profiles/vscode.oclive.toml`（`pure_chat`、`skip_agent`；`[plugin_backends]` 整表替换） |
| 互动模式 | VS Code **永久 pure_chat**（决策门 B） |

桌面 **沉浸模式** UI **不在** VS Code 实现。

---

## 扩展本分

### 核心（长期）

- 侧栏：**立绘** + **对话** + 编辑器上下文 + 当前诊断计数
- 代码特化：显式“分析当前诊断”“审查当前选区”；不隐式运行命令或修改文件
- 内核：`GET /health` + `POST /chat` / `stream`（8420；**profile-aware attach + bundled-first spawn** — 见 [`DISTRO_KERNEL_LIFECYCLE.md`](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md)）
- **不** 长期内置：日记、写信、idle、渗透设置大分区

### 渗透插件（0.4+ · 三仓 + npm）

| 仓库 / 产物 | 说明 |
|-------------|------|
| **`oclive-vscode-host`** | `@oclive/vscode-host@0.2.0` — 宿主 API 契约 |
| **`oclive-vscode`** | 核心 VSIX **0.5.0 Preview** — 轻量 Chat Pro + 只读代码上下文特化 |
| **`oclive-vscode-penetration`** | 官方渗透 vsix **0.1.0** — `oclive-penetration.*` |

- 消费：`penetration_templates`、`.oclive/` 路径约定、`bridge/dispatch` C2
- 主仓 `oclive_validation` 对模板字段的校验 **保留**
- 交接索引：[`VSCODE_PENETRATION_PLUGIN_PIVOT.md`](./VSCODE_PENETRATION_PLUGIN_PIVOT.md)

---

## 角色包要求（vscode-lite）

| 路径 | vscode-lite |
|------|-------------|
| `pipeline.ocblueprint` | 必填 |
| `scenes/vscode/` | 推荐 |
| `config.json` → `penetration_templates` | 可选 · **供渗透插件读** |

---

## 相关文档

| 文档 | 路径 |
|------|------|
| 内核 lifecycle | [`DISTRO_KERNEL_LIFECYCLE.md`](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) · [`KERNEL_SCHEDULER_RESCOPE.md`](../KERNEL_SCHEDULER_RESCOPE.md) |
| 路线图 | [`oclive-vscode/ROADMAP.md`](../../../oclive-vscode/ROADMAP.md) |
| 扩展契约详表 | [`oclive-vscode/docs/VSCODE_DISTRIBUTION.md`](../../../oclive-vscode/docs/VSCODE_DISTRIBUTION.md) |
| 决策门 | [`GATE_DECISIONS.md`](../../../oclive-vscode/docs/GATE_DECISIONS.md) |
