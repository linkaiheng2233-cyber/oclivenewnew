# 近期 SDK、接线视图与参考 Host CI · 有限质量审查

**SSOT范围**：本轮选样、语义判断与证据限制；不新增规则、模块定义或技术债状态。
**日期 / 状态**：2026-10-13；**Locally verified（有限审查）**。审查与适用报告门禁通过，不构成新SHA的完整CI或独立复核。
**受检**：干净 `2d36bb017a4dd81bdd5608d31cad0c5120307978`；区间 `f4f0ecdb^..2d36bb01`；controller第二遍实施/语义自查，**independent=false**。

## 方法与基线

按[巡检手册](../../RECURRING_OPTIMIZATION_PLAYBOOK.md)快档基线及受影响架构、设计/易读性、文档和条理检查，执行[本轮计划](../ROUND-02-PLAN.md#近期-sdk接线与-host-ci-质量审查2026-10-13)。最多八选样或120分钟；已查八项，以D1主路径为主，输入错误与CI可信控制面作有限D2；没有自动D3、全仓负面证明或全档评分。选样覆盖当前开发动作，不能将八项解释成全仓覆盖率。

规则以[AI修改边界](../../AI_CHANGE_BOUNDARIES.md)、[核实协议](../../AI_VERIFICATION_PROTOCOL.md)和[工程门禁](../AI_AND_PIPELINE_GATES.md)为准。维护者的GPT局限补充从本机 `E:/OCLive/_recovery/AI-GUARDRAILS-20261009-R0/DeepSeek-review-original.md` 读取，hash与登记 `422A32A4C4CF95BB648FABB106F9882EAD644083589C8D6ACC4F1109A677895D` 相符。它只提示限定/跨行搜索、文档漂移、共享脚本、冻结代码与单实现接口等误判；其中旧数量硬门禁没有采纳，后续规则取舍见[AI限制Wave](WAVE-20261009-AI-GUARDRAILS.md)。没有把侧对话的判断或回滚命令当执行指令。

本轮基线为**PASS（限定复用与当前定向检查）**：

- B1重新只读查询[正式CI 38199557652](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38199557652)：目标 `08348f31b8f3459537c13227302bf4b8e273baa8`，21/21 job及ci-gate success。B2b核Host/runtime/contracts/types/validation、根manifest/锁、package、scripts、workflow与catalog对象不变。原Dimension5与Host默认检查因此只在这组不变输入的范围复用；不声称新CLI或本报告SHA跑过完整CI。
- 接线七份Rust/模板/测试与原G18摘要相同；原G4为100 bin＋8接线＋7基础E2E，原G3 fmt/G5 Clippy均exit0，本轮未重跑。SDK两份原源与种子锁一致，原P1–P10退出码全0、P9 binary三测与P10真实消费者输出可查；保留空library/doctest零项不作功能证据。
- B3本轮分层exit0（use-import 3/FQ 1，未上涨）；B4本轮docs+code旧路径exit0。AI入口引用现行边界、核实协议和手册；不把补充文档或旧Wave改成新的规则SSOT。

本机原始记录在 `.cursor/plans/recent-quality-review-20261013-r0/`，连同SDK `_recovery` 和原接线证据并不随Git自动转让；源码测试与操作配方可以复现。正式CI步骤中的默认skip不被写成全平台实机、真实模型或音频通过。

## 八项审查结果

| 项 | 精确入口与核对 | 结论与停止线 |
|---|---|---|
| 1 责任边界 | [MODULE_MAP三层](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)、[SDK RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)、[CLI声明读取](../../../kernel/crates/oclive-cli/src/doctor_wiring.rs) `:98/190` | CLI查看、共享消费者、Host执行分别归属；未改Base/公共类型、调度或资源权限。当前范围Supported，停止；不凭crate目录名重定义小Kernel。 |
| 2 SDK工程配方 | [SDK指南§2–4](../../../examples/developer-sdk-memory/README.md)、基础Cargo模板、原P5/P6/P8/P9/P10 | 完整源码checkout→四文件基础→显式runtime→作者独立workspace/派生锁；无Host/Tauri依赖。文档承认缓存/链接工具、path移动和锁不固定Git内容的前提，未伪装注册表发布。Supported；干净新机器/其它平台未实跑。 |
| 3 Memory实际接入与可读性 | [作者Memory实现](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs) `:32`、[调用者](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs) `:53/72`、[消费者](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs) `:131`，原三测/运行输出 | 作者实现经共享消费者调用，选择算法小而直接，返回原文与完整Unsupported；Cell计数在当前线程亲和Base内合法，不因不Send而改接口。即时helper的Pending失败和调用方执行器要求已写清，不是通用executor或隐含调度。Supported；不扩检索质量/异步provider。 |
| 4 声明输入与失败 | `doctor_wiring.rs:18–84/98–127`；[8项真实CLI测试](../../../kernel/crates/oclive-cli/tests/doctor_wiring.rs)的错误/去敏路径 | 小型私有Serde类型核版本、标签、槽键和重复范围；沿既有GROUP_SLOT_TYPES，Base名称match只做显示，不是第二套蓝图/后端parser。TOML及字段错误不回显输入、完整校验后才输出；未读取Host秘密。Supported，仅核明确输入边界，不承诺检测任意字段中的敏感含义。 |
| 5 接线输出语义 | `doctor_wiring.rs:128–187`，CLI指南[状态说明](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring)和重读测试 | Base与实现分开，来源为author declaration，绑定/调用/功能未核实；call_scope不是调用记录，连线不是执行顺序。修改声明会更新快照，改Rust不会自动同步。Supported；不外推真实绑定、热替换或Host报告。 |
| 6 CLI、模板与维护负担 | [doctor分派](../../../kernel/crates/oclive-cli/src/doctor_cmd.rs) `:112`，空元数据模板、原8＋7回归 | 复用doctor域，查看不落入环境/修复/监控/SBOM路径；空模板不登记教学模块。私有校验/渲染函数职责可定位，测试走实际CLI、无工具PATH及禁止构建目标工程，未新建通用观察框架。没有确认收益足够的重复/死代码，不因实现/测试文件长度强拆。 |
| 7 默认Host/CI工程 | [默认Host入口](../../../scripts/check-reference-host.mjs) `:8/13`、同文件回归、workflow的reference-host-default/rust/ci-gate、catalog及trusted探针回归 | 两固定命令复用native helper，保留失败/超时；default与旧workspace桌面feature是不同条件，增加所有者并非同条件重复验收。trusted base缺/坏拓扑走full，原gate及桌面覆盖保持。Supported；remaining工作组仍mixed，未宣称CI全部拆净、选择性权力扩大或提速比例。 |
| 8 文档与开发者阅读顺序 | [SDK主指南](../../../examples/developer-sdk-memory/README.md)、生成project README、CLI中英与[冻结报告](WAVE-20261013-SDK-WIRING-CLOSEOUT.md) | “改哪里→跑什么→看什么”位于主路径；可选视图在后，CLI指南唯一维护字段格式；生成README链接完整说明。示例复制后归作者工程、教学/契约/接入/目标功能区分，当前证据与历史状态未混写。Supported；双语及保全说明较长，先收真实上手反馈，不为缩行数删必要前提。 |

入选/已查/未知为8/8/0，仅指以上限定问题；未入选的其它模块/产品路径不计入分母，也不声称逐条枚举。分项未独立计时，不编造耗时；私有计划、查询和退出记录保留整批执行时序，预算未扩张。

## 发现、误报剔除与处置

**本范围没有确认新增阻断或需要立即实施的架构/契约修复。** 没有把“未测”写成缺陷，也不以没有新债证明全仓无债。本轮未改产品代码、模板、公共面、锁、CI或技术债状态；普通文档/工程表现足以支撑已确认第一棒。

| 候选判断 | 本轮判读 |
|---|---|
| 单实现/小包装应删 | 公共树外Base与当前consumer有明确职责；CLI为实际声明输入建私有类型，不增加公共抽象，不要求第二实现者。 |
| 六槽都显示/绑定，应每轮全跑 | 六个接口可见与具体调用范围不同；Memory计数和零生成、文本标签均有直接证据。 |
| 视图未知意味着没有实现 | 未声明、未连接报告才是当前视图事实；实际实现另从源码与消费者核，不做否定推断。 |
| 两次Host检查浪费 | 默认Host与合并桌面feature的配置不同，剩余覆盖保持；没有可删除检查的证据。 |
| 种子锁应与派生锁逐字相同 | **我的B2私有核对器首轮误判**，exit1原样保留；先停查原因，保全脚本原件，再改为核Cargo.lock.seed，B2b exit0。源码/作者工程不动，没有通过重跑业务或弱化产品测试洗绿。 |

复用源/依赖范围与语义已足够行动，本轮不再跑抽象总量来“证明没有过度设计”，也不重建完整CI、真实业务或其余五槽案例。

## 工程纪律、未测项与后续

G1–G8/G17：未改角色/蓝图/契约、运行编排、权限、错误码、依赖或消费者行为；无需新增doctest/模型/IPC或全量业务。G9–G16：未强拆或引入规则体系，跨主题链接既有SSOT；仅PLAN/DCL追加与本报告。报告适用文档/编码/精确写集/历史前缀及旧源码摘要核验收尾，不把结构检查叫行为通过。

本轮V1–V7均exit0：默认及三文链接、登记、docs-only旧路径、债结构、严格UTF-8/无BOM/无乱码、精确三文写集/两历史前缀、四份旧冻结上手文与七源码/模板/测试摘要保全、main与SDK工作树干净及diff。最终正文更新后仅复查受影响链接、最终范围与暂存diff，不重跑不变的Rust/业务。首次B2核对失败与修正B2b独立留存，没有删除或覆盖失败日志。

仍需按真实目标而非本报告去证明：新机器/其它平台构建、作者模块目标功能、真实Host绑定/调用报告、发布组合、真实模型/语音/性能及既有未覆盖崩溃窗口。S01和既定冻结不变。这些不是本轮新增阻断；首次出现具体反馈再定场景和验证预算。

旧 `checkpoint/source-sdk-wiring-20261013-r0` 保持指向 `2d36bb01`，当前main `08348f31`和独立SDK `9c0ce96f`保持；本轮不合main/push，不移动标签。通过文档门禁后可提交本地审查报告；最终SHA和clean状态在交付记录，不为回写CI绿灯造第二提交。

下一步可沿既有有限成果准备正式集成，届时目标SHA的适用完整链/CI另判；或接受具体使用反馈。当前没有新增需维护者决策的问题，不自动开发SDK门面、注册系统或运行追踪。
