# 从 v1 迁移到 v2 角色包

**目标读者**：仍使用 `manifest.json` + `settings.json` 的创作者。按本文操作，**约 10 分钟**可完成迁移与校验。

**权威格式**：[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · 校验实现：`crates/oclive_validation`。

[English](../../creator-docs-en/role-pack/V1_TO_V2_MIGRATION.md)

---

## 1. v2 蓝图架构（一句话）

**`pipeline.ocblueprint` 是角色包的唯一配置中枢（SSOT）**：`schema_version: 2`、`meta`（原 manifest + 引擎 settings 字段）、`slot_registry`（开放多实例槽位）。**不得**与 legacy 双文件并存；**禁止**在蓝图文件中写 `steps[]`、`entry`、`module_relations`（运行时由 `slot_registry` 派生）。

桌面主应用对话编排仍以 **`process_message` → `co_present`** 为准，**不**再按旧蓝图 `steps[]` DSL 调度首轮路径（见根目录 `AGENTS.md`）。

---

## 2. 字段映射表

### 2.1 `manifest.json` → `meta`

| legacy `manifest.json` | v2 `meta` | 说明 |
|------------------------|-----------|------|
| `id` | `id` | 与目录名一致 |
| `name` | `name` | |
| `version` | `version` | |
| `author` | `author` | |
| `description` | `description` | |
| `default_personality`（7 元数组） | `personality` | 对象 `{stubbornness,…,warmth}` 或 7 元数组 |
| `user_relations` | `relations` | 键名改为 `relations` |
| `default_relation` | `default_relation` | |
| `scenes` | `scenes` | |
| `evolution` | `evolution` | |
| `memory_config` | `memory_config` | |
| `identity_binding` | `identity_binding` | |
| `life_trajectory` / `life_schedule` / `knowledge` | 同名 | |
| `dev_only` | `dev_only` | |
| `min_runtime_version` | `min_runtime_version` | |
| `ollama_model`（若写在 manifest） | `ollama_model` | |

> **说明**：历史上单独的 `personality.json` 未进入宿主加载路径；七维人格以 manifest `default_personality` 为准，迁移时并入 `meta.personality`。

### 2.2 `settings.json` → `meta` 或 `slot_registry`

| legacy `settings.json` | v2 位置 | 说明 |
|------------------------|---------|------|
| `interaction_mode` | `meta.interaction_mode` | `immersive` \| `pure_chat` |
| `remote_presence` | `meta.remote_presence` | |
| `autonomous_scene` | `meta.autonomous_scene` | |
| `reply_quality_anchor` | `meta.reply_quality_anchor` | |
| `plugin_backends`（六槽枚举） | **`slot_registry`** | 每槽一条实例；见下表 |
| `plugin_backends.directory_plugins` | 各 directory 槽的 `plugin` / `plugins` | |

### 2.3 `plugin_backends` → `slot_registry`（默认实例键）

CLI 迁移与 `oclive_validation::plugin_backends_to_slot_registry` 使用下列**默认键名**（单实例时）：

| 模块 `type` | 默认实例键 | `backend` 来源 |
|-------------|------------|----------------|
| `memory` | `memory` | `plugin_backends.memory` |
| `emotion` | `emotion` | `plugin_backends.emotion` |
| `event` | `event` | `plugin_backends.event` |
| `prompt` | `prompt` | `plugin_backends.prompt` |
| `llm` | `llm` | `plugin_backends.llm` |
| `agent` | `agent` | `plugin_backends.agent` |
| `complex_emotion` | `complex_emotion` | 若 legacy 未配置则 `builtin` |

每条实例还需：`label`、`position`（同 type 从 0 递增）、directory 时的 `plugin`/`plugins`。

---

## 3. 自动迁移：`pack migrate-to-blueprint`

在 **oclivenewnew 仓库根**执行（需已 `cargo build -p oclive-cli` 或直接用 `cargo run`）：

```powershell
cd D:\oclivenewnew
cargo run -p oclive-cli -- pack migrate-to-blueprint roles\my_role
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `path` | （位置参数） | 角色包根目录，须含 `manifest.json` |
| `--remove-legacy` | **true** | 写入 `pipeline.ocblueprint` 后删除 `manifest.json` 与 `settings.json` |
| （省略 `--remove-legacy`） | — | 加 `--no-remove-legacy` 可保留旧文件（**不推荐**；`pack validate` 默认 v2 会拒绝双轨并存） |

成功输出示例：`Migrated to roles\my_role\pipeline.ocblueprint (legacy files removed)`。

**手工步骤（可选）**：用编写器「架构图」导出或编辑 `pipeline.ocblueprint`；自动迁移已覆盖常见字段，复杂 `directory_plugins` 请在编写器中核对实例键与 `plugin` id。

---

## 4. 迁移后校验

```powershell
cargo run -p oclive-cli -- pack validate roles\my_role
```

- 默认 profile 为 **v2**（等同 `default` / `blueprint-v2`）。
- 仅维护未迁完的 legacy 包时使用：`pack validate roles\legacy_role --profile legacy`。

**编写器**：打开包 →「运行全部检查」。**主应用**：设置页环境自检 → 加载角色 → 试聊一条。

**参考样例**：仓库内 `roles/mumu/`（仅 `pipeline.ocblueprint`，无 legacy 双文件）。

---

## 5. FAQ

### 迁移后旧配置还在吗？

- 默认 **`--remove-legacy`**：磁盘上**不再**保留 `manifest.json` / `settings.json`；内容已合并进 `pipeline.ocblueprint`。
- 若迁移前已 Git 提交，可从版本历史恢复旧文件。

### 可以回退吗？

- **无**宿主「一键回退 v1」；请用 Git 还原角色包目录，或保留迁移前的备份 zip。
- v2 与 legacy **不能** 同时存在同一角色根目录（校验失败）。

### 会话里改的「模块后端」还有效吗？

- v2 使用 **`slot_registry` + 会话 `slot_key` 覆盖**（Tauri `set_session_slot_override`）。
- C1 API `set_session_plugin_backend`（按 `module` 名）仍可用，内部映射到默认实例键（如 `memory` → `memory`）；**要求**包内已有 `slot_registry`。

### 编排 `steps[]` 去哪了？

- 已移除；对话主路径不读蓝图步骤 DSL。扩展逻辑请用目录插件、`plugin_backends` / `slot_registry` 与内核模块文档。

---

## 6. 检查清单（10 分钟）

| 分钟 | 动作 |
|------|------|
| 0–2 | 备份角色目录；阅读本文 §2 映射 |
| 2–4 | `pack migrate-to-blueprint <角色根>` |
| 4–6 | `pack validate <角色根>`（默认 v2） |
| 6–8 | 编写器打开，核对 `meta` 与 `slot_registry`（至少一个 `type: llm`） |
| 8–10 | 主应用 `load_role` + 试聊；必要时 `oclive doctor` 蓝图项 |

---

## 修订记录

| 日期 | 说明 |
|------|------|
| 2026-05-20 | 初版：v2 SSOT、`pack migrate-to-blueprint`、校验与 FAQ。 |
