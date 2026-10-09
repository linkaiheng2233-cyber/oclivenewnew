# 二次代码质量与过度工程审核（2026-10-09）

**受检基线**：`89b7fa09812d53ba2cf563d919276d860456a358`，开始时本地 main 干净。本报告由 controller 作第二遍语义自查，`independent=false`；没有另一个审查者的独立结论。
**状态**：Locally verified（限定复核与报告）；当前实现保持，没有债务状态迁移或新的正式CI。
**规则来源**：[AI 修改边界](../../AI_CHANGE_BOUNDARIES.md)、[核实协议 §2.7](../../AI_VERIFICATION_PROTOCOL.md#27-代码冗余--过度工程--不简洁声称)、[多轮巡检手册](../../RECURRING_OPTIMIZATION_PLAYBOOK.md)及[仓库工程流水线](../../../.cursor/skills/oclive-dev-pipeline/SKILL.md)。本报告只保存选样、证据、判断和建议，不增加规则、必需门禁或债务状态表。
**范围**：快档基线，加维度一、三、六、七与近期模块实现者路线有关的抽查；不是全档、全仓无缺陷证明或性能评级。入选八项，八项完成有限复核，没有因预算未能分类的选样；全仓未入选总数未枚举，不报告覆盖率。

## 1. 结论与处置

受检范围内未确认需要阻断开发的新代码缺陷，也没有足够依据把现有 Base、共享消费者或借用适配判为过度工程。公共槽接口、调用方装配、共享最小适配与发行版状态仍可分辨。保留当前实现，不为降低接口数或行数重写它们。

保留一条非阻断的教学边界提醒：较早的 `minimal_role_host` 例子在自己的私有 `HostError` 中只保留错误类别；它适合展示有限装配，不适合原样复制为生产诊断模板。详见 §4。它不推翻消费者或实际 Host 已保留完整失败的证据，也不自动生成新技术债。

本轮不改运行源码、API、依赖、默认 feature、配置、注册或测试，不变更台账/自动队列状态，不启动业务进程、模型、语音或新 CI。报告、原计划/DCL 追加和巡检滚动记录构成此次文档写集。达到能决定“保留还是另开修复片”的程度即停止，不继续追其它五槽的所有后端。

## 2. 基线与证据复用

基线判定为 **PASS（复用已核原始代码证据，加本轮适用检查）**，不是当前 SHA 重新执行完整链。

原 `f94069ddd7b26e7661c81dd310175c0b6fc9e857` 上的 `npm run check:ci-local` 回执为 native 0、未超时；stdout 有 Dimension 5 **31 checks**、Host lib **637 passed**、runtime lib **273 passed**。实际 Rust lib 命令为仓库入口中的 `cargo test --workspace --lib -j 1`，包含 Host lib，但不包含 doctest。`--ci` 明确跳过 sample workspace lib，不能把它计作通过。

本轮已严格核对这份原回执、stdout/stderr 和两个 SHA 间的差量：

- `kernel/` 与 `distros/` 只在 contracts 的 `slot_base.rs` 有差量，新增/删除行均为注释；其它 Rust 运行体、Host/runtime/产品源码相同。
- 所有 Cargo manifests、根 Cargo.lock 和 package-lock 相同；既有 npm 命令与依赖相同。新 `check:abstraction` 脚本/入口以及 `LAYERING_BASELINE.json` 中的观察配置单列，本轮实际重跑观察器。
- 原分层与依赖重复组硬基线字段相同，未因复核放宽。
- 目标 `89b7fa09` 上十三份上一片文档与其冻结原件的 Git blob、规范化文本相同；原件本身的 bytes/hash 全部吻合。跨工作树的 LF/CRLF 差异分别登记，不称为原始磁盘字节全同。

原完整本地链与正式 CI 仍只属于 `f94069dd`；本轮没有为 `89b7fa09` 或报告提交取得新的全量/远端 CI。旧 doctest 的首次失败与后续通过原件没有被改写，本轮不以 `--lib` 补出 doctest 结论；此次公开签名与 doctest 示例均未改变，不再重复 workspace doctest。

本轮直接执行：

| 命令 / 核对 | native / 结果 | 能证明的范围 |
|---|---|---|
| `node scripts/check-domain-layering.mjs` | 0；use-import 3 / 上限3，FQ 1 / 上限1 | 既有分层棘轮未回退，不代表全仓全部架构正确 |
| `node scripts/check-stale-paths.mjs` | 0；docs + code | 工具覆盖的旧路径、别名、archive truth 检查 |
| `node scripts/check-abstraction-ratchet.mjs --self-test` | 0 | 原观察器同实现自测，不是独立代码审查 |
| 同脚本 `--json` | 0；1215 源文件、43 trait、23 单实现文本命中、最大2557物理行、105脚本 | `OBSERVATION_ONLY`，不是23笔债或2557行文件应拆的判据 |
| `cargo test --locked --offline -p oclive_kernel_runtime --example external_memory_base` | 0；3 passed / 0 failed | 当前受检 SHA 的真实消费者调用、选取差异、正常空结果和完整 Unsupported；只用内存实现 |
| 本轮忽略目录中的基线复用核对 | 首次1；原因修正后0 | 原件身份、编译/依赖差量和十三文档 Git 内容；第一次失败详见 §6 |

原始证据在本机 `E:/OCLive/_worktrees/second-review-20261009-r0/.cursor/plans/second-review-20261009-r0/`，每项 stdout、stderr、命令、cwd、HEAD、native exit、deadline 分件保存；不随 Git 自动转让。原完整链保留在 `external-memory-base-20261009-r0` 工作树的对应 ignored 目录，本轮未重写。Cargo 编译进度及 Git 换行提示不称为零 stderr。

## 3. 八项有限代码复核

按当前主线选样，不按文件大小排名。默认只读定义、直接消费者和一条对应测试；Memory 的选取/Host fallback、Host 的错误/授权边界进入有限 D2。未开 D3、反例穷尽或全仓逐实现调查。整批上限120分钟，D1每项至多15分钟，具体疑点的D2额外至多20分钟且最多两个关联边界；这是代码质量专题，不冒充完整 Document → Code 专项。

| 选样 | 实现与直接消费 / 证据 | 判断与停止线 |
|---|---|---|
| 六槽公共 Base 的职责 | [slot_base.rs](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs):116 起；六个单方法 trait/local future，[独立 fixture](../../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs)核 dyn、借用、空结果与已知失败。原完整链包含该 fixture | 保留公共边界。它服务树外实现者，不要求第二个生产实现才准存在；不从 local future 推导跨线程、远端取消或零分配，STOP |
| 共享消费者的层次与转发 | [minimal_role_consumer.rs](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs):69、98、131、213、264；[六槽回归](../../../kernel/crates/oclive_kernel_runtime/tests/minimal_role_six_slots.rs)的独立调用、Pending/借用、完整失败测试在原日志实际通过 | 六个转发履行六个不同能力，不是六套调度器。Prompt 准备、可选 Prompt→LLM 操作和六槽独立调用服务不同调用形状；共享准备没有复制角色加载/Host状态，STOP |
| Memory 看似相同的筛选器 | [KeywordMemoryBase](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_memory.rs):93；[QueryMemoryRetrieval](../../../kernel/crates/oclive_kernel_runtime/src/domain/query_memory.rs):80、111、175；[作者实现](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs)。Host实际在 [process_message.rs](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs):94–117 生产区调用 Query 的 Base view | 整串子串、分词/非ASCII bigram，以及精确大小写/最新N是不同语义。Query 的 rich 排序fallback与Base空命中也不同，不能统一成“失败就回全部”。本轮案例3测区分近两条和关键词三条，STOP |
| Host 最小文本和 Event 借用 | [minimal_llm.rs](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/minimal_llm.rs)、[minimal_event.rs](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/minimal_event.rs)，共同复用 [generate_minimal_text](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs):270；Event再借用 [LlmEventAnalyzer](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_event.rs):247 | 薄适配各保留槽请求/结果语义，不重复创建provider/runtime；Event显式按需一次生成，无格式自动重试。Host 公共API的Prompt失败停在模型前且完整保留的测试在原日志通过，STOP |
| Agent 借用与权力边界 | [task_execution_base](../../../kernel/crates/oclive_kernel_host/src/domain/agent.rs):170 → [agent_base_binding.rs](../../../kernel/crates/oclive_kernel_host/src/domain/agent_base_binding.rs):241 → 同一 `execute_react`。原日志中非空context零资源调用、扣留discovery grant保留Unavailable均通过 | 绑定当前身份/资源，不用默认丰富角色伪造上下文；task文本不授予新权限。报告不是目标终态/效果回滚证明。没有理由为统一错误映射而抹掉各适配的实际来源，STOP |
| 外部 Memory 案例与私有 Host 例子 | [external_memory_base](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs):72 通过已有消费者；[minimal_role_host](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs):174 的私有 HostContent 分别适配 LocalMinimalRoleSnapshot 与 MemoryContent，main 实际演示两种来源 | 前者没有注册/发现新框架；后者抽象已有两种实际来源，而非虚构未来需求。立即轮询器不是通用async executor；错误诊断限制单列 §4，STOP |
| 人类入口与AI边界 | [Memory指南](../../../human-docs/modules/slots/memory.md)区分Base案例/rich路径；[Prompt](../../../human-docs/modules/slots/prompt.md)、[LLM](../../../human-docs/modules/slots/llm.md)、[Agent](../../../human-docs/modules/slots/agent.md)等入口只导航到唯一契约/实现。新AI规则不要求为数字拆代码 | 当前路径不再要求基础模块作者先改ChatPro/STM/LTM；保留接入限制。没有把文案更新写成五槽新增实况。结构检查与本轮语义读查分开，STOP |
| 抽象观察器本身 | [check-abstraction-ratchet.mjs](../../../scripts/check-abstraction-ratchet.mjs)收集Git跟踪文本，输入/读取失败非零，输出方法/位置/局限；没有新CI required gate | 有限观察工具，不是架构裁判或第二套规则。单实现、源码/测试共计行数和未解析cfg/macro的统计不能当质量结论；真实JSON和同实现自测均通过，STOP |

这些判断只支持所选接入路径和职责，不证明所有 backend、所有 Host 或所有模块实现合规。

## 4. 一个非阻断的教学提醒

**位置**：[minimal_role_host.rs](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs):46、57、313。私有 `HostError::Capability(BaseCallErrorKind)` 只存kind；`poll_immediate` 投影时没有保留 `BaseCallError.detail`。文本示例分支也把 Prompt/LLM phase 转为该窄错误形状。源注释已经说明这是例子既有的窄 HostError，不能据此说共享消费者丢了完整错误。

**影响**：近期 [Prompt指南](../../../human-docs/modules/slots/prompt.md):20–24、[LLM指南](../../../human-docs/modules/slots/llm.md):20–24 和 [Event指南](../../../human-docs/modules/slots/event.md):20–24 同时要求保留完整失败并导航到这份装配例子。开发者若直接复制例子的私有错误层，可能只能得到类别，失去具体诊断。实际参考 Host 的 `selected_prompt_errors_stop_before_model_and_preserve_complete_error` 已在原完整链通过；当前外部 Memory 例子也实际核完整 Unsupported。所以它是教学投影的限制，未证实为产品故障。

**局部等价方案**：下次确有作者使用该例子时，在相关指南的例子链接旁明确“该例私有 HostError只演示原因类别；生产Host按自身诊断需要保留原始失败/phase”。此文案方案不改变任何行为/API；若确实需要改例子的错误形状，则另选私有例子切片并跑其定向回归，而不是统一所有错误或新建通用执行框架。

**本轮处置**：记录提醒，保持例子与指南原件；无实际受阻作者时不把低收益整理变成新实现任务，不登记OPEN/P1或改旧债Done。

## 5. 为什么没有做“简化重构”

- [Prompt材料连接](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_prompt.rs):97 与 [最小人设准备](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs):264 已各有共享核心。不同入口仍承担不同请求/错误与配置语义，继续抽公共父类没有本轮收益依据。
- 三种 Memory 的输出规则不同；机械合并会改变查询、顺序、限额或Host fallback。保留不同实现比用一个开关密集的万能selector更容易说明承诺。
- 公共 Base 的 dyn/local future 与 Host 私有适配承担实际公共/资源边界；观察到单实现不能作为删除接口的依据。六个转发方法直接接收已有请求，没有新增权限/发现/调度服务。
- `minimal_role_host` 的两个内容来源和 Memory 案例的有限立即实现已有实际用途。它们仍是案例，不提炼为所有发行版的框架，也不扩大到六槽固定执行。
- 2557物理行的最大文件由观察器定位，但本轮主线没有对应维护障碍或改动任务；不为降数字进入剧场大拆分，也不把未查文件判为质量优。

这些是本轮保留设计的理由，不是永久禁止改动。具体新消费者、重复职责或故障反例出现时，再开有限切片。

## 6. 审查工具自纠与原件

第一次基线复用核对 native 1：脚本把新checkout的磁盘bytes直接与旧文档冻结件比较，`WAVE-20261009-FIVE-SLOT-AUTHOR-GUIDES.md` 实测4804B，旧原件4773B。原因是31个LF被checkout为CRLF，而非语义漂移。

原脚本与 `06-baseline-reuse.*` 保留；修正写成另一份本轮忽略脚本，再执行 `07-baseline-reuse-corrected.*`，native 0。它先核原件本身bytes/hash，然后核两个工作树的规范化文本、Git blob和目标提交blob，并分别记录双方磁盘身份；没有改源码、回写原hash或降低断言。脚本同作者实现，只用于本轮取证，不加入工程门禁。

一次只读文件名输入误写为 `minimal_role_text.rs`，读取非零；用限定目录的 `rg --files` 定位实际 `minimal_role_public_api.rs` 后读其定义和原运行记录。未把未命中的旧名称写成“没有测试/实现”。这也说明新核实协议限制否定结论的用途。

## 7. 出口、未测项与下一步

本轮只形成复核快照和现有日志中的一次实际巡检记录，适用文档链接/旧路径/登记/债结构/编码/diff检查随文档写集执行，原计划与DCL历史前缀保留。巡检评分 **未评分**。没有新的维护者架构问题，需要用户选项的冻结事项均未触发。

四份文档写集的实际检查均 native 0：默认52份及本批4份显式链接、镜像、登记（26 handoff root / 5 sentinels）、债结构（147台账行 / 12 auto plans，非状态评估）、四文件编码和暂存diff。七项写集/历史/手册边界保全核对通过；手册只滚动§8的五行记录，§0–§7与§9规则未改。该报告状态不等于全仓质量、独立审核或新SHA全量CI已验。

没有验证全仓质量、真实模型语义、真实TTS/音频、硬件平台、全部崩溃窗口、独立浏览器发行形态或性能矩阵；H04的有限验收、S01历史FAIL等仍按原范围管理。Full韧性、双核、多Agent、签名/TLS、高级情绪记忆、Production Stream、统一远程wire、core物理迁移、rich接口整撤和CI分层没有解冻。

下一步可以直接以当前 Memory 案例与六槽作者导航作为可接手入口，遇到具体模块/Host的实际接入需求再选一个槽施工。暂无指定对象时保持这片基线，不自动再写五套机械例子或以“证据还可增加”为由继续审查。
