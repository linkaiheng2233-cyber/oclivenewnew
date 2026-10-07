# DUAL-CORE-FREEZE · 双核 Beta 维护冻结

runner=skip · dual_core / expert_routing **默认关**。禁止隔夜默认打开。Stage0：确认仍关 → skip。

## 冻结决定与接手入口

2026-10-07，维护者选择暂停双核 Beta 的继续维护，保留现有实现、已知缺口和解冻方向，继续其他技术债。当前状态唯一见[主台账 K-DUAL-ROLLBACK-02 与 §2 冻结表](../../TECHNICAL_DEBT_INVENTORY.md)；本页拥有冻结范围和后续准入规则，变动事件见 [DEBT_CHANGELOG](../DEBT_CHANGELOG.md#dcl-20261007-57--双核-beta-维护冻结与已验-remote-基线接续)。

冻结代码参考点为 `90b9cf4d3f9a9a17744323d1177f138c90c686ea`。它的完整本地链 native 0、[正式 CI 37607847085](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37607847085) attempt 1 / 17 jobs success，证明该提交的既有门禁通过；**不证明下述 NULL 缺口已修复**。冻结决定本身是文档与维护范围变动，不是新增双核运行验收。

## 保留的边界

- 保留 v3 opt-in Beta 的 P2–P5 已实现历史、源码、显式配置入口和已有测试；暂停主动开发、扩面及自动执行实验验证，不把已有 Beta 改称 Stable 或生产保证。
- `dual_core` / `expert_routing` 保持默认关闭。本次不改任何 feature 或配置，也不为未来 AI 预先授予启用权限。已有显式 opt-in 入口没有被删除。
- 默认 Stable 仍走 `process_co_present`；普通新角色包使用 v4，v2 保留兼容边界。Stable / 共享六槽 / 最小角色运行不等待 Beta 的全部债务解决，小 Kernel 边界不变。
- `expert_routing` 在本页仅按与双核实验配套的范围冻结；不据此停止其他正常槽位实现、资源绑定或 Remote 整理。

实现与契约直接查[双核交接](../../DUAL_CORE_CURSOR_HANDOFF.md)、[权威 RFC](../../../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)和[开发者指南](../../../creator-docs/dual-core/DEVELOPER_GUIDE.md)，这里不复制方法/阶段长表。

## 未解决事项与方向

K-DUAL-ROLLBACK-02 仍未解决：`TurnRollbackSnapshot` 对 narrative hint、emotion、presence scene 做有限补偿；既有 `Some` 值可写回，但原 emotion / scene 是 `None` 时，实验后来写入的值不会被清空。例子：原 emotion 为 NULL，实验步骤写入值，后续失败降级到 Stable；当前补偿不能保证 Stable 前恢复 NULL。这不是通用数据库事务。

主要代码锚点：

- [dual_pipeline.rs](../../../kernel/crates/oclive_kernel_host/src/domain/dual_pipeline.rs)：快照、补偿与既有回归。
- [emotion_scene.rs](../../../kernel/crates/oclive_kernel_host/src/infrastructure/db/role_runtime/emotion_scene.rs)：当前 emotion / scene 的读取与写入接口。
- [process_message.rs](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)：Host 双核门控与 Stable 入口。

未来有两条既有方向：增加显式 nullable restore/clear；或收窄实验步骤，在成功终止前禁止写 DB。**本次不选择、不实现任一方向。** 若采用 nullable restore，需先区分“读取失败”和“确实缺值”，不能将读取错误当作 NULL 去清空；首片只覆盖这三个字段与 Stable 前的补偿边界，不借机建立泛化事务框架。

## 解冻条件与调查停止线

1. 维护者明确决定重新维护 Beta，并选择剩余范围及上述处理方向；普通“继续其他债务”不等于解冻。
2. 在当时已验的干净基线上建立新的有限计划与写集；旧快照只作为历史，不要求其它债务停留在本代码提交。
3. 首片以“原 NULL → 实验写入 → 后续失败 → Stable 前恢复预期”为可判定闭环，补对应失败/非缺值边界；已有 Some 恢复和默认关闭路径保持。测试与模型/网络预算在执行前明确，不复用历史业务身份。
4. 获得足够施工与验收证据即停止深入；不自动开启全部 DAG / 并发 / 事务 / 崩溃窗口矩阵。新的公开语义或更深调查交维护者决策。

冻结期间仅允许记录已发生问题、修正文档事实及继续其他已授权债务。新发现若影响默认 Stable 的数据安全，应独立按其真实影响处理，不能用 Beta 冻结掩盖。队列保持 `runner=skip`；Frozen / Deferred 不等于 Done，也没有恢复普通无人值守领取资格。
