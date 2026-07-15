# Plan 输出模板（阶段 ① · OCLive overlay）

在通用 `~/.cursor/skills/dev-pipeline/plan-template.md` 上**追加**以下段落（勿省略）。

```markdown
## OCLive 对齐（overlay）

- **尺寸**：S / M / L（触及台账 Done / main CI / 发版 → L）
- **风险触发器**：（契约 / 迁移 / 权限 / 供应链 / 跨仓 / 无；风险覆盖文件数）
- **相关 G 约束**：（如 G1 · G6 · G11）
- **场景路径**：[`AI_READING_INDEX` §9](../../../handoff/AI_READING_INDEX.md) — （小节名）
- **影响域细化**：（kernel / distros/shared / desktop-tauri / chat-pro / 姊妹仓 / handoff）

## OCLive 验收清单（只勾 applicable）

- [ ] `npm run check:rust`（或更窄测 + 说明）
- [ ] `npm run check:ci-local`（集成行为 / 跨宿主 / 债收口；纯静态门禁不自动适用）
- [ ] `npm run check:release`（发版 / 债收口）
- [ ] `node scripts/dimension5-acceptance.mjs --ci`（N = 脚本输出）
- [ ] `node scripts/check-error-codes-drift.mjs`
- [ ] `node scripts/check-markdown-links.mjs`（默认仅 human-docs/modules）
- [ ] `cargo test --workspace --doc`
- [ ] **远程 `ci.yml` success**（L：push 后；Partial→Done 硬门禁）
- [ ] TECHNICAL_DEBT 回写 HEAD SHA + 日期 + 证据（若动台账）

每个已勾命令后补一句 **applicable 原因**；未勾的大门禁不需要为了形式运行。

## OCLive 证据状态

- **目标**：Implemented / Locally verified / Done-eligible
- **远程 CI**：不适用 / 已授权可 push / 无权限待维护者执行
- **台账动作**：不动 / 保持 Partial/OPEN / 证据齐全后升 Done

> Plan todos `completed` ≠ Done。本地 PASS 不能代替远程 main CI（若任务声称恢复/收口）。
```

保存建议：`.cursor/plans/<task-slug>.plan.md`
