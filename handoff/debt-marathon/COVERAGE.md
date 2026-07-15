# 计划书覆盖审计（对照 TECHNICAL_DEBT）

**更新日期：** 2026-07-16  
**结论：** 活跃 OPEN/Partial/Observe/Deferred/冻结项均有 `long-plans/` 条目；§4 长期 Deferred 已于本轮补 stub。  
**详略：** `auto` 书 = Minimal 可执行分阶 + `oclive-marathon-contract`；`skip`/`human` = stub + 触发条件。`npm run check:debt-marathon` 强制校验所有 auto 书的文件范围、验收理由、产出、回退与父债处置。**深度细则以 [`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md) + 七阶段 Skill 为准**（不在每本 stub 重复）。

## 覆盖矩阵

| 台账状态类 | 覆盖 | 注 |
|------------|------|-----|
| OPEN 主仓可 Minimal | ✓ auto 书 | T-DOC-02 · D-ROLEVER · RESILIENCE · CROSS/DIST/MARKET · VOICE-06/07 · SUPPLY-05-Full · FOLLOWUP-PR123 |
| OPEN 跨仓/人工 | ✓ human stub | VSCODE · PE-* · P0-STRANGER-EXT |
| Partial | ✓ | K-PERF-10 skip |
| Observe | ✓ skip stub | SLOT/TRAIT/PORT/POLICY/ORPHAN · F4 · SUPPLY-04/08 · VOICE-05 等 |
| Deferred / 冻结 | ✓ skip stub | VOICE-01/08 · SUPPLY-06/07 · MEGA · MODE3 · §4 项 |
| 已 Done | 不建施工书 | PLATFORM/LLM/VOICE-04 Minimal 等；仅 FOLLOWUP 跟随项 |

## 本轮补齐的 §4 / 缺口 stub

| ID | 文件 |
|----|------|
| K-PERF-15 | long-plans/K-PERF-15.md |
| V-FUSED-01 | long-plans/V-FUSED-01.md |
| V-LORA-WORKSHOP-01 | long-plans/V-LORA-WORKSHOP-01.md |
| D-OPUS-05-P2 | long-plans/D-OPUS-05-P2.md |
| K-UID-POST-01 | long-plans/K-UID-POST-01.md |
| ROADMAP-MODAL-EDGE | long-plans/ROADMAP-MODAL-EDGE.md（§3.5–3.7 + §5.3 UGC 合并 stub） |
| DUAL-CORE-FREEZE | long-plans/DUAL-CORE-FREEZE.md |
| D-READ-03 | long-plans/D-READ-03.md |

## 详略诚实声明

| 类型 | 详细度 | 是否够隔夜 |
|------|--------|------------|
| auto Minimal | Stage 表 + 机器契约 + 非目标 + 停条件 | **够**（须通过 `check:debt-marathon`） |
| auto Full 战役 | 机器契约 + 有限 Stage + 停条件，禁止假 Done | 够「跑到 blocked」 |
| skip/human stub | 薄 · 故意 | 够「跳过不犯错」 |

若要加强某本 auto 的行级文件清单：优先加厚该 ID 的 long-plan，而不是膨胀 stub。
