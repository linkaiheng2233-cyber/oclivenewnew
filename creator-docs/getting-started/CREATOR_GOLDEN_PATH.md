# 创作者黄金路径（V4 大纲 · ≤2 页）

**状态**：Wave 4 扩写（陌生人 Theater 自动化前置通过）；详细截图待人工陌生人复测。  
**受众**：初级创作者 —— **30 分钟内**完成可对话角色包，**不涉及** `slot_registry` / 蓝图编排。

---

## 0. 前置（5 分钟）

- 安装 OCLive 桌面或打开 [oclive-pack-editor](https://github.com)（姊妹仓编写器）
- 本地 Ollama 或云端 API Key 二选一（剧场 demo 可零配置）
- 克隆/下载官方示例包 `distros/chat-pro/roles/mumu` 作参照（非上限）

## 1. 初始化角色包（5 分钟）

1. 编写器：**新建角色包** → 填写 `manifest.json`（`id`、`name`、`version`）
2. 选择 **v2 最小模板**（`pipeline.ocblueprint` + `settings.json` 默认六槽 builtin）
3. 保存到 `distros/chat-pro/roles/{your_id}/`

**禁止在本路径修改**：`slot_registry` 多实例、`groups`、Experimental 核。

## 2. 身份与人格（10 分钟）

| 文件 | 做什么 |
|------|--------|
| `prompts/system.md` | 一句话角色 + 口吻示例 |
| `config.json` → `reply_quality_anchor` | 可选：替换默认回复锚点（不可替换 guardrails） |
| `user_identities/`（可选） | 一个默认 `.md` + `index.json` |

**验收**：编写器预览 Prompt 含「系统 / 角色 / 用户」三层，无蓝图 step 字段。

## 3. 本地试跑（5 分钟）

### 桌面通用

```powershell
$env:OCLIVE_ROLES_DIR = "path\to\roles"
npm run tauri:dev
```

- 选择新角色 → 发送「你好」
- **Ctrl+Shift+F** 打开插件管理（`SimplePluginManagerPanel`）；**Ctrl+Shift+M** 打开模型管理
- 设置 → 模型管理：确认 LLM 后端可达

### 剧场发行版（`distro_id=theater`）

```powershell
npm run dev:theater
```

- 早餐场景 · 双角色对比 · 3 poke chips（见 [theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md) §4）
- 自动化烟测：`npm run test:unit` → `distros/theater/src/theater.acceptance.test.ts`（9 测）

## 4. 分发与下一步（5 分钟）

- `oclive-cli` / 编写器：**打包** → `.oclive-plugin` 或整包 zip
- 文档索引：[ROLE_PACK_SPEC.md](./role-pack/ROLE_PACK_SPEC.md) · [DOCUMENTATION_INDEX.md](./getting-started/DOCUMENTATION_INDEX.md)

**进阶（不在 30 分钟路径）**：目录插件、remote 槽、`distro.oclive.toml`、Agent/MCP。

---

## 关联债项

- **V4 完整版**：陌生人 Theater 测试 ≥60% 通过后，从本大纲扩为分步截图文档。
- **冻结期**：不引导创作者改 `runtime_config.dual_core` 或蓝图 `steps[]`。

[English](../creator-docs-en/getting-started/CREATOR_GOLDEN_PATH.md)（待镜像）
