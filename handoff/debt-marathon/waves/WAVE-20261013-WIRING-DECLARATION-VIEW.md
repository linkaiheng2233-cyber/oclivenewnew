# 终端接线声明视图 · 2026-10-13

**状态**：Locally verified；定向代码、实际示范、适用治理与范围检查通过；controller语义自查 independent=false，冻结SHA在交付报告披露。
**基线**：SDK `9c0ce96f13b7804c288783537143b760c89a35b5`，main `08348f31b8f3459537c13227302bf4b8e273baa8`；两冻结工作树保持。新分支 `codex/wiring-view-20261013-r0`，controller实施与语义自查 independent=false，M/常规开发片。
**范围**：[十五路径计划](../ROUND-02-PLAN.md#终端接线声明视图--脚手架中的可选元数据2026-10-13)；格式与使用SSOT为 [CLI指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#doctor-wiring)，SDK路径只链接它。本报告不改架构或技术债状态。

## 选择原因和停止线

维护者希望模块作者在替换实现后看到独立的Base与实现关系，并采纳元数据由脚手架初始提供、作者工程维护的放置方式。有限核对doctor域、config-resolve、基础模板和Memory直接消费者后已足够行动：doctor有可复用子命令域，config-resolve返回参考Host后端枚举而非任意Rust Base实现；现有绑定是借用trait对象，没有现成可读报告。

因此第一片只读作者声明，`doctor wiring`不扫描Rust/import或类型名推断事实。公共六槽键复用validation的既有常量，只做显示映射和元数据检查，不重新解析蓝图。新增CLI私有读取/格式逻辑，没有SDK门面、注册/发现系统或公共Base/Host义务。达到生成可查看、修改声明重读更新、来源与未知正确和适用验证通过即停止；实际Host报告或调用追踪另有具体来源时才另片。

## 关联闭环

| 环节 | 本片处理 |
|---|---|
| 生产者 | 基础Cargo模板仅新增可选空元数据，4/5文件及依赖保持；教学TestMemory不自动成为运行绑定 |
| 输入 | 作者清单的可选wiring v1；实现/接入短标识、用途和调用范围，缺失不猜测 |
| CLI入口 | 复用doctor子命令，显式manifest-path；新分支在原环境探测前返回，组合fix/watch/SBOM/JSON等拒绝 |
| 消费者 | 一次读取/校验完整源后输出终端；六个Base和实现分开、来源和未核实标签明确，不把排列当调度 |
| 状态 / 错误 | 不写清单、不跑作者代码；缺声明为未知，格式错误非零且不回显原值，未知/控制字符标签拒绝 |
| 文档 / 测试 | CLI指南中英拥有格式；生成README、SDK两文及crate入口接线；真实CLI和原生成/Cargo回归 |

公共contracts/types/runtime、Host/Tauri/前端、原Memory源码、其它模板和工作流、根manifest/锁、债状态、SDK原Wave无需改。15路径中12份原件先保全，PLAN/DCL历史按原磁盘字节前缀核。源端和显示端同片完成，不因CLI看见声明就宣称模块已安装。

## 定向原生结果

证据保存于本工作树 ignored `.cursor/plans/wiring-view-20261013-r0/`，每标签独立stdout/err/exit，Cargo offline，共享target/jobs只在验证调用期间设置并finally恢复；不以最后一个shell退出码替代前项native结果。

| 标签 | 实际结果与口径 |
|---|---|
| R1-wiring-cli-red | exit101：8项中2过6败；旧CLI不识别wiring、旧模板无元数据。不是公共Base表达缺陷；旧代码下只拒绝输入的两个负例不构成新功能通过 |
| G1-wiring-cli-green | exit0，8/8；首版新命令和生成断言通过 |
| G2-fmt-first | exit1，新增Rust的格式差异；当前修改件先保全，再按CLI包rustfmt规范化 |
| G3-fmt-green | 原cargo fmt --all -- --check，exit0；其它源码未被顺手规范化 |
| G4-cli-targeted-regressions | exit0，bin100＋新wiring8＋原基础E2E7共115 passed/0 failed；7项包含独立Cargo、作者模板与README有意改行为红后绿，串行运行避免锁争抢 |
| G5-cli-clippy | 默认CLI all-targets、-D warnings，exit0；未启diagnostics-host，未增加Host依赖 |

G4中原CLI单测会在临时目录执行本地Git插件夹具，stderr有clone提示；这不是查看入口的行为，也不称“零stderr”。Cargo编译输出保留。定向门禁不是CLI所有集成目标或完整发行版验收。

8项新集成分别覆盖：正常六Base与Memory声明/再次读取名称/用途为教学与空调用范围；无元数据且Rust含实现仍保持未知；无关私有元数据及TOML解析失败不泄漏值；版本/类型/未知字段/伪核实状态/控制文本拒绝；缺文件、目录或workspace-only错误；环境动作不能混入只读视图；显式清单路径且带不可编译build.rs也不构建；默认与none脚手架均初始化空声明且两依赖保持。查看子进程PATH只指夹具根，不能借Cargo/rustc/探测成功取得证据。

## 真实终端示范

新夹具位于 `E:/OCLive/_recovery/WIRING-VIEW-20261013-R0/author-project/`，不是旧SDK原件。D1实际CLI生成none基础工程；D2空声明显示六个实现未提供；D3作者声明Memory为RecentLiteralMemory、接入MinimalRoleBaseConsumer；D4只把声明名称改为AuthorMemoryV2。四条native exit0，查看前后清单hash相等，源码不由CLI修改。

```text
Project: my-sdk-memory
Wiring metadata: provided
Host verification: not connected
Binding / call / target functionality: not verified

Interfaces (relationships, not execution order):
  MemoryBase -- declared --> RecentLiteralMemory
    Contract: oclive_kernel_contracts::MemoryBase
    Entry declaration: MinimalRoleBaseConsumer
    Scope declaration: project (author declaration)
```

D4同一位置变为 `MemoryBase -- declared --> AuthorMemoryV2`。名称演示取自SDK案例；仅编辑夹具元数据，未安装/替换或执行真实模块，更没有Host核实。实际绑定和三项Memory行为证据继续查原SDK Wave，不能拿此显示代替。`demo-facts.json`显式记录NOT_RUNTIME_BINDING与两份作者编辑后的摘要。

## 适用治理、收口及边界

G6分层、G7/G8原module-compat两步、G9默认52份/G10改文8份链接、G11镜像、G12登记、G13 docs-only旧路径、G14债结构和G15 diff，均native exit0。G16精确15路径、12份原件与其原摘要、PLAN/DCL原始字节前缀、严格UTF-8/汉字、父SDK/主树clean与原Memory源码保持均exit0；未变路径由与父提交的精确diff限定，不改源/锁/CI/债状态。缓存CLI二进制在复核时另记hash，不把共享构建缓存当不可变发布产物。

只读语义自查核对：新子命令在旧环境探测前返回；仅元数据白名单参与显示；所有输入验证先于stdout；无扫描/模块调用/新Host义务；模板空声明与教学模块分开；作者改元数据和命令只读各自留hash；指南中英与源码字段/拒绝选项一致。未发现需要架构裁定的问题。最终文案更新只再核受影响链接、范围与暂存diff，Rust代码未再变，不重复115项测试或完整链。

本片修改CLI内部二进制模块及模板，未改公开DTO/trait/crate/re-export，workspace doctest不适用；也无CI控制面、发布或Done状态变化，不重复完整发行版链/远端CI。controller语义自查后本地提交，主树及SDK冻结保留，不合入/推送main、不借main的旧21/21证明新SHA。

显示限制：项目/实现/接入只能填短ASCII公开标识，作者不得将秘密填成标识；其它Cargo元数据不输出，错误不回显原TOML/字段值。没有可靠版本/资源来源时不填满字段。调用范围是声明、不是执行顺序/调用日志，缺失表示未提供。只有读到可靠运行报告后才可能展示绑定/调用核实，本片不实现它，不强制所有合法Host接入报告接口。

普通处理由controller完成：文档补丁首尝试因完整行定位不足被工具拒绝，读取已知原句后修正；未修改错正文。格式差异属于新代码书写修正，原失败保存。当前无新架构取舍；GUI/监控/导出/反向编辑/模块管理等不自动扩面，原冻结项和历史产品验收边界保持。
