# 路线一接续：五槽实现者入口导航（2026-10-09）

**状态**：Locally verified。基线 `01b1e48c0767fa8c74577bb43c3ca3498258c3b5`；M / 轻，controller 直接实施与语义自查，`independent=false`。本报告只记录本批范围和证据，不接管模块定义或债务状态。

## 问题与结果

维护者确认下一批持续推进。Memory 的基础入口已冻结，其余五份人类指南仍主要按 rich trait、蓝图和参考 Host 流程引导作者。已有 Base 契约、消费者、有限实现与显式 Host 接线可用；本批把这些入口接到原指南前部，避免读者误把产品设施当成基础实现前置条件。

入口分别是 [Prompt](../../../human-docs/modules/slots/prompt.md)、[LLM](../../../human-docs/modules/slots/llm.md)、[Emotion](../../../human-docs/modules/slots/emotion.md)、[Event](../../../human-docs/modules/slots/event.md)、[Agent](../../../human-docs/modules/slots/agent.md)；对应 EN 摘要同轮更新。五份 ZH 原 §1–§6 只改入口标题，EN 原丰富正文保留；作者先选基础路径或参考 Host 维护路径。

例如，作者只需对材料做 Event 分析，就能沿已有 Base→借用生成器→消费者/Host 显式入口开工，无需先改影响演化或 Ring。普通聊天仍不自动追加分析；选用现有 LLM 分析实现时须识别一次生成成本与私有协议。Agent 则区分纯计数例子与已有 ReAct 授权借用，不因补导航启动工具任务或解冻多 Agent。

## 事实归属与差量

公开定义只链接 [MODULE_MAP §3.1.1](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)及其源码锚点。五份指南分别拥有阅读路由与 checklist，没有另造 SSOT 或复制六槽定义长表。本批只核旧指南、直接实现和消费者，达到足以选择文案后停止；没有搜索所有 backend 或扩大到发行版验收。

写集十三 Markdown：十份既有 ZH/EN 指南、[原计划追加](../ROUND-02-PLAN.md#路线一--其余五槽实现者的入口导航2026-10-09)、[DCL-80 追加](../DEBT_CHANGELOG.md#dcl-20261009-80--其余五槽实现者的入口导航)、本报告。旧计划和 DCL 历史正文保持，已冻结 Memory 入口与原报告保持；原 Rust / Host / Tauri / TS / 契约 / 依赖 / 锁 / 配置 / 注册 / 测试 / 脚本 / required gate 无改动，台账与自动队列没有状态迁移。

每槽的范围与参考实现约定分开：Prompt 人设准备不重复；Emotion 线索不变成状态或真实情绪判定；LLM 借用已装配客户端，独立 generate 不隐式先调 Prompt；Event 按需且不签发主动许可；Agent 任务文本不授予权限，报告不证明效果完成或回滚。各实现的私有约定不升级为 Base 强制义务，也不规定固定六槽回合。

## 验证与收口

适用检查均 exit 0：默认链接 52 份＋十三改文显式链接、docs-only 旧路径、镜像结构、登记、债结构（147 行 / 12 计划，非债状态判定）、十三文件编码及暂存 diff。写集/保全核对六项通过：十三路径且无源码或 Memory 变更、旧计划/DCL前缀、DCL-80 唯一且旧事件保留、五份 ZH 旧丰富正文、五份 EN 旧丰富正文、各语言的既有契约/消费者/owner导航且无新增命令。原始 stdout/stderr/native exit 保存在工作树忽略目录 `.cursor/plans/five-slot-author-guides-20261009-r0/`，不随 Git 自动转让。

最终 diff 与中英含义由 controller 逐槽自查：Prompt 要求与人设准备、LLM 两种消费及资源归属、Emotion 线索与状态区分、Event 成本/按需/私有约定、Agent 案例/授权借用/效果边界一致。本批适用检查没有失败 attempt；暂存提示 LF→CRLF，不作零 stderr 声明。精确保全核对与镜像工具都不代替上述语义自查。

运行/注册/API/配置面未改，因此不重复 Rust、workspace doctest、模块配置兼容、产品场景或全量 CI。此前案例与 CI 保持原 SHA 身份；链接源码、fixture 或 rustdoc 不声称本批取得新的五槽运行证据，镜像结构 PASS 也不证明翻译正确。

适用检查与语义自查通过后本地提交；main 仍为基线且干净才安全 FF，既有授权内可备份无 PR 分支，不推 origin/main 或 dispatch 正式 CI。达到五槽作者能选择正确入口并识别限制即收口。当前没有新增需要维护者裁定的架构问题；后续实现以具体模块/Host 为对象，不为继续工作重造五个例子或穷尽调查。

既定冻结及 H04 有限验收、S01 历史 FAIL、真实音频/平台/未覆盖崩溃/独立浏览器等边界均保持；本批执行状态至多 Locally verified，不将 OPEN/Partial/Deferred 转 Done。
