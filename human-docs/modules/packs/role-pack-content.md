# 角色包开工包 · 文案与人设内容

> **读者**：只改 `roles/{id}/` 内文案、场景、人设文件的创作者（非内核）。  
> **读完能做什么**：在 G1 边界内改 mumu 等人设，不碰 `slot_registry` 与主链代码。  
> **耗时**：约 **40 min**  
> **SSOT 范围**：人类 checklist；边界见 [ROLE_PACK_BOUNDARY](../../handoff/ROLE_PACK_BOUNDARY.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[role-pack-config](role-pack-config.md) · [CREATOR_LEARNING_PATH](../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§14 配置四层 · 角色包层](../../handoff/MODULE_MAP_AND_HANDOFF.md#14-配置四层谁可改什么)  
- **目录 SSOT**：`distros/chat-pro/roles/{role_id}/`  
- **Tier0 人设真源**：`core_personality.txt`（**非** `prompts/system.md`）  
- **不进**：`process_message` · 蓝图 `slot_registry`

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| `core_personality.txt` · `scenes/` · `prompts/`（含 `deep_capsule.txt`）· 立绘 catalog 资源 | 改 `slot_registry` / `plugin_backends`（G1） |
| `reply_quality_anchor`（仅替换默认锚点，**不可替** guardrails） | 覆盖 `KERNEL_DIALOGUE_GUARDRAILS` |
| 姊妹仓 **oclive-pack-editor** 可视化编辑 | 在角色包任务里改内核迁移或 DTO |

---

## 3. 阅读清单

1. [ROLE_PACK_BOUNDARY](../../handoff/ROLE_PACK_BOUNDARY.md)  
2. [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)  
3. [01 简架构 §六槽 vs 角色包](../../01_ARCHITECTURE_SIMPLE.md)  
4. [distros/chat-pro/roles/README_MANIFEST](../../distros/chat-pro/roles/README_MANIFEST.md)  
5. [PACK_VERSIONING](../../creator-docs/role-pack/PACK_VERSIONING.md)

---

## 4. 开发流程

- [ ] L0–L2 跑通主仓  
- [ ] 确认改动的文件在 `roles/{id}/` 下  
- [ ] 文案改动后 `npm run tauri:dev` 目视一轮对话  
- [ ] 若动 manifest 版本 → 遵循 PACK_VERSIONING  
- [ ] 需要 `config.json` 字段时 → 转 [role-pack-config](role-pack-config.md)

---

## 5. 验收

- [ ] Prompt 中人设来自 `core_personality.txt`  
- [ ] PR 未包含 `slot_registry` 或内核 Rust 改动（除非维护者另开任务）  
- [ ] guardrails 仍由内核注入  
- [ ] 角色包路径与 `README_MANIFEST` 一致

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `prompt` 槽 | 读取角色包段落组装 prompt |
| `memory` | 人设不进 LTM；对话记忆独立 |
| [role-pack-config](role-pack-config.md) | `config.json` 行为开关 |
