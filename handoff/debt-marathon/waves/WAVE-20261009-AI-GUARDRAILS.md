# AI 限制优化与有限复核（2026-10-09）

**状态**：Locally verified，适用本地验证通过。起点 `894047c52780375c9c01d596e442ff59231c596d`；本批实现、规划和总审均由 controller 完成，independent=false。现行约束以 [AI_CHANGE_BOUNDARIES](../../AI_CHANGE_BOUNDARIES.md)、[核实协议](../../AI_VERIFICATION_PROTOCOL.md)与[工程纪律 §8](../AI_AND_PIPELINE_GATES.md#8-大文件拆分红线2026-08-13--轮次-2913-个-rustts-巨文件拆分) 为准，本报告不另建规则或债务状态表。

## 原因与取舍

维护者提供桌面《OCLive-gpt6.1复核缺陷与限制优化-20261009.md》，SHA256 `422A32A4C4CF95BB648FABB106F9882EAD644083589C8D6ACC4F1109A677895D`。该材料是建议和候选事实，未提供被评审模型的完整原报告，本批不据此评价模型整体质量，也不执行文内回滚指令。main 上四项未提交候选已逐字节保全；先隔离施工，再复核真实重叠，模块开发工作树不碰。

| 已定位问题 | 实际修正与例子 | 限度 |
|---|---|---|
| 限定路径/泛型/跨行搜索漏检 | 核实协议要求读定义与直接消费者；观察器覆盖常见 qualified/multiline impl 并给行号 | 搜索未命中只能说明所查范围，cfg/宏/别名/树外实现仍 unknown |
| 旧计数示例误分生产/测试 | 删除用累计 prod 推单文件 test 的 Node 示例，改为定位后核实际 cfg 边界 | 不新造全仓 Rust 静态分析框架；历史快照不作为现值 |
| 文案说未接线，已有 Base 消费 | 仅改 `slot_base.rs` 两处 rustdoc，承认显式选定 Base 视图并保持 legacy 边界 | 不宣称普通聊天固定跑六槽或产品全部接入 |
| 单实现接口被候选规则拒绝 | 依据当前职责、公共/树外契约和真实替换需求审阅，不强制第二实现者 | 仍拒绝无当前需求的预设泛型/接口，公共变更仍走 G8 |
| 四个数量全只许降低 | 将候选脚本改为 OBSERVATION_ONLY；Git 已跟踪工作区、方法、坐标、限制可查 | 数量不是质量、覆盖或生产可达性；不接入必需 CI |
| 多场景/冻结代码被误作冗余 | 分别核共享 core、职责与当前冻结决定 | 本批不重构脚本家族、不删除默认关闭的实现 |
| 拆分指南用 HEAD 重切 | 当前磁盘原件/hash 先保全，Rust 核所有权/可见性/feature/顺序 | 不以文件行数强拆，不覆盖并发未提交工作 |
| 任何整理都要求完整 doctest | 适用公共 API 变化按 G8，文案/私有等价整理按实际影响测 | 现有正式 CI 和债 Done 条件不降低 |

## 方法与范围

写集与 CI 节奏见[本批有限计划](../ROUND-02-PLAN.md#ai-限制优化与有限证据复核2026-10-09)。原分层 `3/1`、依赖重复组 `80` 与其历史说明保留；只增加观察配置和 npm 入口。观察器一次读每个已跟踪源文件，排除注释/字面量后做文本匹配；包括源码中的测试与默认关闭代码，不按运行时配置过滤。不再吞读取失败为零；错误输入、Git/读取/编码失败为非零。

基于 Git 索引的观察排除 untracked/ignored，故 DeepSeek 原候选的 `43/22/2558/106` 不作为新方法的硬基线，也不将两种口径直接比较为偿债成果。新增脚本在暂存前不计入已跟踪脚本数，这是范围定义，不能写成删除了脚本。输出行数按物理行计，末尾换行不虚增一行。

所有普通运行源码、公开方法/请求/结果、权限、默认 feature、锁和 CI 组合保持；Rust 只改注释。六槽 Base、最小角色共享消费者与小 Kernel 的边界不变，原 Full/双核/单 Agent/签名等暂缓不解冻。Memory selector 和 trace 脚本的结构候选没有行为或职责重复证据，不借建议启动新架构整理。

## 验收记录

命令 cwd 为本批工作树 `E:/OCLive/_worktrees/ai-guardrails-20261009-r0`；原始日志与建议/四文件保全在本机 `E:/OCLive/_recovery/AI-GUARDRAILS-20261009-R0/`，不随 Git 自动转让。记录按命令级退出码判断；局部绿灯不称正式 CI 或技术债 Done。

| 命令 / 核对 | 实测 | 为何适用 |
|---|---|---|
| `node scripts/check-abstraction-ratchet.mjs --self-test` | exit 0，限定/跨行 impl、注释/字面量、增长及输入/读取错误负例通过 | 新观察器直接同实现回归 |
| `node scripts/check-abstraction-ratchet.mjs --json`；`npm run check:abstraction` | 均 exit 0；1213 已跟踪源文件，43 trait 声明 / 23 仅一个实现文本命中的声明 / 最大2557物理行 / 105已跟踪脚本文件 | 确认真实收集及 npm 入口；这些数不是质量裁定 |
| 当前脚本的独立 Git 夹具 CLI | 6000 行文件 exit 0；非法 UTF-8 exit 1 且给文件路径；未知 `--strict` exit 1 | 不误挡增长，收集失败不得静默归零；两个 exit 1 均为预期负控 |
| `cargo test --locked --offline -p oclive_kernel_contracts --doc` | exit 0，原11 passed / 0 failed | 仅 rustdoc 修正，原示例和签名未变 |
| `cargo fmt --all -- --check`；`node --check scripts/check-abstraction-ratchet.mjs` | 均 exit 0 | Rust 注释/新增脚本语法 |
| `node scripts/check-domain-layering.mjs` | exit 0，3 imports / 1 FQ，与原硬基线相等 | 确认基线扩配置未放宽原约束 |
| `node scripts/dimension5-acceptance.mjs --ci` | exit 0，PASS (31 checks)；sample lib 为显式 skip | 脚本/基线配置变动；原组合/构建/依赖与纪律保持，不是全量 CI |
| 默认 `check-markdown-links.mjs` + 八份改文显式路径 | 两条 exit 0，默认52 / 显式8文档 | 新规则/入口链接关联闭环 |
| `check-stale-paths.mjs --docs-only`、`check-doc-registry.mjs`、`check-debt-marathon.mjs`、`check-doc-encoding.mjs`、`git diff --cached --check` | 均 exit 0 | 适用文档/计划/事件/编码/格式检查；债结构不代替状态结案 |
| 十二路径精确核对 | 原计划/DCL历史前缀、原硬基线全部字段、依赖与其余 npm scripts、Rust 可执行正文和原 doctest 示例均相同 | 防止规则整理夹带业务/锁/历史变化 |

Dimension 5 与 doctest 仅在子进程绑定本机已有 Cargo target；Dimension 5 使用已安装 portable Python 3.12，以及只读指向主树现有 node_modules 的工作树 Junction，未安装/升级依赖或触及用户 Python 环境。Git 换行提示、Rust 编译进度和预期负控 stderr 各自保留，不把所有 stderr 称为故障或零错误。缓存 `cargo audit --no-fetch --stale` 只按现行门禁口径，不宣称重新完成在线安全审计。

观察器首次仓库收集后，仅为减少重复目录 stat 与补读取错误路径做了一次局部修正；最终脚本另跑同实现自测、真实观察/npm与 CLI 负控。早期输出保留，Git 范围脚本数暂存前104、暂存后105只源于新增脚本进入索引。最终报告文案更新后只复验受影响文档/格式和写集，业务源码与已测原组合未变，不再重复全量或同一测试。

## 接手与停止

规则已有明确修复方案，不需要维护者再裁定产品/架构取舍。完成适用门禁与十二路径/历史/原件核验即结束本批；未来是否把观察器升级为硬门禁须另定可靠度、范围和合理的准入条件，不能只套四个总量上限。后续开发继续按实际目标，不再扩大“证明没有过度设计”的调查。
