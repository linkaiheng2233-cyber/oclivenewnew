# ChatPro Host 接入内核：分层验收检查单

**状态（2026-09-27）**：首轮源码核对与定向离线复跑完成；**尚未取得全 Host 合规结论**。本文是验收工作单，不新增六槽公共语义或 ChatPro 产品要求。职责与边界以 [模块注册表 §0.1–0.2](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) 为准；Base 绑定及有限实现的阶段结论见 [六槽收口](README.md#six-slot-stage-closure)。

## 验收对象先分开

1. **Base 契约**：六种可选能力的输入、正常结果和错误边界。它不规定 ChatPro 回合顺序，也不拥有角色、数据库、权限或投递。
2. **共用参考 Host**：`oclive_kernel_host` 准备调用材料、选择能力、编排回合，并负责授权、领域状态与持久化。现行产品路径仍有旧端口；Base 视图或共同核心的测试通过，不等于所有生产消费者已改用 Base trait。
3. **ChatPro 桌面适配**：`distros/desktop-tauri` 把 IPC 与流式传输映射到 Host；正常桌面启动把 `SharedKernelConnection` 和 `SharedAppState` 一起注册，聊天生产路径经 loopback HTTP。它不是另一个领域权威。`distros/chat-pro` 是前端和产品资源，不把 UI 结果当作内核执行事实。

验收顺序是 **Base → Host 对所选能力的适配 → 桌面桥 → 产品回归**。不要求一个回合调用满六槽；参考 Host 的 `process_message` 也不是所有 Host 的通用流水线。

## 当前证据与下一道门槛

| 边界 | 当前已有证据 | 仍需证明 |
|---|---|---|
| Base 可独立使用 | `base_only_fixture` 16/16；五种有限实现外测 26/26；仓库外独立 Cargo 消费者仅依赖 contracts/types 并成功调用 `MemoryBase`（见[接入学习路径](../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)） | 物理独立发布包和 Stable API 均未成立；不作为 Host 合规结论 |
| Host 选择和传递材料 | `cargo test -p oclive_kernel_host --lib cp_b3_all_ --locked --offline` 32/32；`--lib cp_b3_v1_` 6/6。测试覆盖部分 Memory 原件到 Prompt、LLM 输入、Agent/Event 的共同核心与投影 | 对每个**实际使用**的能力列明来源、用途、转换损失及调用次数；不得用补造默认领域事实来满足 Base 形状。生产链采用旧端口的地方，要验证语义适配，不能因 trait 测试转绿就宣称已迁移 |
| Host 结果和副作用 | `AppState`、`process_message`、`post_llm` 的源码显示编排/持久化归 Host；`process_message_golden_path` 1/1 使用内存 DB 和 Mock LLM，验证回复与部分 DTO 字段，**未走 Tauri 命令**；历史 H04 同回合恢复有**限定**证据 | 分开核能力正常/错误、Host 领域提交、传输送达；失败、取消或断线不自动授予重试，也不自动证明无副作用。首轮普通聊天以实际使用的 Prompt/LLM 和可选 Memory 为范围 |
| 桌面身份与转发 | `cross_transport_recovery_contract` 4/4：流/恢复共享回合身份、错误映射不变成新发送；`ChatBackend::Http` 为正常桌面分支 | 核发布构建的连接缺失异常路径及本地后端边界；真实 IPC/桌面证据沿用其各自已登记的限定范围，不由这 4 项定向桥测试代替 |
| ChatPro 发布回归 | 既有发行版 profile、捆绑和聊天回归可作为下游验证 | S01 历史语义质量 FAIL、真实模型/真实语音及未覆盖的崩溃窗口，不因前几层通过而自动转绿 |

## 第一片实施范围

先审**普通聊天**的一条实际调用链：ChatPro 输入 → 桌面 IPC/HTTP → 参考 Host 准备材料 → 已选能力 → Host 应用结果 → 权威回复。逐段记录请求身份与材料来源、是否调用能力、失败分类、生成次数、写入次数、最终 DTO 和送达观察。Memory 可选，Emotion/Event/Agent 只在真实分支使用时纳入，不为凑齐六槽强行调用。

首轮源码观察到：[`ChatBackend::from_app`](../distros/desktop-tauri/src/api/chat_backend.rs) 在缺少 `SharedKernelConnection` 时靠 `debug_assert!` 约束 `Local` 分支；正常 `.setup` 同时注册连接与状态，因此**尚未证明生产可触发该异常路径**。发布构建若需要严格单写者，须进一步验证或设计“缺连接时拒绝”的行为；该项目前标为**待评估**，不是已复现的产品故障。改动可能影响测试/其他 Host 的显式本地路径，实施前先核 12 个 `from_app` 调用点和替代路径。

本检查单的通过条件是：每个实际调用的能力满足合同；Host 不以 Base 结果自行扩权，能如实区分执行、领域应用和投递；桌面适配不改写身份、错误或写入权威；失败路径有针对性证据。局部测试全绿只把对应行改为“局部通过”，不得将整张表一次转绿。
