# 第二轮马拉松计划：解除阻断后再偿还

> 入口门禁：[`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md)。债务状态 SSOT：[`../TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md)。本计划先处理外部阻断，不擅自把 human/skip 项改成 auto。

## 目标

让下一轮只在证据、权限和运行条件齐备后启动自动 Stage；优先收敛已经形成 PR 栈的事项，再开放新的本地债务。

## 阶段

### Wave 0：第一轮封存与证据对齐

- 核对 `MARATHON_QUEUE.md`、long-plan、inventory、Git SHA、PR 状态和 CI。
- 保持 `pr-open` / `blocked` / `human` / `skip` 原状。
- 产出：本轮 closeout、无冲突的队列和可复现的下一步命令。

### Wave 1：PR 栈审查窗口

- 审查 `#124 → #125 → #126` 的依赖、diff 和新 SHA CI。
- 只有获得明确合并授权后才执行 merge；否则维持 `pr-open`。
- 产出：每个 PR 的 accept/revise/block 结论。

### Wave 2：解除外部阻断

- `V-VSCODE-PERF-05`：姊妹仓和 `.vsix` 实机权限。
- `K-CROSS-01`：三平台 smoke 环境。
- `K-DIST-01`：签名/updater 密钥与发布权限。
- `K-VOICE-07`：RFC v2 锚点。
- 产出：授权快照或稳定 blocker code；没有条件则不派发 Implementer。

### Wave 3：下一批自动偿还

仅当 Wave 1/2 消除冲突后，从 queue 中把明确具备 `local-write,test` 条件的单项计划置为 `ready`，按 seq 每次只领取一个 Stage。候选顺序：`K-RESILIENCE-01 Full`、`K-SUPPLY-05-Full`、`D-ROLEVER-01`、`T-DOC-02`。

## 结束条件

- 无可验证证据的事项保持 `Partial` / `blocked`。
- 所有 Stage 均有 Wave、命令、SHA 和测试结果。
- 无 runnable auto 时正常结束，不为了延长轮数而改动 skip/human 状态。
