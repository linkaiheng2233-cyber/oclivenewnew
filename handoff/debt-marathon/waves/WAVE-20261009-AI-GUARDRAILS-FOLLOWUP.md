# 辅助限制接续与并行基线合流（2026-10-09）

**状态**：Locally verified。M / 常规；controller 直接实施与语义自查，`independent=false`。规则继续以 [AI_CHANGE_BOUNDARIES](../../AI_CHANGE_BOUNDARIES.md)、[AI_VERIFICATION_PROTOCOL](../../AI_VERIFICATION_PROTOCOL.md)和[现行工程门禁](../AI_AND_PIPELINE_GATES.md)为准。本报告是本批交接记录，不另建规则、债务状态表或必需门禁。

## 为什么做这一片

维护者要求继续主线，并接手辅助限制的优化完善。上一批 [AI 限制优化](WAVE-20261009-AI-GUARDRAILS.md)已收口，本轮只处理实际接手中出现的四处问题；不重新调查全仓抽象、脚本或六槽最小性。

| 已定位问题 | 修正与例子 | 保留边界 |
|---|---|---|
| 收尾摘要仍把所有整理列为必须跑 workspace doctest / Dimension 5 | 摘要改为按项目门禁表选择；公开 API 变化仍按 G8。纯文案整理无需为了一个摘要重复完整链 | 原 G8、L 类出口、required gate 及维护者明确要求均不降低 |
| 编译红可能被误写成公共契约缺陷 | 红绿证据明确区分原缺陷与构建前置条件。Memory 案例最初缺少新写的私有实现文件，不说明旧 Base 契约有缺口 | 不为制造公共符号红灯而删旧代码、放宽断言或改变契约 |
| 观察 CI 的本地错误可能被误写为远端失败 | 核实协议区分 `gh run watch` 的 API EOF 与目标 run / job 终态；只读恢复查询不是重跑 CI | 无法查询时保留最后已知状态；真实 job 失败仍须看原始日志 |
| 两支线都使用 DCL-20261009-76，且入口文案与新 rustdoc 不一致 | 保留 AI 事件 76，导入 Memory 事件为 77 并记录完整来源 SHA；本片为 78。MODULE_MAP 将早期 B1 的未接线说明改作历史描述 | 原提交和原件不回改；现有显式 Base 绑定不等于固定六槽顺序 |

例如，查 CI 时 API 连接断开只说明这次查询没有完成，不能据此给 Windows 测试判失败；同样，外部示例作者还没写自己的 `memory.rs`，不能据此要求 Kernel 增加一套新接口。这两条只校正证据归因，不为普通开发新增一层执行器或硬门禁。

## 合流范围与证据身份

起点为本地 main `b6a918748a933757011611d625e0e6e17f2e6823`。在独立工作树合入 Memory 分支 `f94069ddd7b26e7661c81dd310175c0b6fc9e857`，保留双方为合流父提交；共同祖先为 `894047c52780375c9c01d596e442ff59231c596d`。DCL 和 ROUND 计划的两处追加冲突逐段保留后解决，没有用一侧整体覆盖另一侧。

相对起点的写集为九条：两个 AI 规则文档、MODULE_MAP、DCL、ROUND 计划、本报告，以及导入的 [Memory 原 Wave](WAVE-20261009-EXTERNAL-MEMORY.md)与两个 [example 源文件](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs)。原 Memory Wave 和两个 Rust 文件与原提交的 Git blob 保持相同；本工作树 checkout 为 CRLF，原工作树为 LF，磁盘 bytes / SHA256 因换行不同而分别登记，不冒充原始字节相同。原工作树、原 AI 与 Memory 历史正文保留，编号来源另作说明。

公开签名、运行源码、锁、默认 feature、观察器、基线配置、npm scripts、CI 组合和台账 / 队列状态均不再修改。没有签名 / TLS、双核、Full 韧性、多 Agent、高级情绪记忆、Production Stream 或 CI 分层解冻。

原 Memory 实现的完整本地链成功运行于 `f94069dd`，耗时 1446.3022212 秒；[正式 CI run 37887351660](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37887351660)的目标同为 `f94069ddd7b26e7661c81dd310175c0b6fc9e857`，17 个 job 成功、0 跳过，`ci-gate` 成功。第一次 watch 的本地 API EOF 原件保留，随后只恢复同一 run 的终态查询，没有另发运行。本轮复用这些证据时仍保留原 SHA，不把它们改写为组合提交的正式 CI。

## 本轮适用验证

实际命令与 stdout / stderr / native exit 分件保存在本机忽略目录 `E:/OCLive/_worktrees/ai-guardrails-followup-20261009-r0/.cursor/plans/ai-guardrails-followup-20261009-r0/`；该目录不随 Git 自动转让。下列结果均有本批命令回执，不能用命令存在或原批次绿灯代替实际退出码。

| 本轮命令 / 核对 | 实际结果 | 证据范围 |
|---|---|---|
| `cargo test --locked --offline -p oclive_kernel_runtime --example external_memory_base` | native 0；3 passed / 0 failed；14.2667995 秒 | 导入的同一个外部 Memory 实现经现有消费者调用，未修改例子 |
| `node scripts/check-abstraction-ratchet.mjs --self-test` | native 0，PASS | 原观察器同实现回归，脚本未改 |
| 同脚本 `--json` | 解决索引冲突后 native 0；1215 已跟踪源文件，43 trait / 23 单实现文本命中 / 最大2557物理行 / 105脚本 | `OBSERVATION_ONLY`；1213→1215 只因导入两个 example 文件，不是质量评分 |
| 默认 / 七份改文显式 `check-markdown-links.mjs` | native 0；52 / 7 Markdown | 当前入口与改文链接，非全仓所有文档完整性证明 |
| `check-stale-paths.mjs --docs-only`、`check-doc-registry.mjs` | 各 native 0；26 handoff root / 5 canonical sentinels | 现行文档门禁 |
| `check-debt-marathon.mjs` | native 0；147 台账行 / 12 auto plans 结构通过 | 不代表债务状态评估或 Done |
| 七份 `check-doc-encoding.mjs --file …`、`git diff --cached --check` | 各 native 0 | 适用编码 / 格式 |
| 本批九路径与历史对账 | native 0，8 核对项通过 | 两个原计划段落、DCL 原正文和编号来源、原 example / Wave 的 Git 内容、MODULE_MAP 唯一追加后的说明修正 |

失败尝试原件保留：合流命令首次 native 1 是 DCL / ROUND 尾部冲突；首次观察器 JSON 收集 native 1 是尚未确认解决的索引中同一路径出现多个 stage，解决并显式暂存后原脚本通过。本批自查首次要求跨工作树磁盘 bytes 全同而 native 1，实际是 Git checkout 的 LF / CRLF 差异；改为分别记录双方磁盘身份并严格核同一 Git blob 与仅换行差异后通过，没有为了自查修改 Rust / 原 Wave 或原件。Git 换行提示和 Cargo 编译进度不称为测试失败或零 stderr。

本轮只复验同一个外部 Memory example、原观察器的同实现自测与真实 JSON 收集，以及适用文档、编码、登记、债务结构、写集 / 历史核对和 diff。没有公共 API / 运行语义 / 门禁脚本变化，不再运行完整本地链、workspace doctest、Dimension 5 或远端完整矩阵。纯报告文案收尾只复验受影响的文档检查。

## 接手与停止条件

有限规则修正、定向验证和九路径保全核对完成后结束本片。若 main 仍为起点且干净，再安全快进合入本地 main；有并发变化则保留分支，不强行替换。此次不推远端 main，不为回写绿灯再造提交。

本轮没有新的架构问题需要维护者裁定。模块实现者主线继续以一个实际调用案例为单位，不扩查其余五槽或发行版；下一次公共契约、权限 / 调度权或新增强制义务的改变仍需明确审阅。H04 的既有有限验收、S01 历史 FAIL、真实 TTS / 平台 / 未覆盖崩溃窗口 / 独立浏览器发行形态等边界均不扩大。
