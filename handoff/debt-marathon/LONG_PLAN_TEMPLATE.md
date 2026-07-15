# 长流程计划书模板 · `<DEBT_ID>`

> 复制为 `long-plans/<DEBT_ID>.md`。一文一债。  
> **强制：** 每书正文开头必须链 [`../AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md)；子 Agent 先读 GATES 再读分阶段。

---

## 元数据

| 字段 | 填写 |
|------|------|
| **债 ID** | |
| **台账锚点** | TECHNICAL_DEBT 行 |
| **标题** | |
| **尺寸** | L（债 Done）/ … |
| **Minimal / Full** | |
| **Owner 轨道** | main-repo / cross-repo / Human-only |
| **runner** | auto / human / skip（与 MARATHON_QUEUE 一致） |
| **状态** | Draft · Ready · In progress · Blocked · Closed |
| **最后更新** | YYYY-MM-DD |

---

## AI + OCLive（每书必有 · 不可删）

- **必读门禁：** [`../AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md)
- **流水线：** dev-pipeline 七阶段 + oclive-dev-pipeline；尺寸 L 不跳纪律/文档/总审
- **相关 G：** （列出适用项，至少含 G11/G14）
- **场景路径：** AI_READING_INDEX §9 技术债
- **禁止超前 Done；** Verification 数字走 AI_VERIFICATION_PROTOCOL

## 机器执行契约（`auto` / Ready 必填）

`npm run check:debt-marathon` 会校验此 JSON。`parentDebtDisposition=keep-open` 表示本书关闭也不得关闭父技术债。

```markdown
<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "<DEBT_ID>",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "keep-open",
  "currentStage": 0,
  "prerequisites": [],
  "stages": [
    {
      "id": 0,
      "title": "Align",
      "files": ["read-only"],
      "actions": ["Read gates and verify scope"],
      "checks": [{"command": "npm run check:debt-marathon -- --id <DEBT_ID>", "why": "Ready contract must pass before dispatch"}],
      "outputs": ["Verified scope"],
      "rollback": "No writes; block on conflict"
    }
  ]
}
-->
```

---

## 目标与非目标

### 目标（Done 定义）
- …

### 非目标（硬边界）
- …

---

## 影响域

- 路径 / crate / 文档：…

---

## 分阶段

> 一 Stage = 一次子会话。每 Stage 表必须含：文件范围 · 验收命令（applicable）· 产出 · 失败回退。

### Stage 0 · 对齐
| 项 | 内容 |
|----|------|
| 动作 | 读 GATES · 台账 · Cursor clean worktree · 关键澄清 |
| 验收 | 无未决关键；GATES 已读 |
| 失败回退 | 无写入；冲突转 needs-reconcile |

### Stage 1 · …
| 项 | 内容 |
|----|------|
| 文件范围 | |
| 动作 | |
| 验收命令 | |
| 产出 | |
| 流水线阶段映射 | ③实现 · ④自审 · ⑤ applicable 纪律脚本 |

### Stage N · 证据
| 项 | 内容 |
|----|------|
| 动作 | PR ·（授权才 merge）· Verification · Wave · 更新 QUEUE |
| Done-eligible | 仅远程硬门禁 + 台账诚实 |

---

## 子 Agent 粘贴块

```text
按 oclive 债偿还马拉松 · Implementer。严格 AI_AND_PIPELINE_GATES + oclive-dev-pipeline。

必读：GATES → long-plans/<ID>.md Stage <N> → TECHNICAL_DEBT 本行 → AGENTS → BOUNDARIES。

授权：仅 Stage <N>。禁扩 Full、禁顶层新 md、禁无 CI Done、禁合 main（除非授权）。

结束：变更摘要 · PASS/FAIL · GATES §6 勾选 · Wave 路径 · 可否进下一 Stage。
```

---

## 关闭条件

- [ ] Stage 完成或 Deferred  
- [ ] 台账与证据一致  
- [ ] Wave + QUEUE 进度  
- [ ] 本书 Closed  
