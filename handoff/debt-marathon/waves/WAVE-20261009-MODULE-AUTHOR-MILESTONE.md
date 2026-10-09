# 模块实现者路线 · 可转让里程碑（2026-10-09）

**性质**：本批工程交接快照与接手入口，不是新的架构、规则或债务状态 SSOT。当前债务查[主台账](../../TECHNICAL_DEBT_INVENTORY.md)，执行边界查[AI限制](../../AI_CHANGE_BOUNDARIES.md)、[核实协议](../../AI_VERIFICATION_PROTOCOL.md)和[流水线](../../../.cursor/skills/oclive-dev-pipeline/SKILL.md)。
**起点**：干净main `566a8a69b2433b5e13e1ac6b9f1a1274d4e7843f`；起始实际远端 `2ef5af051aea57fb76e6cd4d05e3a3468a8b0e63`。controller实施和总审，`independent=false`。本报告随实质对账冻结；冻结之后的本地完整链和正式CI结果以原生回执、最终交付与目标SHA Actions为准，不提前写通过。

## 1. 可以直接接手什么

本里程碑面向模块／插槽实现者及新Host作者。公开六槽Base保持，核心外共享消费者负责最小适配，Host选择能力、绑定资源并拥有自己的状态与调度；没有固定“六槽一轮全部执行”的新要求。小Kernel／共享运行库／Host的职责继续由[模块注册表](../../MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)定义。

| 要开始的动作 | 唯一入口或已有交付 | 当前可用范围 |
|---|---|---|
| 查六槽接入点 | [一页Base接入点](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页) | 请求、返回、失败、资源与消费者定位，不是统一的发行版调度 |
| 写一个独立Memory实现 | [人类Memory指南](../../../human-docs/modules/slots/memory.md)、[外部调用案例](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs) | 独立实现通过现有消费者调用；最新N、literal和正常空结果有实际回归，不新增契约或发现框架 |
| 选择其它槽的基础入口 | [人类模块目录](../../../human-docs/modules/README.md) | 五槽导航已有；导航不冒充新增五槽实况，丰富接口仍可使用 |
| 判读公共面兼容 | [有限兼容规则](../../../creator-docs/COMPATIBILITY.md#six-slot-minimal-compatibility-draft) | Rust源码、数据和行为分别审；新权限、调度或公共强制义务须维护者确认 |
| 按现行AI规则开发 | [限制优化交付](WAVE-20261009-AI-GUARDRAILS.md)、[二次质量复核](WAVE-20261009-SECOND-QUALITY-REVIEW.md) | 数量观察不作质量裁判，搜索未命中不判不存在；发现足以行动即停止调查 |

Memory案例的有限本地运行命令：

```sh
cargo run --locked --offline -p oclive_kernel_runtime --example external_memory_base
cargo test --locked --offline -p oclive_kernel_runtime --example external_memory_base
```

这份案例只有立即就绪的内存能力，自己的立即轮询器不是通用异步执行器；真实I/O、线程安全、取消、资源、持久化与权限由对应Host/实现负责。旧 `minimal_role_host` 教学例子的私有错误只保留类别，不应原样作为生产诊断模板；完整失败保留的实际消费者／Host证据见二次质量报告，不据此改公共契约。

## 2. 此次合批与对账内容

起点之前的七提交分别保留了全景审查、外部Memory实现、AI限制优化、并行接手对账、Memory人类入口、其余五槽导航和二次有限质量报告。它们的细节与失败记录由各Wave拥有，不在本页复制；本次只把已成形成果交到主线。

本次唯一实质整理是两条过期待验：测试环境隔离片的原正式CI为 `37864145434` @ `d9847b70c8c6164ab4715813f7e55bcfb6bb20ed`；供应链撤销批次的原正式CI为 `37877817104` @ `2ef5af051aea57fb76e6cd4d05e3a3468a8b0e63`。原行、原因与当前状态分别读[历史快照](../../archive/TECHNICAL_DEBT_HANDOFF_ROWS_20261009.md)、[DCL-82](../DEBT_CHANGELOG.md#dcl-20261009-82--模块实现者里程碑与两条过期待验对账)和主台账。本次不关闭父债、不升级其它依赖、不改变旧执行结果。

## 3. 验收与转让方式

冻结前先跑五路径适用文档与保全检查；冻结后针对积累批次运行一次 `npm run check:ci-local`。**干净checkout前置**：依赖和本机工具链按[CONTRIBUTING](../../../CONTRIBUTING.md)准备；桌面Tauri配置需要内核资源，先执行下列仓库入口（正式CI也在Rust阶段前执行），不要用空的主平台二进制制造通过：

```sh
node scripts/bundle-kernel-for-tauri.mjs --profile debug
npm run check:ci-local
```

资源只用于本地编译／测试，不等于发布包或真实模型验收；辅助Voice测试使用已准备的本机Python时在验证子进程显式设置 `OCLIVE_VOICE_PYTHON`，不把本机绝对路径当跨机器要求。此次没有新的公开签名、re-export或doctest示例变更，不额外重复workspace doctest；原公开片的doctest／完整链证据仍按原SHA解读。正式出口为冻结提交对应的 `ci.yml` 全量终态和成功 `ci-gate`，不能把旧SHA的绿灯算到本提交。

原始本地stdout/stderr、native exit、cwd、SHA、环境、历史原件与远端API快照保存在 `E:/OCLive/_worktrees/milestone-handoff-20261009-r0/.cursor/plans/milestone-handoff-20261009-r0/`；这些ignored原件不随Git自动携带。跨机器接手以Git中的实现/回归/文档以及目标SHA的正式Actions为可取得入口，需复查本地历史原件时由维护者提供，禁止补造。

取得最终结果后只在交付报告披露，不追加仅回写run ID的提交。接手者用 `git rev-parse HEAD` 取得实际完整SHA，再执行：

```sh
gh run list --workflow ci.yml --commit <完整SHA> --json databaseId,headSha,status,conclusion,url
```

需要验证当前改动时按实际影响选窄测；不因接手者更换就重跑全部历史场景。结构门禁通过、语义自查和真实设备验收分别判读。

**本轮自纠**：桌面工具因调用上下文在Git父目录而拒绝创建worktree，改为在已核仓库使用原生Git创建独立干净工作树；没有清理旧树。一次只读输入把npm组合入口误作不存在的 `scripts/check-ci-local.mjs`，按实际package.json读取组合命令；一次短SHA `d984` 因对象歧义拒绝，改用已核完整SHA。首次计划补丁锚点不匹配而未写入，读取实际尾部后追加。新交接文首次把模块目录误链为 `slots/README.md`，显式链接门禁native 1检出，改为真实 `modules/README.md`；原失败回执保留。上述均为控制者的局部工具/文档错误，不归因为产品缺陷，也不回写旧证据。

**冻结前直接检查**：默认52份链接、修正后五改文显式链接、docs-only旧路径、镜像、登记、债结构和编码均native 0；147行／12 auto plans仅为结构结果。五路径精确核对确认运行实现／公开面／锁／配置／测试／队列不变，台账仅三个许可单元格修改、两条旧行逐字保全，原计划与DCL历史前缀保持。校验脚本只存本轮ignored目录，由同一controller编写，不能称独立复核。最终暂存diff与保全检查在提交前再核，完整本地和远端终态留给冻结后执行。

首次暂存diff另检出新归档文件的EOF多一个空行，native 2阻止提交；移除多余空行后重跑适用检查，归档行文字不变。该失败与Git换行提示分开保留，不把提交前失败写成首跑全绿。

**本地里程碑实际过程**：首次冻结 `aeea1bbcb561fc61002c140c42e3551bc2ae6b6f` 后完整入口native 101，首因是本控制者漏做上述资源准备；失败前Dimension 5为PASS（31 checks，其中sample lib明确SKIP），lint/typecheck/build/fmt已通过。资源入口native 0后，保持同一Git内容，从Rust阶段接续：`npm run check:rust` native 0（Host lib 637、runtime lib 273、desktop lib 33 passed），`npm run check:rust:integration` native 0（workspace与串行CLI）。这是所有适用阶段完成，不是“完整入口首跑／第二跑native 0”；不为重复前端/D5再跑整链。生成bridge的磁盘换行／索引差异先保存raw并核同Git blob，再精确刷新该路径索引，零暂存差量，未还原源码；父环境保持。本段及资源前置在所有本地进程终态后补入，属于实质交接修正；仅Markdown变化复用原源码检查并另验文档，最终正式CI仍绑定随后冻结SHA。详见[DCL-83](../DEBT_CHANGELOG.md#dcl-20261009-83--干净工作树的验证资源前置与分阶段收口)。

## 4. 后续只从真实目标开片

下一片应明确“哪个模块实现者／哪个Host、哪个槽、当前被什么动作挡住”。优先复用现有契约和消费者，外部实现被真实调用成功即停；只有可复现的表达障碍才带新旧用例讨论公共面变化。当前暂无指定对象，不自动补五套实现、不重开全仓质量/最小性调查。

签名信任根、TLS、双核、Full韧性、多Agent、高级情绪记忆、Production Stream、统一远程wire、core物理迁移和CI责任分层沿用既定冻结。H04有限验收、S01历史语义FAIL、真实音频/TTS、平台实机、未覆盖崩溃窗口、独立浏览器和性能矩阵均不因本里程碑扩大；这些缺口不自动阻断无关的模块开发。
