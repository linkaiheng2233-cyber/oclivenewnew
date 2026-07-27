# 编写器能力路线图（立绘 catalog · 视觉舞台）

**状态**：2026-06-13 与 RFC 对齐；现行设施边界见 [立绘 RFC](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md) 与 [视觉表现 RFC](../../creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md)。
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

## Phase E — 对话节奏 · Turn Thinking（Wave F · schema 已落地，编写器暂缓）

**台账**：主仓 **K-TURN-F1** · 编写器 **PE-TURN-01**（[`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) §3 Observe）。

| 能力 | 说明 |
|------|------|
| UI | **简单**：`auto_deep_min_chars`、Deep 关键词列表、强事件 latch（争吵→直到道歉）；**高级**：可选 AND 规则组（角色包内，内核 OR 默认 + 包内叠加） |
| 导出 | 写入角色包 **`config.json` → `turn_thinking`**（契约已落地；与 `memory` / `relation` 同级） |
| 校验 | 与 `oclive_validation` + 内核 `HostProfile` merge 一致；**不提供**玩家端 Deep/Fast 开关 |
| 试聊 | 编写器内试聊显示本轮 `TurnThinkingMode` + reasons（debug，非 P0） |

**依赖状态**：[`RFC_TURN_THINKING_PERSISTENCE.md`](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)
与 K-TURN-F1 已落地；PE-TURN-01 按当前产品安排暂缓，不阻塞角色内容扩展。

---

## Phase F — 成人角色扩展（v1 已落地）

| 能力 | 说明 |
|------|------|
| 独立页面 | “成人扩展”与简单/高级编辑职责分离，不把成人内容塞回基础人设和基础场景 |
| 进入前置 | 必须先有一份完整且通过基础校验的角色包 |
| 编辑分区 | 成人状态人设、成人对话指导、创作者/AI 节奏、按基础场景映射的走向/动作/对话 |
| 导入导出 | 根文件 `adult_extension.json` v1 与完整基础包合并 roundtrip；旧包无扩展时不生成文件 |
| 校验 | 角色成年声明、schema、正数间隔、节奏枚举与基础场景引用一致性 |

运行时契约见 [`ROLE_PACK_SPEC.md`](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#chat-pro-成人角色扩展adult_extensionjson--可选)；后台预生成队列仍由主仓 `K-ADULT-01` 的 D25～D29 跟踪，不在编写器中提前发明第二套状态格式。

---

## 优先级（2026-06）

1. 主仓 Phase 1–2 + 编写器 Phase A–B  
2. 主仓 Phase 3 表现导演 AI  
3. 编写器 Phase D 分级导出  
4. **Wave F**：内核 K-TURN-F1 已完成；角色包可手工配置，编写器 Phase E 暂缓
5. 主仓 Phase 4–5 + 编写器 Phase C  

原 **Phase B「扩展 Emotion 枚举」** 已由 **portrait_catalog + AI 选 id** 取代。
