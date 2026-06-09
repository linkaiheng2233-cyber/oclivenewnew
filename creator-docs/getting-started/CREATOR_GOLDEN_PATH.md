# 创作者黄金路径

**受众：** 普通人 / 初级创作者 · **不**重复内核六槽文档。

完整规范见 [ROLE_PACK_SPEC.md §0](role-pack/ROLE_PACK_SPEC.md) 边界说明。

---

## 1. 选预设，先聊起来

- 桌面首启：[预设画廊](../src/components/onboarding/PresetRolePicker.vue) 或 **AI 剧场**（`distro_id=theater`）。
- 日常聊：`pure_chat` — 隐藏场景/插件复杂度，适合第一印象。
- 官方示例包：`roles/mumu`（非产品上限）。

## 2. 改人格 / 关系 / prompts（初级创作者）

在角色包目录内编辑（**勿动** `slot_registry` / 蓝图调度）：

| 文件 | 作用 |
|------|------|
| `pipeline.ocblueprint` → `meta` | 展示名、关系、场景列表 |
| `core_personality.txt` | 核心人格 |
| `prompts/` | 场景/关系提示词 |
| `user_identities/` | 用户身份模板 |
| `config.json` → `reply_post_processor` | 可选回复后处理（默认关） |

保存后重启或热重载（`oclive dev` 监听 `roles/`）。

## 3. 何时打开编写器（暗门）

- 需要可视化编辑七维、场景 JSON、导出 `.ocpak` 时 → **oclive-pack-editor**。
- 剧场壳内 **「改性格」** → 深链到对应 `roles/<id>/`（配置 `VITE_OCLIVE_PACK_EDITOR`）。
- 编写器 **不**向简单创作流暴露 `slot_registry` / `runtime_config` / `expert_routing`。

## 4. 后处理（可选）

- 表单：`oclive-pack-editor` → 角色包面板 → `config.json` 后处理区。
- 预研插件：`examples/reply-post-process-polish/`（剧场局部补丁技术，非剧场产品本身）。

## 5. 发布角色包

```bash
cargo run -p oclive-cli -- pack validate roles/your-role
cargo run -p oclive-cli -- pack export roles/your-role
```

拷贝到任意宿主的 `OCLIVE_ROLES_DIR` 即可加载（见 [CROSS_HOST_MEMORY.md](role-pack/CROSS_HOST_MEMORY.md)）。

---

## 相关文档

- [DOCUMENTATION_INDEX.md](getting-started/DOCUMENTATION_INDEX.md) — 全站索引（含内核）
- [ROLE_PACK_SPEC.md](role-pack/ROLE_PACK_SPEC.md) — 契约 SSOT
- [THEATER_V0_ACCEPTANCE.md](../handoff/THEATER_V0_ACCEPTANCE.md) — AI 剧场 v0 验收
