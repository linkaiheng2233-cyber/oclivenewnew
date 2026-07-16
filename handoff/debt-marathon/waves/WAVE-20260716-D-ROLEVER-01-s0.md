# WAVE-20260716-D-ROLEVER-01-s0

> 计划书：[`../long-plans/D-ROLEVER-01.md`](../long-plans/D-ROLEVER-01.md)

## 摘要

| 字段 | 填写 |
|------|------|
| **债 ID** | D-ROLEVER-01 |
| **执行 Stage** | Stage 0 · Locate canonical migration wording |
| **分支** | `debt/t-doc-02-theater-status`（马拉松脚手架尚未合 main；后续 Stage 1 开 stack 分支） |
| **日期** | 2026-07-16 |
| **执行面** | [oclive-debt-stage](ce9e1824-c4f8-4fe9-8215-e9650840133f) |
| **状态三态** | Implemented（只读） |

## 证据

| 项 | 值 |
|----|-----|
| HEAD SHA | `f7e723001a797450753354e3b6f5da7ef084eaad` |
| Base SHA | `f7e723001a797450753354e3b6f5da7ef084eaad` |
| Changed files | none |
| Claim | `406a14b9-9269-4688-a10b-7e13106ce448` · attempt 1 |
| 检查 | `npm run check:debt-marathon -- --id D-ROLEVER-01` → **PASS** |

## 对齐结论

- 中英 `ROLE_PACK_SPEC` **均无**「版本/迁移」专节；现有仅为字段级 `version` / `schema_version` / `min_runtime_version`
- 姊妹文 `PACK_VERSIONING` / `V*_MIGRATION` 已存在 → Stage 1 **只链不抄**（G14）
- 插入锚点（中文）：`## 10. 可选 · voice_profile.json` 之后新建 `## 11. 版本与迁移`
- EN mirror：`check-doc-mirror` HIGH_TRAFFIC → **必须** 对等短节

## 下一跳

`node scripts/cursor-marathon.mjs claim --debt D-ROLEVER-01 --stage 1 --agent oclive-debt-stage --capabilities local-write,test,commit,push,open-pr --authorization "user 2026-07-16 standing auth"`

## GATES §6

- [x] 只动文件范围（read-only）
- [x] 已读 GATES §2–§3
- [x] 验收 PASS 已列
- [x] 未升 Done / 未合 main
- [x] QUEUE + checkpoint 由父 Agent 更新
