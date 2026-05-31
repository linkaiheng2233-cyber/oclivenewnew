# 测试与路演文档索引

| 文档 | 用途 | 交给谁 |
|------|------|--------|
| **[`CODEX_测试指南.md`](../codex-testing/CODEX_测试指南.md)** | **转发 Codex 用 · 单页总览 + 一键任务** | 复制第七节给 Agent |
| [`CODEX_TEST_RUNBOOK.md`](../codex-testing/CODEX_TEST_RUNBOOK.md) | **Track A · 文本轨**（cargo/OOCP/Vitest/E2E 退出码） | DeepSeek、Codex、纯文本 Agent |
| [`CODEX_TEST_RUNBOOK_VISION.md`](../codex-testing/CODEX_TEST_RUNBOOK_VISION.md) | **Track V · 视觉轨**（Playwright 截图/UI 定性） | GPT-4o、Claude vision、Gemini vision |
| [`ROADSHOW_NARRATIVE.md`](ROADSHOW_NARRATIVE.md) | 路演口述稿 | 人工答辩 |
| [`项目说明.md`](项目说明.md) | 项目申报说明 | 评审 / 合作方 |

报告输出目录：`test-reports/`（Agent 自动创建，默认不提交 git）。

---

## 推荐流程

```text
1. DeepSeek / 文本模型 → Track A → test-reports/codex-track-a-*.md
2. 多模态模型 → Track V → test-reports/codex-track-v-*.md
3. 人工或任一模型 → 合并 §5 模板 → test-reports/codex-merged-*.md
```

---

## 一键提示

**Track A（DeepSeek / 无图）**：

```text
执行 oclivenewnew/dev-notes/codex-testing/CODEX_TEST_RUNBOOK.md 第 I 部分（Track A only）。
不要读 CODEX_TEST_RUNBOOK_VISION.md；不要分析任何图片。
顺序: E0 → A → B → C → D → E → F →（G/H 按 SKIP）→ A-T0 build:e2e → A-T1 → A-T2 → J。
OOCP 必须 OCLIVE_HTTP_API_MOCK_LLM=1。写入 test-reports/codex-track-a-<时间>.md。不要 git commit。
```

**Track V（GPT-4o / Claude vision / Gemini）**：

```text
执行 oclivenewnew/dev-notes/codex-testing/CODEX_TEST_RUNBOOK_VISION.md（Track V only）。
不要重跑 Track A 的 cargo/clippy/OOCP。
先 build:e2e，再 Playwright；失败必须读 test-results PNG 写 visual_assessment。
读取 test-reports/codex-track-a-*.md 的 handoff 并做 V-REV。
写入 test-reports/codex-track-v-<时间>.md。不要 git commit。
```
