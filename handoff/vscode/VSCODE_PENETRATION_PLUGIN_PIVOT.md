# VS Code 渗透插件化 · 交接索引

**锁定日期**：2026-06-11  
**战略 SSOT**：姊妹仓 [`oclive-vscode/docs/PENETRATION_PLUGIN_MODEL.md`](../../oclive-vscode/docs/PENETRATION_PLUGIN_MODEL.md)

---

## 决策门（D1–D4）

| 门 | 选择 | 含义 |
|----|------|------|
| **D1** | **B** | 独立 npm **`@oclive/vscode-host`**（类型 + `resolveOcliveHost()` 运行时） |
| **D2** | **A** | 核心 Chat **动态插槽** `registerChatToolbarAction` |
| **D3** | **A** | 官方渗透扩展姊妹仓 **`oclive-vscode-penetration`** |
| **D4** | **仅新名** | 命令前缀 **`oclive-penetration.*`**；0.4 起 Breaking，**无** shim |

详见 [`oclive-vscode/docs/GATE_DECISIONS.md`](../../oclive-vscode/docs/GATE_DECISIONS.md)。

---

## 三仓关系

| 仓库 | 产物 | 依赖 |
|------|------|------|
| **`oclive-vscode-host`** | npm `@oclive/vscode-host@0.1.0` | `peerDependency`: `@types/vscode` |
| **`oclive-vscode`** | vsix 核心 **0.4.0** | `@oclive/vscode-host` |
| **`oclive-vscode-penetration`** | vsix 渗透 **0.1.0** | `@oclive/vscode-host` + `extensionDependencies: oclive.oclive-vscode` |

本地联调：`file:../oclive-vscode-host`；multi-root workspace 三仓并列。

**GitHub**（2026-06-11）：

| 仓库 | URL |
|------|-----|
| host | https://github.com/linkaiheng2233-cyber/oclive-vscode-host |
| 核心 | https://github.com/linkaiheng2233-cyber/oclive-vscode |
| 渗透 | https://github.com/linkaiheng2233-cyber/oclive-vscode-penetration |

**npm**：[`@oclive/vscode-host@0.1.0`](https://www.npmjs.com/package/@oclive/vscode-host)（npmjs.org）

- 核心：`oclive.oclive-vscode`
- 渗透：`oclive.oclive-vscode-penetration`

---

## 主仓边界（不变）

- `process_message` / 六槽：**不**为渗透改编排
- 角色包 `penetration_templates`：**仍由** `oclive_validation` 校验
- 插件读模板；核心不删 schema

---

## 文档索引

| 文档 | 位置 |
|------|------|
| 迁移 Breaking | `oclive-vscode/docs/MIGRATION_0.3_to_0.4.md` |
| 宿主 API | `oclive-vscode/docs/HOST_API_V1.md` |
| 插件作者 | `oclive-vscode/docs/PENETRATION_PLUGIN_AUTHOR.md` |
| F5 验收 | `oclive-vscode/docs/F5_ACCEPTANCE.md` |
| 跨仓契约 | `handoff/vscode/VSCODE_DISTRIBUTION.md` |

---

## 版本矩阵（0.4.x GA）

| 产物 | 版本 |
|------|------|
| `@oclive/vscode-host` | 0.2.0 |
| `oclive-vscode` | **0.4.1** |
| `oclive-vscode-penetration` | 0.1.1 |

**GA 状态**（2026-06-11）：F5/VSIX 自动化签核绿 · Release 可下载 · README 两步安装路径明确。见 [`oclive-vscode/docs/F5_ACCEPTANCE.md`](../../oclive-vscode/docs/F5_ACCEPTANCE.md)。
