# D-DEBT-LEDGER-01：Event Stream 状态行与历史证据分离

**范围**：从干净的 `5f2836ce67a8103882a383765d5a01913ccbb365` 开始，仅处理主台账 `K-EVENT-STREAM-01` 的长状态行与治理父债 `D-DEBT-LEDGER-01` 的进展描述。计划见[第二轮计划](../ROUND-02-PLAN.md#d-debt-ledger-01--event-stream-长历史出权威行2026-10-02)，当前状态仍以[主台账](../../TECHNICAL_DEBT_INVENTORY.md)为准；[旧状态行快照](../../archive/TECHNICAL_DEBT_EVENT_STREAM_STATUS_20261002.md)只供追溯。

## 为什么要分离

原 `K-EVENT-STREAM-01` 单行有 2,571 个字符、4,340 个 UTF-8 字节。它在一个状态格内连续叙述 B0、Stage A 和 R0–R6，末尾才列出 Production 缺口。比如 R5/R6 是 synthetic-only / networkless 决策证据；若接手者只扫到“跨进程 lease”或“恢复策略”，可能误以为生产消费与恢复已经交付。新状态格先写 `OPEN · Production Stream not implemented`，再区分 R1–R4 有限真实样本、R5/R6 合成证据和仍缺的生产能力。

这只改变阅读顺序与历史承载位置，不重新裁定 RFC、改变验收范围或授权 Stage C。Event Ring 的 B0 Trace-only 仍无读取、消费、Replay、Prompt 或主动触发权；A.2.2.2 与 Stage C 仍未完成。

## 保真与验证

- 起点提交中的原表格行完整放入历史快照的 fenced block；逐字比较相等，原行 SHA256 为 `3C421D9F2DFCC11C7D68CFFE0BEE0741DCBAD4DC8B4E3A1D3B845B51BA575E9C`。快照文件顶部另给当前台账与 RFC 的可点击链接，fenced 原文中的旧相对地址不冒充新路径。
- 权威行的前四格（ID、问题、优先级、完成条件）与起点逐字相同；该行由 4,340 缩至 1,281 UTF-8 字节。独立脚本对比起点与当前 123 个独立 ID 行，只有 `K-EVENT-STREAM-01` 和 `D-DEBT-LEDGER-01` 两行变化，未改变其它状态。
- 本轮只作文档结构整理，没有运行 QQ、B0、Rust 或生产消费场景。默认入口链接 52/52、本轮改文链接 5/5、docs-only 旧路径、文档登记（26 根文件/5 哨兵）、债务结构（146 当前行/12 自动计划）、活跃编码（309 份）及未暂存 diff 均通过；归档文件另用显式 `--file` 核编码。暂存差量和目标 SHA 的远端 CI 与父提交分列。

## 保留边界

`D-DEBT-LEDGER-01` 仍为 Partial。主台账其他长行、全表自由文本状态冲突和历史证据完整迁移没有在本轮解决；本轮也不把“主表更短”当成 Event Stream 产品能力进展。
