# ChatPro Host 接入内核：分层验收检查单

**状态（2026-09-27）**：普通非流式聊天的 **Memory → Prompt → LLM 所选接入切片已有本地限定证据**；桌面同身份恢复另有其自身的限定验收。本文整理已有工作并规定下一片的止点，**不以全 Host 合规为目标**，也不新增六槽公共语义或 ChatPro 产品要求。职责与边界以 [模块注册表 §0.1–0.2](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) 为准；Base 绑定及有限实现的阶段结论见 [六槽收口](README.md#six-slot-stage-closure)。

## 验收对象先分开

1. **Base 契约**：六种可选能力的输入、正常结果和错误边界。它不规定 ChatPro 回合顺序，也不拥有角色、数据库、权限或投递。
2. **共用参考 Host**：`oclive_kernel_host` 准备调用材料、选择能力、编排回合，并负责授权、领域状态与持久化。现行产品路径仍有旧端口；Base 视图或共同核心的测试通过，不等于所有生产消费者已改用 Base trait。
3. **ChatPro 桌面适配**：`distros/desktop-tauri` 把 IPC 与流式传输映射到 Host；正常桌面启动把 `SharedKernelConnection` 和 `SharedAppState` 一起注册，聊天生产路径经 loopback HTTP。它不是另一个领域权威。`distros/chat-pro` 是前端和产品资源，不把 UI 结果当作内核执行事实。

验收顺序是 **Base → Host 对所选能力的适配 → 桌面桥 → 产品回归**。每片只核其实际消费的能力及必需依赖；不要求一个回合调用满六槽，也不把 `process_message` 变成所有 Host 的通用流水线。

## 已有证据按接入切片归档

| 切片 | 已有证据及精确范围 | 本片处理 |
|---|---|---|
| Base 独立调用 | `base_only_fixture` 16/16、五种有限实现外测 26/26；仓库外 Cargo 消费者只声明 contracts/types 并成功调用 `MemoryBase`（[接入学习路径](../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)） | **已有本地证据；不重复要求 ChatPro 证明内核本身**。独立发布包和 Stable API 仍未成立 |
| 普通非流式 Host：Memory → Prompt → LLM | `cp_b3_v1_` 6/6 验相邻消费点；`cp_b3_all_` 32/32 验选定共同核心与投影；`process_message_golden_path` 1/1 验内存 DB＋Mock LLM 回复/部分 DTO。历史 **M-V1-R2 `MEMORY_HOST_PASS` 已由主控限定接受**：两条隔离 Host 回合分别验证命中/无命中、Prompt 原文、一次记录型生成、受控记忆写回及两消息绑定。其冻结清单中的 `memory_turn.rs`、`long_term_memory.rs`、`slot_runner.rs` 与当前磁盘 SHA256 三项全同 | **所选路径限定成立，不重跑已消耗身份**。M-V1 不证明 `MemoryBase` 在产品链被直接调用，也不证明真实模型、流式或全部角色配置；Host 旧端口的语义适配与 Base trait 直接接线分开表述 |
| 用户 Emotion → Host Prompt/结果 | 普通用户输入在 `pre.rs::resolve_user_emotion_for_turn` 调用 `SlotRunner::analyze_emotion`，随后生成七维结果和 Prompt 语气线索。历史 **CP-INT B1-R1 `CP_INT_B1_PASS` 已获主控限定验收**：四条隔离非流式 Host 回合覆盖愉快、无词表线索、否定和中性线索；实际 Prompt、DTO、用户行 metadata、一次记录型生成和隔离 DB 的两消息绑定均被核对。当前 `emotion_turn.rs` 与 R1 冻结哈希相同，`pre.rs`、`run_middle.rs`、`BuiltinUserEmotionAnalyzer` 相对受测 HEAD 无差异 | **作为已选用能力的独立限定切片沿用，不重跑四个已消耗身份**。`joy` 是七维数值键，`happy` 是 Host 标签；不宣称 Base 报告已由产品直接消费、真实情绪质量或其他后端通过 |
| 桌面同身份传输/恢复 | 本轮 `cross_transport_recovery_contract` 4/4；历史 R2-G／R2-CANCEL-C 已获限定产品验收（本机记录 `.cursor/plans/cp-int-b9-b11-r2.controller-review.md`），只覆盖默认令牌配置下已测桌面普通聊天及取消路径 | **独立的下游限定证据**，不作为 Memory/Prompt/LLM 的新内核要求；不重跑旧业务身份 |
| 其他可选能力和发行版 | Agent/Event 的 Base 视图及消费点有各自定向证据；发行版 profile、捆绑和聊天 smoke 可作下游回归 | **不进入以上两片的封闭条件**。只有下一片实际消费相关能力时才取相应证据；S01 历史语义质量 FAIL、真实模型/真实语音、未覆盖崩溃窗口各自留在原范围 |

## 本片止点与下一片选择

普通非流式聊天的所选 Host 延伸切片止于：**已知材料的来源与选择 → 实际 Prompt → 一次受控生成 → Host 应用结果**。以上证据来自不同层级，M-V1 是合成角色、隔离 SQLite、记录型 LLM 的真实 Host 回合；V1 单元测试是手动组合；黄金路径没有走 Tauri 命令。它们可共同支持**限定接入判断**，不能拼成“同一次运行覆盖所有层”或“全 Host 合规”。当前只继续做源码漂移关联和证据索引，不为补一个总计数再发已消耗场景。

M-V1 的主控限定验收、两条已消耗身份及原始树见本机 `.cursor/plans/chatpro-verification-memory-turn-v1-run-only.execution.md` 的 R2-E3/R2-E4；Emotion 四条回合的执行记录与主控限定验收分别见本机 `.cursor/plans/chatpro-integration-stage-batch-v1-r1.execution.md` 和 `.cursor/plans/chatpro-integration-stage-batch-v1-r1.controller-review.md`；R2-G 的裁定见上表所列本机记录。这些目录不在 Git 跟踪范围内，**本页只保存结论边界与当前源码对照，不代替跨机器可携带的原始证据**。

**M-V1 与当前源码的关联复核**：其 R2 冻结清单中 `memory_turn.rs`、`long_term_memory.rs`、`slot_runner.rs` 三项对当前磁盘逐项 SHA256 相同；`pre.rs` 相对当时 HEAD 无改动。此后 `process_message.rs` 增加了带 `client_request_id` 的收据分支，M-V1 请求仍为 `None`；`post.rs` 的新增行为集中在流式 fallback，M-V1 是非流式。这里是变更关联审查，**不是在当前 HEAD 重新发起两条回合**。

**Emotion 与当前源码的关联复核**：`emotion_turn.rs` 当前 SHA256 为 `02ED58E1AF35C692C5DAD80BB5912B29FCA7A69CA3BCCBDB2F4C2C50A9E59E05`，与 B1-R1 冻结值相同；`pre.rs`、`run_middle.rs` 和 runtime 的 `user_emotion_analyzer.rs` 相对受测 HEAD `4b7ccc66` 无差异。该回合也只覆盖当时的合成角色、builtin、记录型 LLM 与隔离 SQLite；这是证据可沿用性审查，**不是当前 HEAD 的新回合或真实情绪分析质量测评**。

下一片只在**真实选用能力或产品接入边界出现明确缺口**时开启，并写出一个可观察的通过条件及最小测试面。优先使用已有隔离夹具；实际模型质量、真实音频、独立浏览器发行形态及所有崩溃窗口不因本片成为 Host 门槛。ChatPro 发布回归继续下游独立管理。

### 后续按变更触发回归

| 变更面 | 最小相关回归；扩大条件 |
|---|---|
| 六槽 DTO/trait/共同错误 | `base_only_fixture`；只对受影响的具体实现再跑对应 `base_*` 外测。公共边界变化须先复核 [模块注册表](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)，不能由 ChatPro 用例倒推新增要求 |
| 参考 Host 的 Memory/Prompt/LLM 选择或交接 | `cp_b3_v1_`、受影响的 `cp_b3_all_` 过滤项与 `process_message_golden_path`；LLM 完成语义变化时加 `cp_b3_c1_`。仅有明确未覆盖的产品行为变化，才设计**新身份**的隔离回合；M-V1 的旧身份不复用 |
| Host 用户 Emotion 的分析、七维投影或 Prompt 语气交接 | `cp_b3_c2_`、`cp_b3_v1_emotion_consumer_returns_builtin_seven_dimensions` 与相关 `prompt_builder` 测试；只有实际交接语义变化，才考虑新的隔离回合，E01–E04 旧身份不复用 |
| 桌面 IPC/HTTP 身份或流式桥 | `cross_transport_recovery_contract` 和受影响的前端定向测试；只有这层的实际转发语义变化，才决定是否增加受管 loopback/桌面回合，不把其作为每次内核改动的常规门槛 |
| 发行版 profile、捆绑或发布件 | 对应 profile 解析与 `e2e-distro-kernel` smoke；真实安装包/模型/语音另按发布目标单独验，不回写 Base 合格结论 |
| 仅文档和证据索引 | 链接、镜像、登记与 `git diff --check`；不重跑业务身份或模型样本 |

表内命令是相关性入口，不是所有提交的统一 CI 套餐；现有测试的具体范围和结果以上表已注明的边界为准。

**另记而不抢主线**：[`ChatBackend::from_app`](../distros/desktop-tauri/src/api/chat_backend.rs) 在缺 `SharedKernelConnection` 时只靠 `debug_assert!` 约束 `Local` 分支；正常 `.setup` 同时注册连接与状态，尚无生产可触发该异常路径的证据。已数到 12 个调用点。它是发布异常路径的待评估项，**不是本片失败或必须先修的架构门槛**；若以后要改为发布构建拒绝本地后端，先核测试/其他 Host 的显式本地用途，再裁定改法。
