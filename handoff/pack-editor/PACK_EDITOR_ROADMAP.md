# 编写器能力路线图（立绘 catalog · 视觉舞台）

**状态**：2026-06-13 与 RFC 对齐；**代码分阶段见** [PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN.md](../PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN.md)  
**姊妹仓编写器副本**：[`oclive-pack-editor/handoff/PACK_EDITOR_ROADMAP.md`](../../../oclive-pack-editor/handoff/PACK_EDITOR_ROADMAP.md)

**RFC**： [RFC_PORTRAIT_FACILITY.md](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md) · [RFC_VISUAL_PRESENTATION_FACILITY.md](../../creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md)

---

## 已收敛（v0.3 现状）

| 项 | 说明 |
|----|------|
| 立绘标签 | 内核 `Emotion` **7 类**；`pick_portrait_emotion` ALLOWED 同上 |
| 前端 | `emotion-assets.ts` + `CharacterInfo` 文件名候选 |
| 编写器 | 简单/高级共用 `EmotionAssetsControl`（批量 pick/append） |
| 导出 | `assets/images/{原文件名}` |

**下一共识**：文件名 **不再** SSOT；**catalog `id` + AI 表现导演**（第 3 设施 RFC）。

---

## Phase A — 简单创作 · 7 槽（编写器）

| 能力 | 说明 |
|------|------|
| UI | 7 固定槽（happy…shy）；无 append / 无簇 |
| 导出 | 生成 `config.json` → `portrait_catalog` 7 条 |
| 命名 | 文件名任意；槽位绑定 `id` + 默认 `desc` |
| 检查 | 缺槽警告（可配置是否阻断导出） |

---

## Phase B — 高级创作 · catalog 编辑

| 能力 | 说明 |
|------|------|
| 簇 | 追加条目、`cluster` 标签、多 `desc` |
| 预览 | 条目列表 + 缺失 tag 提示 |
| 导出 | 完整 `portrait_catalog.assets[]` |

---

## Phase C — 角色舞台 · 视觉表现

| 能力 | 说明 |
|------|------|
| UI | 高级：`visual_presentation`、Live2D/3D 资源 |
| 校验 | backend 与 catalog `kind` 一致性提示 |

---

## Phase D — 分级导出（多发行版）

| Profile | 立绘 catalog | 视觉表现 |
|---------|--------------|----------|
| `vscode-lite` | 7 槽或精简 | off |
| `desktop-full` | 完整 | image / live2d 可选 |
| `theater` | 完整 + 多 kind | stage_full |

---

## 优先级（2026-06）

1. 主仓 Phase 1–2 + 编写器 Phase A–B  
2. 主仓 Phase 3 表现导演 AI  
3. 编写器 Phase D 分级导出  
4. 主仓 Phase 4–5 + 编写器 Phase C  

原 **Phase B「扩展 Emotion 枚举」** 已由 **portrait_catalog + AI 选 id** 取代。
