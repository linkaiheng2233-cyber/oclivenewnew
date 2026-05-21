# 角色包创作者学习路径

按 **时间盒** 划分，便于第一次上手与进阶排期。权威格式仍以 **[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md)** 与 **[roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)** 为准；磁盘校验命令见 **`oclive-cli`**（仓库根执行 **`cargo run -p oclive-cli -- pack …`**）。

---

## 入门（约 30 分钟）

| 步骤 | 做什么 | 读什么 / 做什么 |
|------|--------|------------------|
| 1 | 理解角色包长什么样 | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) **§1 目录结构** |
| 2 | 生成第一个可校验的最小包 | `cargo run -p oclive-cli -- pack create -o <输出父目录> --id my_first_role --format-blueprint-v2`（写入 `pipeline.ocblueprint`；见 [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)） |
| 3 | 在编写器中打开 | **oclive-pack-editor** 编辑 v2 蓝图或 legacy 双文件（分工见 [CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)） |
| 4 | 改门面信息 | v2：编辑 `pipeline.ocblueprint` → `meta`（`name`、`description`、`personality`、`scenes` 等）；legacy 见 [README_MANIFEST](../../roles/README_MANIFEST.md) |

**验收**：`cargo run -p oclive-cli -- pack validate <角色根>` **默认 v2** 通过；维护中的旧包用 `--profile legacy`。

---

## 进阶（约 1–2 小时）

| 主题 | 读什么 |
|------|--------|
| **七维人格向量** | [README_MANIFEST § default_personality](../../roles/README_MANIFEST.md) · [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) |
| **系统提示词与开场白** | 角色包内 `prompts/` 自管素材与引擎组装关系见 ROLE_PACK_SPEC 与 [WORLDVIEW_KNOWLEDGE.md](WORLDVIEW_KNOWLEDGE.md)；主对话 Prompt 由 **`plugin_backends.prompt`** 与内置策略决定 |
| **`plugin_backends` 第 1–6 模块** | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |

**验收**：能在 `settings.json` 中为目标槽选择 `builtin` / `remote` / `directory` 之一，并理解 `directory_plugins` 与 manifest `id` 的对应关系。

| **团队协作** | `oclive collab init/status/pull/push` · [ROLE_PACK_SPEC.md §7](ROLE_PACK_SPEC.md#7-团队协作oclive-collab) |

---

## 高级（约半天）

| 主题 | 读什么 / 注意 |
|------|----------------|
| **`reply_quality_anchor` 与回复风格** | [README_MANIFEST](../../roles/README_MANIFEST.md) · ROLE_PACK_SPEC 中 `settings` 合并表；用于锚定回复质量/风格相关配置（以校验 crate 与宿主加载为准） |
| **`pipeline.ocblueprint` v2（推荐 SSOT）** | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · [handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md)。**桌面编排**以 **`process_message` → `co_present`** 为准，**不**再读蓝图 `steps[]`（见 [AGENTS.md](../../AGENTS.md)）。主应用架构图可 **`save_role_slot_registry`** 写回 `slot_registry` |
| **校验** | 默认 v2：`pack validate <角色根>`；旧包 `--profile legacy`；无头交付 `--profile robot-soul`（须 legacy 形状，见 ROLE_PACK_SPEC §6） |
| **编写器侧 wasm 校验** | [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) 的 `wasm:build` 与「运行全部检查」 |

**验收**：`pack validate` 无错误；理解「哪些键会进宿主合并校验、哪些仅作者自管」。

---

## 发布

| 步骤 | 命令 / 文档 |
|------|-------------|
| **打出 `.oclivepack`** | `cargo run -p oclive-cli -- pack publish <角色根目录> -o <输出路径>`（默认生成 `<id>-<version>.oclivepack`；见 CLI 指南） |
| **社区索引 JSON** | [ROLE_PACK_INDEX.md](ROLE_PACK_INDEX.md) · 市场/站点侧流程见 [../roadmap/MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| **与主程序版本对齐** | [COMPATIBILITY.md](../COMPATIBILITY.md) · `manifest.min_runtime_version` |

---

## 学完以后

- 维护包版本与 `schema_version`：[PACK_VERSIONING.md](PACK_VERSIONING.md)  
- 深度校验路线：[EDITOR_VALIDATION_ROADMAP.md](EDITOR_VALIDATION_ROADMAP.md)
