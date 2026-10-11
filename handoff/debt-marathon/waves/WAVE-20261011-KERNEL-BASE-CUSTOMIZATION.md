# Kernel 基础工程：问答与模板自定义

状态：**Locally verified**，在原独立分支保存本地提交；controller 实施与语义自查，`independent=false`。未合入/推送main，未启动本片远端CI。

## 1. 范围、原因与实际选择

维护者确认基础工程仍不带Host；“关闭记忆”在本片仅表示不生成Memory教学内容，公共六槽保留，运行绑定由开发者负责。起点 `615a97a72472f11e90153d7fc40dcc2f62e543e2`、main `2b0390cdecaeed428f9ea1f753098f4aa69f3ebd` 均干净，复用原独立工作树。施工计划和19份原件先保存于本机忽略的 `.cursor/plans/kernel-base-customization-20261011-r0/`。

例如作者要从自己的Emotion实现开始，不必先删除Memory测试：选 `--base-example none` 即生成空白contracts/types工程；仍能导入Memory等全部Base。需要自己的目录和教学文件时，复制作者模板例子、编辑文件映射/变量和源文件，再显式刷新摘要及来源锁。这些动作不要求改变公共Base或准备Host。

## 2. 交付

- `init --template kernel-base` 的问答补收源码根、`memory|none`、作者/许可证/描述、输出目录；既有最终确认与取消保留。问答和脚本化生成共用配置/文件选择，不新增问卷框架。
- `--base-example memory` 默认保持五文件；`none` 省去 `tests/base_contracts.rs`，四文件。dry-run与实盘同一文件集；运行配方不能误用该选项，基础工程也不接受Host运行参数。
- [作者模板例子](../../../examples/scaffold-kernel-base-custom/README.md)只依赖现有Stage 2B。模板四源文件、字符串变量及作者显式摘要辅助脚本；辅助脚本不被生成器执行，不承担schema校验或事务发布。摘要按字节核验，`.gitattributes`仅对本例package固定LF。
- CLI指南中英、Scaffold RFC中英、crate及生成README同步选择与边界；原常用指令分类保持。入口见[CLI指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start)。

精确28路径中，生产修改限CLI内部配置/生成/问答；新数据包为作者示例。公共trait/DTO/re-export、参考Host、内核运行逻辑、Cargo依赖/锁、已有五配方、工作流、技术债状态均未改。计划/DCL历史仅追加。

## 3. 本地验证

| 验证 | 实际结果与含义 |
|---|---|
| 先红后绿 | `T0-blank-before-implementation` native101：测试可编译，旧CLI不认识 `--base-example`，没有生成工程。不是公共契约缺失或编译红。实现后同调用通过 |
| 最终CLI回归 | `T5-cli-final-after-option-guard`：`cargo test --locked --offline -p oclive-cli --bin oclive-cli --test e2e_kernel_base --test e2e_init_templates --test scaffold_contract -- --test-threads=1 --nocapture` native0；100项CLI单元＋7项原模板＋7项基础E2E＋5项Scaffold合同均通过 |
| 新直接用例 | 空白预览四文件/零写入、真实生成无tests目录、全部六trait作为 `dyn` 导入后Cargo通过；无效示例/误用运行配方/quick/Host参数拒绝。作者包真实锁定、预览、生成及Cargo检查；源改动不更新摘要拒绝，摘要更新而未重锁也拒绝，重锁后新变量/作者说明均可观察 |
| 同实现输入收集 | 2项单元纳入100项，核问答输入和显式CLI参数配置相同、已设选项跳过提问、坏答案拒绝；数据源注入不能代替终端证据 |
| 真实Windows PTY | `TTY1-generate` native0：显式源码/工程名/输出，终端选择none、空作者、MIT、空描述并确认，实盘四文件；`TTY2-cancel` native0：实际询问工程名/源码/示例/元数据/输出，最后选no，目标目录absent。原始PowerShell transcript及退出件保留 |
| 生成工程常用动作 | 作者包 `A7/A8/A9`、问答工程 `TTY3/TTY4/TTY5` 的 `cargo check/test --offline` 与 `cargo fmt -- --check` 全native0，空白工程实际0项测试。验证时仅复制仓库Cargo.lock作离线依赖种子，不是生成器产物 |
| 作者文档实际操作 | `A2–A6` 检查/查看/锁定/预览/生成全0；修改模板后显式运行本例Node辅助脚本，`A10–A14` 更新摘要/检查/查看/重新锁定/再生成全0。`module_goal` 从Memory改Event，新作者说明出现，预览零写入，隔离OCLIVE_HOME未创建 |
| Git检出摘要 | `S2-windows-checkout-digests.json`：从索引以 `core.autocrlf=true checkout-index` 导出包6文件，instruction摘要及4个source摘要均匹配；6路径的eol属性全部LF。这是本机Git字节验证，不是Linux/macOS实机验收 |
| 静态与边界 | 最终 `R2-cli-clippy-final`（all-targets、`-D warnings`）native0，`F3-fmt-final` native0；分层3 imports/1 FQ保持，module-compat 10slots/9manifests/7UI及6plugins通过，Node辅助脚本语法通过 |
| 文档与保全 | 默认52份＋改文7份链接、docs-only旧路径、镜像、登记、债结构通过；28路径UTF-8无BOM、19原件hash、PLAN/DCL两份历史字节前缀原样、精确写集及main身份检查通过，最终diff检查通过 |

终端两次使用T4构建；之后只新增新参数的 `requires=template` 解析约束及对应拒绝用例，T5在最终源码重验。TTY实际请求均已传template，收集/确认/生成正文逐字未变，沿用有身份的终端事实，不冒充再次实跑。

本片不改公开trait/DTO/re-export，未重复workspace doctest、Host/Rust全workspace或完整远端CI；原基础片的公开面验证和原消费者证据保留其原提交身份，不给本片新SHA借绿灯。最终CLI回归内Memory教学练习有意产出“1 passed / 1 failed”，然后按明确功能约定更新期望回到2passed；父E2E通过，不能把日志里的故意失败说成整轮无错误。Git换行提示与编译stderr也按原件保留。

### 自主修正与真实失败

- 先红测试实际拒绝新参数；首次fmt检查1后仅格式化本片Rust，新旧日志分别保存。
- 语义自查发现无template时 `--base-example` 会被旧quick分支忽略；新增解析约束并加真实拒绝调用，不改原quick行为，不扩大CLI协议。
- 作者包Rust E2E先验证过Node辅助脚本，随后改为Rust fixture准备摘要，避免Rust-only CI需要Node；最终E2E独立于Node，作者辅助脚本另走实际文档操作。
- 文档walkthrough脚本在全部A2–A14成功之后，其本机附加检查错误假设非Force枚举会隐藏 `.gitignore`，导致脚本exit1。这是controller检查脚本错误，产物本来四项正确；修正为实际四项/无tests检查后只接续TTY Cargo动作，不覆写原CLI成功记录、不重造模板。该本机脚本不进交付写集。
- `G3-diff-check` 首次2：新增属性行的CRLF被视为尾随空白；修正新增行换行后diff0。旧属性规则语义保持，索引对属性文件的LF规范化如实保留。没有改SQL/其它文件的历史校验字节。

证据均在本机上述忽略目录，包含失败/续验stdout、stderr及立即捕获native码。提交前冻结本片文件hash与报告；原件、历史和main最终状态另作复读。此报告自身不含自指SHA，提交身份由Git和交回维护者的结果确认。

## 4. 停止线和接手边界

问答生成与取消、作者模板修改后再生成的第一条路径成立后冻结本地提交，不继续补五套同形模块或扩大Host验证。空白工程执行零项测试不代表模块功能通过；Memory教学约定不是强制Base协议；自己的文件/字符串变量不等于运行时六槽开关。条件/循环/include/自动继承/组合/网络安装/第三方执行仍未实现。

main和其它工作树保持；本片无merge/push或新远端CI。当前用户架构范围无新歧义；后续Host预设、SDK、CI分层按具体目标另行推进。签名/TLS、双核、Full韧性、多Agent、高级情绪记忆、Production Stream等冻结保持，S01/真实音频/平台/崩溃窗口/独立浏览器结论不扩大。
