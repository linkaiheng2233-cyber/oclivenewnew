# 本地说明（本目录常被 gitignore）

正式 **长流程计划书** 与 **波次日志** 的 SSOT：

`handoff/debt-marathon/`

本 Skill 目录是 Cursor 本机入口；配套：

- `.cursor/agents/oclive-debt-stage.md`：一次只执行一个 Stage 的自定义子 Agent
- `.cursor/hooks.json` + `.cursor/hooks/oclive-marathon-stop.mjs`：Cursor IDE stop hook 续轮
- `scripts/cursor-marathon.mjs`：本机 session / checkpoint / 熔断
- `scripts/check-debt-marathon.mjs`：Git 内计划契约门禁

需要本机草稿时可建 `scratch/`（勿当仓内权威）。Cursor Background/Cloud Agent 不依赖 lifecycle hook，按 Git 内 Wave 恢复。

同步：改规程改本 `SKILL.md`；改计划书内容只改 `handoff/debt-marathon/long-plans/`。
