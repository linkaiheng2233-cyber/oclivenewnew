# 编写器能力路线图（含情绪素材）

**状态**：日程项；**不阻塞** VS Code 发行版 Phase 1。  
**当前导出**：`oclive-pack-editor` / 工作室「简单创作」已支持向 `assets/images/` **上传文件**（文件名需与 oclive 约定一致，如 `happy.png`）。

---

## 已收敛（v0.2 现状）

| 项 | 说明 |
|----|------|
| 立绘标签 | 内核 `Emotion` **7 类**；立绘 LLM `ALLOWED` 同上（见 `portrait_emotion_engine.rs`） |
| 前端映射 | `emotion-assets.ts` + `disgust_*` 文件名扩展；未知 tag → `{tag}.png` 或 emoji |
| 包内路径 | `roles/{id}/assets/images/*.png` |
| 导出 | 编写器 zip 可附带 `emotionImages[]` |

**暂缓**：「情绪族」（多套立绘皮肤 / 更多离散表情）—— 开发任务紧，列入下文 Phase B。

---

## Phase A — 情绪图片编辑（下一版编写器）

| 能力 | 说明 |
|------|------|
| 素材面板 | 7 标签缩略图网格 + 拖拽替换 + 缺失提示 |
| 预览 | 与桌面 `CharacterInfo` 相同候选路径规则 |
| 校验 | 导出前检查必填 `happy` / `neutral`（可配置） |
| 与简单创作合并 | 不再只靠「选 N 个文件」，而是 **tag → 文件** 绑定 UI |

---

## Phase B — 情绪族扩展（需内核协同）

| 能力 | 说明 |
|------|------|
| 契约 | `meta.emotion_family` 或 `assets/emotions/{family}/*.png` |
| 内核 | 扩展 `Emotion` 或允许立绘 `ALLOWED` 可配置列表 |
| 发行版 | 桌面 full / VS Code lite 导出不同族或占位图 |

---

## Phase C — 分级导出（多发行版）

| Profile | 用途 |
|---------|------|
| `full` | 桌面 Tauri |
| `vscode-lite` | VS Code 扩展（见 [`creator-docs/role-pack/VSCODE_DISTRIBUTION.md`](../creator-docs/role-pack/VSCODE_DISTRIBUTION.md)） |
| `headless` | `robot-soul` / CI |

实现落点：`exportPack.ts` 的 `buildRolePackFiles` 分支 + 编写器 UI 单选。

---

## 优先级（2026-05 共识）

1. **VS Code 发行版**（`oclive-vscode`）— 进行中  
2. **Phase A** 情绪图片编辑 — 编写器  
3. **Phase C** 分级导出 — 编写器  
4. **Phase B** 情绪族 — 需 RFC + 内核
