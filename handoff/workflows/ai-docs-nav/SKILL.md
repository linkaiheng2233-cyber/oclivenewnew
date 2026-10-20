---
name: ai-docs-nav
description: >-
  在 oclivenewnew 里需要定位"某个概念、命令、错误码或债务 ID 写在哪些文档"时使用：
  先用只读的 scripts/ai-docs.mjs 把候选缩到少量文档，再读原文对应章节。
  命中超过 20 份时先 --count 看规模，需要穷举再用 --all。它只做定位，不替代 SSOT、
  影响面分析或验收证据。
---

# AI 文档导航（可选 · 只读）

**SSOT 范围**：本技能只说明"什么时候用、怎么用、什么时候停"；工具行为以
[`handoff/catalog/README.md`](../../catalog/README.md) 为准，规范仍由各自 SSOT 拥有。

## 何时用

- 不知道某个概念 / 命令 / 错误码 / 债务 ID 落在哪份文档；
- 需要初筛"哪些文档提到 X"（耦合核对的第一步）；
- 新会话接手，想快速定位而不是逐份翻 `handoff/`。

## 何时不用

- 已经知道文档路径 → 直接读；
- 需要规则权威或状态判断 → [AGENTS.md](../../../AGENTS.md)、[AI_READING_INDEX](../../AI_READING_INDEX.md)；
- 需要判断改动影响面 → 按 G17 影响链核对，本工具不提供该结论；
- 需要验收证据 → 回到源码与测试。

## 三条命令

```sh
node scripts/ai-docs.mjs query --query <词>                   # 默认：每份文档一行，最多 20 份
node scripts/ai-docs.mjs query --query <词> --count           # 只报数量与穷举成本
node scripts/ai-docs.mjs query --query <词> --all             # 穷举：一行一文档
node scripts/ai-docs.mjs query --query <词> --select entries  # 逐标题 + 240 字引文
```

- 默认窗口按**文档**计（不是标题）；精选记录优先占前 3 席；
- 截断提示会写 `Showing 20/N` 并给出 `--all` 的 token 成本；
- `discovered` 是即时导航（`read-time-only; not pre-reviewed`），`curated` 是登记过的精选入口。

## 失败与回退

- 零结果、缺卡、失配都不要解释成"没有实现"；回退到既有阅读索引：
  [`AI_READING_INDEX.md`](../../AI_READING_INDEX.md) 与 [`handoff/README.md`](../../README.md)；
- 工作树有未提交文档时，命令可能以 exit 1 提示"所列记录需要核对"，这是状态提示，不是查询失败；
- 一轮查询后必须读原文对应章节；不拿命中片段或摘要当结论。

## 质量基线（可选复跑）

```sh
node scripts/ai-docs.eval.mjs          # 定位召回 5/8 @8、6/8 @12、8/8 @20；耦合 3/3 完整
node scripts/ai-docs.eval.mjs --gate   # 不达线时 exit 1
```

该脚本只衡量定位与覆盖，不代表原文正确、实现符合或运行验收。
