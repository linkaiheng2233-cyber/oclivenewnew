# 公共基础契约本地检查出口 · 2026-10-11

**SSOT范围**：本片实施、有限验收与续跑快照；责任分层和命令范围由[CI设计](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#kernel-host-ci-first-slice)拥有，使用动作见[人类门禁矩阵](../../../human-docs/08_PR_GATE_MATRIX.md)。
**状态**：Locally verified；真实四步、定向回归及适用治理/文档门禁已通过。仅本地开发切片，不构成新SHA正式CI或技术债Done。
**身份**：base `a74e249c8c444525073eecf3705e587f0df84089`；分支 `codex/ci-layering-preparation-20261011-r0`；controller实施和语义自查，`independent=false`。没有使用马拉松runner claim或业务run ID，不复用历史场景。

## 原因与实际收益

当前Rust全量入口会准备前端/Tauri并覆盖参考Host，基础模块作者则首先需要知道公共契约测试是否成立。本片把既有四条Cargo命令合成 `npm run check:kernel-base`，复用现有native工具定位/错误处理，不增加依赖、框架、DTO或trait。例如Memory作者先验公共基线，再在自己的模块工程验检索行为；消费者绑定及发行版验收仍按实际目标补测。

本片只修改[计划](../ROUND-02-PLAN.md#k-ci-impact-01--公共基础契约本地出口2026-10-11)登记的十路径；七份既有原件及SHA256已在本机 ignored `.cursor/plans/kernel-base-local-check-20261011-r0/` 保全。计划与DCL旧正文仅追加；公共Rust、Cargo/npm锁、生产实现、原四条测试、主工作流与所有冻结项保持。

## 已执行的定向验收

环境为Windows、Node22.23.2、现有Rust工具链；原生退出码即时捕获，stdout/stderr/exit件按命令分别保存。R1故意红与R2绿不覆盖。

| 入口 / 证据标签 | 真实结果 | 可支持的结论 |
|---|---|---|
| R1-entry-absent-red | exit1，ERR_MODULE_NOT_FOUND | 新脚本尚未实施；不是旧公共契约缺陷 |
| R2-chain-green | 4 passed / 0 failed，exit0 | 选择固定四步；同一链第二步真实native exit7即停止，后续不执行；真实CLI缺cargo/未知参数非零且无PASS |
| R3-real-npm-chain | exit0，36.21秒 | 实际从npm入口运行全部四步；不是注入runner的通过 |
| Base fixture / minimal role | 16 / 2 passed | 沿用现有契约相关用例，无Host装配或服务启动 |
| 两crate Clippy / 显式doc | exit0 / 11+17 passed | 当前两crate编译面与文档示例通过；不含workspace其它crate |
| R4-public-crate-tree | exit0；目标两crate的normal/dev依赖未含Host/runtime/Tauri | 只判当前目标依赖，不作全仓或所有feature边界证明 |
| R5-toolchain-regression | 17 passed / 0 failed，exit0 | 复用helper原命令合同未回退 |
| S1 / S2 syntax | 均exit0 | 新脚本与测试语法通过 |

Cargo进度输出为正常stderr，不把有stderr等同失败。入口失败返回非零，原native退出码保留在错误诊断中；没有承诺把所有native码原样作为npm退出码。每步20分钟超时，输出在步结束后显示，沿既有helper上限处理，不引入持续监控或另一份调度实现。

## 治理门禁与自查

| 检查 | 结果与适用原因 |
|---|---|
| Dimension5 `--ci`，G1b | 原生命令exit0，PASS (31 checks)；sample lib按原规则SKIP仍计入。新检查入口属于门禁脚本改动，适用 |
| D1 / D2链接 | 默认入口及本轮七份Markdown显式目标均exit0；新报告收口文字另做有限复核 |
| D3–D7路径、镜像、登记、债结构、编码 | 均exit0；文档与债计划关联改动适用，结构绿不代替债状态判断 |
| 精确十路径、历史正文、package差量与diff | PASS；两历史原始字节前缀保留，package只增加一个脚本，UTF-8无BOM；controller语义自查 findings=[] |
| 公开Rust / 业务 / 远端 | G8 workspace doctest不适用（未改变公开签名）；本入口已显式运行两crate doctest。不重复完整业务链/发版或远端CI |

**环境失败与自主修补**：G1原生exit1，29项未失败（包含sample SKIP），两项失败均由工作树环境缺项造成：语音检查直接启动Python但PATH只有cmd包装器；本工作树尚无node_modules导致Vite入口缺失。保留原日志，在本工作树创建ignored node_modules Junction到已核主树依赖，并在验证子进程临时指定本机现有Python3.12.14，不安装依赖、不修改门禁。G1b原生Dimension5遂exit0；不因此把G1改写为成功。

**包装器失败单列**：G1b外层PowerShell仍exit1：我的临时环境包装器将原先不存在的变量经.NET setter恢复成空字符串，收尾断言如实拒绝；原生门禁exit0和包装器exit1不能混为一谈。该工具进程结束，未修改调用方持久环境。原环境报告保留，后续正确恢复按原变量存在性分支，不存在时Remove-Item环境项；E1在同一进程验证不存在/空值/非空值三状态各含正常与抛错路径，共6项通过，子进程实读指定值、最终变量不存在，exit0。只验证修正后的收尾，不为它重跑已通过Dimension5或业务。修补和证据限本机ignored辅助文件，不新增仓库执行框架。

原始证据均位于本机 `.cursor/plans/kernel-base-local-check-20261011-r0/`；`baseline.json`、`R1/R2/R3/R4/R5`及`G1/G1b/E1`标签分别有stdout/stderr/exit件。语义自查为controller自查，不能称独立审查。门禁未接入必需workflow、未改required context；后续远端片需给入口回归明确执行所有权。

## 边界、主树与下一条动作

主树与实际origin/main仍为 `24f62303e9bdea37061fe44acc7a0830dc519f98`；它的正式[CI38111785703](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/38111785703)17/17成功是独立历史基线，不证明本片新SHA。当前入口只在此本地开发分支可用，未合入/推送main、未创建PR。本片不把K-CI-IMPACT-01或任何其它债转Done。

contracts/types保留rich兼容导出，不能将整crate等同小Kernel；本入口不验每个硬边界、共享消费者、模块目标质量、Host业务、全部发行版或真实模型/TTS/平台/崩溃/性能。脚手架生成工程仍自行执行Cargo测试，入口不会自动发现外部模块。

达到本地命令成功与失败关闭即停止本片。下一条动作是在当前干净提交上，有限核现有rust责任组与受信catalog命令，然后设计远端唯一执行所有权；保留旧组合回归，不能靠重复运行同一批测试宣称分层完成。需要改变ready选择类别、权限、公共强制义务或冻结项时交回维护者。当前静态验证可安全重跑（新日志标签）；业务/真实模型执行未授权于本片。
