# 模块行为质量对比台

该工具用同一组固定角色、场景和 replay，分别检查 memory、emotion、prompt、LLM 四个模块的可判定行为。它回答“这套模块配置是否满足当前 fixture 的明确要求”，不回答“哪个模型在所有场景中客观最好”。

## 快速使用

先构建内核服务端，再运行当前内核远程槽基线，并与仓库参考观察并列：

```bash
cargo build -p oclive-kernel-server
npm run quality:modules
```

输出包含三段：

- `observations`：当前运行经真实 HTTP、replay 与 remote-slot 链采集的、合同允许的观察；
- `report`：当前配置逐用例、逐维度的判定；
- `comparison`：参考配置与当前配置的并列结果。两套配置必须使用完全相同的 suite digest，且模块身份组合不能相同。

也可比较已有观察文件：

```bash
node scripts/module-quality-compare.mjs \
  --observations path/to/config-a.observations.json \
  --observations path/to/config-b.observations.json
```

每个 observation 必须声明四个模块的 `id` 与 `version`。不要仅改 `run_id` 把同一模块组合伪装成两套配置；比较器会拒绝重复组合。

## 输入与评分

版本化 suite 位于 `examples/module-quality-harness/fixtures/suite.v1.json`。每个 case 固定：

- `role_id`、`scene_id` 与多轮 `replay`；
- memory/prompt 必须出现与禁止出现的文本；
- emotion 允许标签；
- LLM 必须命中任一文本、禁止文本与用户复述率上限。

四维分数独立保留，不计算总分。失败意味着某条明确 fixture 期望不满足，不等于整个模块或模型在所有场景都差。通过也只证明当前 fixture 合同，不替代人工角色自然度评估、长对话测试或真实模型抽样。

## 隐私与隔离

运行器复制 fixture 使用的角色包到临时目录，临时切换四个 slot 到本地 JSON-RPC sidecar，并通过现有 `/chat/storage` 与 `/chat` 链执行。它只把 `mq-*` fixture memory 写入安全观察 Prompt；其他运行时记忆、完整生产 Prompt、用户令牌都不进入报告。

临时 API token、LLM token、数据库和角色副本只存在于单次临时目录；退出时回收内核进程树并清理目录。报告仍可能包含 fixture 中的对话文本，因此不要把私密真实对话直接改写进 suite。

## 质量与性能必须分开

`comparison.quality` 只含行为判定。`comparison.performance.status` 当前固定为 `not_measured`，不会从执行耗时推断延迟、吞吐、CPU、GPU 或内存。性能仍使用 `oclive bench` 和专门的硬件矩阵；如需同时展示，应把两类报告并列，不要合成一个总分。

## 维护者检查

```bash
npm run test:module-quality
npm run check:module-compat
node scripts/check-doc-mirror.mjs
node scripts/dimension5-acceptance.mjs --ci
```

`test:module-quality` 是离线合同自测，不启动模型；`quality:modules` 才启动本地内核采集。发布或技术债关闭还需要目标提交远程 CI，不能用本地全绿代替。

---

[English](../../creator-docs-en/testing/MODULE_QUALITY_HARNESS.md)
