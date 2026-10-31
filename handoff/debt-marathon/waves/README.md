# 波次记录的最小项目入口

当前状态读 [主台账](../../TECHNICAL_DEBT_INVENTORY.md)，执行准入读 [队列](../MARATHON_QUEUE.md)与对应计划，接手和最近事件读 [DEBT_CHANGELOG](../DEBT_CHANGELOG.md)。

详细波次流水已移至维护者个人文档；现行文档保留固定 Git 来源。下列两份仍承担必要维护规则，继续随源码保留：

- [Actions 固定引用规则与来源](WAVE-20260929-ACTIONS-PINS.md)。
- [债务权威行、引用与结构检查合同](WAVE-20260929-DEBT-REFERENCES.md)。

控制器需要的新波次只记录最小恢复信息，格式沿用 [模板](../WAVE_LOG_TEMPLATE.md)；逐命令日志与机器环境放仓库外，按 [文档边界](../../README.md#documentation-publishing-boundary)保存证据位置与可携带范围。目录整理不改变控制器权限、队列或 Done 条件。
