# 上游研究与本仓取舍

**SSOT 范围**：记录第二条模型分工流水线借鉴的公开思路与本仓边界；不复制源码、不定义业务契约。
**最后更新**：2026-09-05。

| 来源（固定版本） | 借鉴思路 | 本仓取舍 |
|---|---|---|
| [obra/superpowers writing-plans](https://github.com/obra/superpowers/blob/b36e0829c6d0140e93cfef2ca599b1b07d4a7797/skills/writing-plans/SKILL.md) · SHA `b36e0829c6d0140e93cfef2ca599b1b07d4a7797` · 查阅 2026-09-05 · MIT | 可执行步骤、依赖、验收和小范围计划 | 保留证据化 dispatch；不强制 2–5 分钟小步，也不要求每个计划复制代码。
| [obra/superpowers subagent-driven-development](https://github.com/obra/superpowers/blob/b36e0829c6d0140e93cfef2ca599b1b07d4a7797/skills/subagent-driven-development/SKILL.md) · 同 SHA · 查阅 2026-09-05 · MIT | 受限 worker、规格交接、质量审查 | 拒绝可自行裁决冲突的 “Rulings not stalls”、递归/频繁新 worker 和自动分支清理；本仓由用户授权与 controller 停止条件约束。
| [obra/superpowers verification-before-completion](https://github.com/obra/superpowers/blob/b36e0829c6d0140e93cfef2ca599b1b07d4a7797/skills/verification-before-completion/SKILL.md) · 同 SHA · 查阅 2026-09-05 · MIT | 先证据后完成声明、复验实际 diff | 纳入 GPT6 对真实 diff 与证据的验收；不要求每句话重跑全量测试。
| [anthropics/skills grader](https://github.com/anthropics/skills/blob/41bbe19d1a1a7eaab5e7bb9050a417e5c6cffc8f/skills/skill-creator/agents/grader.md) · SHA `41bbe19d1a1a7eaab5e7bb9050a417e5c6cffc8f` · 查阅 2026-09-05 · skill-creator 目录 Apache-2.0 | 依据真实 artifact 与过程，避免只测文件存在 | 本仓验行为、权力边界和证据链；不使用其 CLI eval 或 benchmark 框架。
| [openai/skills skill-creator](https://github.com/openai/skills/blob/49f948faa9258a0c61caceaf225e179651397431/skills/.system/skill-creator/SKILL.md) · SHA `49f948faa9258a0c61caceaf225e179651397431` · 查阅 2026-09-05 | 精简、按需参考、风险匹配自由度 | 以本仓 SSOT 和用户授权为准；本次未核对该仓整体许可证，不作推断。

本表只说明研究取舍，不证明任何上游流程已在 OCLive 中自动化，也不构成工具调用授权。
