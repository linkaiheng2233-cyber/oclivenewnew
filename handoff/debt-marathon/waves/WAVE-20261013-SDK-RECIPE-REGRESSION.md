# 源码 SDK Memory 配方自动回归

**SSOT范围 / 更新**：本片实际实现、验证与交接；2026-10-13。交付形态见[SDK RFC](../../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)，操作配方见[Memory指南](../../../examples/developer-sdk-memory/README.md)，不复制六槽或SDK接口表。
**身份 / 状态**：**Locally verified**。起点干净 `aebe7c466bfc316951529c2664bdd56083a82c23`，controller实施与语义自查 independent=false；按[七路径计划](../ROUND-02-PLAN.md#源码-sdk-memory-配方自动回归实施2026-10-13)完成实现及适用门禁后本地冻结，最终SHA/tag/clean以原生交付记录为准。原[准备记录](WAVE-20261013-SDK-READINESS.md)中尚未实施的文字保留为历史。

## 实现与实际价值

新 [e2e_source_sdk.rs](../../../kernel/crates/oclive-cli/tests/e2e_source_sdk.rs)复用 `tests/common` 与Cargo本次选择的CLI二进制，不调用缓存路径下未经对应构建的可执行文件。生成带空格路径的临时作者工程，明确 `--base-example none`，隔离CLI home；原Memory两源、项目README、许可证与根锁种子复制。Cargo显式添加runtime后，在独立workspace实际metadata/check/test/run。没有新生成器、公共门面或第二份教学实现。

工程断言核一个workspace成员、三个准确的直接依赖及其manifest确实来自本次checkout、无参考Host/Tauri/tao/wry及第三方name/version/source沿源码种子锁。派生锁不与根种子要求字节相等，而在后续 `--locked` 命令及负控前后保持。原binary三项行为和消费者第一行 `external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0`必须成立；空library/doctest不作为功能证据。

负控只把作者副本 `.take(self.max_results)` 改成 `.take(0)`，单独运行原选取行为测试；必须同时观察子命令非零、确切测试名 `FAILED`及 `0 passed; 1 failed`，编译或工具缺失不能满足断言。成功运行该负控后显式close临时目录。原canonical源码不改，界面声明不充当调用证据。现有组件足够，未为了“先红”删公共接口或伪造旧实现缺口。

例如共享runtime升级后，库内例子仍可能通过，而作者工程的依赖或组装步骤已经退化；这项回归检查的正是现有上手配方。作者的检索质量、持久化、性能、真实异步执行器及目标Host仍需其实际场景验收。

## 实际命令与证据

本机原始stdout/stderr/原生退出件在 `.cursor/plans/sdk-recipe-regression-20261013-r0/`；只在命令进程使用原Cargo target、jobs=1及offline并finally恢复，未安装或升级依赖。内层Cargo输出被测试读取并断言，不另造每个子命令的退出件；外层native成功与已运行的断言共同支撑本片结论。

| 记录 | 命令与结果 |
|---|---|
| G1 | `cargo test --locked -p oclive-cli --test e2e_source_sdk -- --test-threads=1 --nocapture`：exit0、外层1 passed，41.17秒含构建；内层三项行为、实际run及预期失败负控均通过断言 |
| G2 | CLI bin＋doctor_wiring＋e2e_kernel_base串行：exit0，100/8/7 passed |
| G3 | CLI `clippy --locked --all-targets -- -D warnings`：exit0 |
| G4/G5 | runtime原Memory example：测试3 passed，实际run为selected=2/baseline=3/memory_calls=1/llm_calls=0，均exit0 |
| G6 | `cargo fmt --all -- --check`：exit0；R1初次exit1仅新测试一处换行，F1仅格式化新测试后通过，原日志未覆盖 |
| G7–G9 | 分层3 imports/1 FQ ref、module-compat和插件索引，均exit0 |
| G10–G12 | 语义自查补三份直接依赖manifest来源核对后，只复跑受影响SDK目标、该目标Clippy及fmt，全部exit0；最终SDK外层1 passed，4.16秒含构建。原G1保存为加来源断言前的通过，不代替最终字节结果 |
| G13–G20 | 默认及六文链接、镜像、登记、docs＋code旧路径、债结构、严格编码/七路径/五原件/历史前缀/原源码及旧tag保全、diff，全部exit0 |

最终报告文字只复查受影响链接、范围与暂存diff，然后提交/核clean，不重复已通过的代码检查。新目标由Cargo默认集成测试发现，现有 `check:rust:integration:cli` 与主工作流的CLI串行测试命令可选择它；本片没有改job、影响选择器或required gate，不能把此归属核对说成已经执行远端CI。

## 收口与边界

新增一个集成测试并更新六份文档/记录，适用门禁通过后本地提交、新增 `checkpoint/source-sdk-recipe-20261013-r0`；旧ready/reviewfix/上手checkpoint及main保持，不push或改变技术债状态。测试及文档无公开Rust变化，所以不额外workspace doctest；也未改门禁组合，不重复Dimension5、完整发行版链或远端矩阵。原失败只属执行者新测试格式，未确认新产品缺陷。

本片达到一条源码接入配方可重复回归的停止线，当前无须新增架构决策。证据限Windows与既备缓存，未验其它平台、新机器或注册表SDK发布；未启动Host/模型/真实网络/语音，不改变小Kernel与共享消费者边界。已定冻结及真实产品、平台、崩溃等历史未覆盖项保持。
