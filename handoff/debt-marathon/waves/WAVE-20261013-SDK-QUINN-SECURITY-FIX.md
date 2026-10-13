# SDK正式CI暴露的Quinn兼容安全补丁

**范围 / 日期**：具体安全门禁阻碍的修补与SDK冻结续接；2026-10-13。漏洞状态由[KNOWN_VULNERABILITIES](../../../creator-docs/security/KNOWN_VULNERABILITIES.md)拥有，开发操作仍由[Memory指南](../../../examples/developer-sdk-memory/README.md)拥有。
**身份 / 基线**：controller实施与语义自查 independent=false；起点干净 `c42da06af5c2a71bcc09850776ad38476cea26bc`，原 `checkpoint/source-sdk-final-20261013-r0` 已远端推送且不移动。沿[六路径计划](../ROUND-02-PLAN.md#sdk正式ci暴露的quinn兼容安全补丁2026-10-13)，PR #188先转draft，修补后冻结新提交/r1 tag再转ready，不在未冻结时重复触发正式全矩阵。

## 原问题、来源和有限修复

首轮正式 [CI 38375524660](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38375524660) 的Dimension5 audit命中锁内 `quinn-proto 0.11.17`：RUSTSEC-2026-0338、0340、0343，修复要求至少0.11.18。SDK提交未改变Cargo.lock，同一问题也在原main锁中；先前本地 `--no-fetch --stale` 使用旧数据库通过，不足以替代当前审计，也不是SDK实现引入漏洞。

刷新本地数据库后原锁实际native1、三条命中，与远端相符。数据库commit `d5f46becf4d442d460e1fce5e12f2c0f00924868`、1308公告、698锁依赖；[上游0.11.18发行](https://github.com/quinn-rs/quinn/releases/tag/quinn-proto-0.11.18)明确修复该组panic问题。不将公告声明日期自动视为本地数据库已包含日期。

精确 `cargo update -p quinn-proto --precise 0.11.18`。Cargo同时重选了 `tempfile → getrandom` 边；它不属于这次修补，旧0.4.3符合tempfile的 `>=0.3.0,<0.5` 范围，因此保全Cargo初版锁后恢复原边。locked metadata和全target/allfeature树均接受最终锁。逐包比较698→698，仅Quinn的version/checksum不同，其余包与全部依赖边逐项相同；不更新manifest/feature/TLS配置、审计ignore或项目API。

当前默认及全target/allfeature workspace反向树对Quinn均提示nothing to print。它只限定这些构建配置，不证明任意未来配置无暴露；锁审计仍应修复，不能据此豁免。Memory作者工程没有Quinn依赖，但官方配方使用根锁作新工程种子，故实际重跑配方检查。

后置审计native0、漏洞级0；仍有7条警告：五个unic与unsafe-libyaml的6条unmaintained，glib的1条unsound。中英安全文同步当前扫描/三ID状态及unsafe-libyaml跟踪；历史扫描叙述保留，不将本次0写成永久安全或关闭父债。

## 原生证据和验收节奏

本机原始stdout/stderr/退出件在 `.cursor/plans/sdk-final-security-fix-20261013-r1/`。Cargo审计主动刷新现有数据库，精确更新/下载官方补丁；其它验证沿已有缓存，未更换工具链、安装新工具或启用实际QUIC通信。

| 记录 | 实际结果 |
|---|---|
| R1 | 原锁 `cargo audit --json`刷新数据库：native1，三公告命中 |
| R2 | Cargo精确更新native0，最初额外重选边的锁单独保全 |
| R3 | Cargo初版后置audit native0；不是最终锁比较的替代 |
| R5/R6 | 恢复原tempfile边后的locked metadata与audit native0；0漏洞/7警告 |
| R4/R7 | 默认与全target/allfeature反向树native0、未打印Quinn路径；R4同时下载官方0.11.18 |
| precise-lock-delta.json | 698→698，唯一改变包Quinn，字段只有version/checksum，所有依赖边不变 |
| R8 | 新锁下SDK树外真实配方native0、1 passed；含作者三测、实际调用/run及具名失败负控 |

R9当前数据库下Dimension5实际PASS31、native0（含cargo-deny；sample lib按--ci跳过）；R10 CLI all-target Clippy native0。五文门禁按最终文字另核。原SDK完整阶段本地验收与首轮远端CLI通过仅覆盖其原输入；新SHA正式门禁仍需要独立的一轮，不能借旧绿灯转为Done。首轮其它job事实待运行结束后按各自结果保留，不无变化rerun失败。

源码、模板、SDK指南/教学、原例子/回归、固定脚本、workflow/catalog/required及技术债主台账保持。六路径包括锁与五份Markdown，PLAN/DCL只追加，旧SDK报告和原件不回改；没有新增SDK门面、公共强制义务或运行权力。原架构冻结、真实音频/模型、平台实机等边界不扩大。

## 续接冻结与远端

适用验收后以单独的供应链修补提交分类，新增注释tag `checkpoint/source-sdk-final-20261013-r1`；旧r0 tag和失败记录保留。精确推送同一PR分支与新tag，再转ready触发新SHA正式CI；未成功前仅称Locally verified。绿灯之后只在交付报告核对最终SHA、run与门禁，不为回写编号制造新提交；不自动合入受保护main。

达到兼容补丁、三公告消除、锁范围准确和相关配方通过即停止；不为此研究整个QUIC协议、扩展SDK案例或迁移YAML警告依赖。本片未出现需要维护者选择的新契约、权限或架构事项。

## CI失败诊断输出保真续片

Quinn六路径独立提交 `134f19a6` 后，续接首轮Windows参考Host默认job的native101。GitHub原始job日志已保全，但旧 `GateToolchainError`消息只选stderr前4000字符，编译进度遮住尾部错误，stdout断言也丢失；现有原始流属性却是完整的。因此远端有效首因unknown，不能用臆测修Host。R12直接默认Host Cargo命令在本机native0，也不能替代远端失败事实。

只修工具链lib的错误显示：分别标stdout/stderr，短流保留全文，长流保留有界首尾并标记省略，完整属性不截断。code/native exit、超时、cwd、命令和接受码不改，三固定检查脚本不改。两项实际子进程负控检查stderr编译输出不能遮住stdout断言，以及双流尾部故障必须可见；不是第二套日志或诊断协议。

R13初版负控用node -e把故障串放在命令参数中，错误头本就能看到该串，导致一项假通过（14/15）。发现后改为真实fixture脚本，让参数不含预期故障文字；R14旧helper13过/2败，R15新helper与三入口30/30 native0。两个红日志及原件保全，正确负控的红/绿才用于验收；没有为了通过弱化断言。

本续片五路径包括两脚本和PLAN/DCL/本Wave追加。最终helper下Dimension5及适用三文门禁后单独工具提交，再与供应链提交一起冻结r1。取消旧SHA因转draft自动排队的草稿run `38377361736`，避免已知失败输入重复耗时；原正式run的两个失败job及其它结果按最终状态记录，不借取消改成通过。新SHA正式CI负责重新验证：若WindowsHost仍失败，保留有效首因再定点处理；没有same-SHA rerun或无界Host调查。

R24在最终helper下实际Dimension5 PASS31、native0；与R15的30项回归共同验收这次错误显示变化。R12默认Host完整默认测试native0的事实保留，不写成远端问题已经修复。后续只核脚本语法/三文门禁、精确diff及冻结身份，不再添加测试或推导缺失的远端首因。

## Windows测试夹具释放等待续片

r1冻结 `27c514bc` 后，对同时产生的两次PR运行曾按运行编号顺序判断事件身份，判断不充分，现撤回：38381090723实际上是draft运行，21项成功的总门禁为 `ci-draft-gate`；38381090607是正式运行，plan被取消、其余19项成功、`ci-gate`失败。取消plan在主动cancel之前已观察到，不能将全部取消因果归给后一次操作；controller把90723当正式运行并试图结束90607的执行安排有误，原操作与快照保留。草稿通过不替代正式结果。

为修补这处取消阻碍执行一次 `--failed`，最初预期只补plan与gate，实际GitHub还重跑了所有下游。attempt2为19成功、Windows默认Host失败、正式总门禁失败，不能继续称只重跑两个job。该次已知取消的补跑不是对产品失败反复赌博；出现新有效失败后没有再rerun。正式原始失败日志保全于 `.cursor/plans/sdk-final-security-fix-20261013-r1/R43-r1-formal-attempt2-host-failure-log.log`。

新的双流诊断确实指出 `extra_requirements_and_blank_message_are_rejected_before_model_calls` 在清理时2秒DB独占探测超时。生产 `shutdown(self)`已在夹具中调用；当前证据只证明句柄未在2秒内释放，不证明占用者或所有worker已join。只按[本片计划](../ROUND-02-PLAN.md#sdk收尾windows夹具释放等待2026-10-13)调整该测试文件的有界等待，不研究全仓生命周期、不改生产SQLite/shutdown或模块承诺。

测试共享原独占探测：仍用 `share_mode(0)`，只有错误32可等待，其它错误立即失败；测试夹具上限2→10秒，不添加固定睡眠。两个Windows定向用例持有真实文件句柄：2.2秒后释放的正例在旧上限1过/1败（F1 native101），新上限两项成功（F2 native0）；持续持有的负例在短deadline内仍失败，关闭句柄后同一探测成功。各自临时目录显式close，成功不是忽略清理或删除断言。它证明等待行为与上限适配，不声称重现了CI具体句柄持有原因。

本机证据在 `.cursor/plans/sdk-final-fixture-fix-20261013-r2/`，原件磁盘SHA256 `D022A8662EFB473DA6D4A05D6AA43578D942C9C77B5610AC352E37FF3515E88A`，不是用HEAD替代未提交原件。F3默认Host完整Cargo测试native0：lib637、minimal_role_public_api在Windows16、其它默认集成目标均通过；没有ignored业务场景/真实模型/TTS。本片仅测试与三文追加，额外公共API doctest不适用。适用Clippy/fmt/分层和文档检查在冻结前核实。

旧r0/r1 checkpoint、正式/草稿各次原生结果均保留。有限返修完成本地验收后冻结新测试提交和 `checkpoint/source-sdk-final-20261013-r2`，只进行一次ready分支同步推送，让当前SHA正式ci-gate验收，避免再次交错push/ready事件；其成功与否另核。没有新增SDK功能、公共强制义务、工作流选择或技术债Done声明，不自动合main，不为远端状态回写制造新SHA。

最终本地F9该测试目标16/16、F10默认Host all-target Clippy、F11workspace fmt均native0；F3其它默认测试结果按其原范围复用。F6分层、显式三文及默认链接、债结构、编码、登记、旧文路径均通过。F5初次fmt要求一个assert折行，修正并保留日志；F8误用不存在的债脚本名而native1，按package.json真实入口F12通过，不算仓库结构失败。F7标签虽含default，实际只传三文，默认范围由F13单独通过；二者不能混算。两条新负控的相同文件准备已在同一测试文件复用，无新通用测试框架。
