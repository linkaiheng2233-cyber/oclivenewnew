# 独立公共库远端交付与原工程固定来源

日期：2026-10-14；controller实施与语义自查，`independent=false`。原工程迁移状态：**Locally verified，固定Git来源的适用本地验收组合通过；正式目标SHA CI另验**。不将独立库CI代替原工程验收或主树合入。

## 两份交付的归属

维护者提供并纠正目标为[独立仓库](https://github.com/linkaiheng2233-cyber/oclive-kernel)。新仓main固定 `60c6cc0bb911bf8e15efd4352f00561da6a6ee4f`，源码提取标签 `checkpoint/kernel-base-extraction-20261014-r0` 指向历史 `605914cfb48c977db1e24675e039b8b198857b55`，分发标签 `checkpoint/kernel-base-source-20261014-r0` 指向60c6cc0b，均精确推送；没有创建仓库或写入先前小号仓库。

独立库只定义六槽Base、请求/失败和最小逻辑角色校验；最简Host有自己的私有Memory/Prompt/确定性LLM实现与执行器。没有SQL、HTTP、Tauri、模型连接或共享runtime依赖。共享消费者继续在小Kernel之外；参考Host与发行版继续承担自己的运行和产品策略。有限接口不承诺新增统一权限/调度/终态执行层。权责只维护在[模块图](../../MODULE_MAP_AND_HANDOFF.md)，新库另有自己的边界说明。

原工程从[兼容准备](WAVE-20261014-KERNEL-BASE-COMPAT-PREPARATION.md) `e99feb72bc568e65c3113d4a603f714e63fd3d62` 继续：三个旧crate已保留旧路径、re-export相同Rust符号。本片把临时绝对path替换为新仓的完整Git rev；旧SDK冻结80ab8582、605914和本机准备checkpoint保持，不改旧报告。

## 独立库的验收

本次60c6cc0b只补仓库元数据、克隆入口与来源/迁移说明，公共库及Host源码、锁和workflow与605914逐字保持。

- `P01`普通workspace测试：11库单测、16Base fixture、5Host测试及5doctest通过。
- `P02`显式workspace doctest5通过；`P03`Clippy all-targets/Dwarnings、`P04`fmt、`P05`实际终端Host运行、`P06`本轮Markdown链接均exit0；21个已跟踪文本UTF-8核对通过。
- 精确SHA正式[CI 38540103718](https://github.com/linkaiheng2233-cyber/oclive-kernel/actions/runs/38540103718)：Windows/Linux两项verification均success。它只验独立库和示例，不代表真实模型、硬件或原工程迁移已过。

## 固定来源核实与门禁

`P09`实际从GitHub解析固定rev。metadata全图只有一个Base包ID，三个兼容crate都指向它；Git缓存中的src/tests与独立冻结库逐文件hash相等。相对准备锁，699包集合不变，仅Base包新增Git source字段，684个registry完整记录保持。`P10`缓存audit exit0（1308公告、699依赖、漏洞0、既有warning7），不改变ignore/TLS，也不冒充新公告数据库扫描。

失败原件保留：`P07 cargo update -p`未匹配尚未解析的新来源，native101；`P08`过早离线metadata因Git缓存尚无该rev而native101。随后`P09 metadata`允许一次Git源码获取并成功，锁只增来源字段；不是依赖升级或业务测试失败。

本机执行偏差另列：`P15`误写不存在的编码脚本名，native1；改用既有`check-doc-encoding.mjs`的`P17`通过。`P11`完整链在Dimension 5的Python注册表检查失败，其余30项通过/设计内skip；Windows默认`py`入口在本环境不可用。一次定向检查`P20`误填环境变量名也失败，按实际脚本的`OCLIVE_VOICE_PYTHON`指定便携解释器后`P21`通过。没有修改语音实现或判据，`P22`在修正的进程环境重跑完整链；各次日志独立保留，不以局部通过替代完整退出码。

`P19`固定Git来源下显式workspace doctest46项通过/native0。`P22`完整链的Dimension 5（31项）、lint/typecheck/build、Rust fmt/all-target/all-feature Clippy、workspace库测试及非CLI集成均通过；四项身份回归逐条ok。该命令最终在CLI阶段native101，**不标为check:ci-local命令通过**：全局`CARGO_TARGET_DIR`被生成工程继承，使三个monolith/bench测试无法在作者自己的target下取得已编译binary。源码不需修改；`P23`移除该环境覆盖，只给外层`cargo test --locked --offline --target-dir <shared> -p oclive-cli --tests -- --test-threads=1`显式指定缓存，重验CLI全测（含树外SDK）。前段已过证据复用，不为这个进程环境问题重复前端、Clippy及整个workspace链。

`P23` CLI全测最终native0，包括此前三个失败用例及真实树外SDK配方；作者工程在自己的target编译，正常调用与变异失败判据均实际执行。完整适用本地验收由`P22`已过前段与`P23`修正环境的CLI叶步骤组合成立，**原单命令native101保留，不冒称native0**。显式doctest独立由`P19`覆盖。本轮修改文档的链接/镜像/登记/编码/旧路径检查全部通过，最终文本变动再跑适用文档门禁。

冻结后只推一次精确分支及本批标签，在PR中取得原工程该SHA正式CI；不为每条检查单独触发远端矩阵，不为CI绿灯回写另造提交。

## 接手与限制

证据保存在本工作区 `.cursor/plans/kernel-base-remote-20261014-r0/`（私有实施日志，不是构建依赖）。原主树仍为08348f31，SDK PR与迁移交付状态分别管理；新仓绿灯不自动合入旧仓main，不关闭 `K-CORE-BOUNDARY-01` 或其它父债。

发布package仍关闭。普通Host可直接绑定Base，或按需使用共享消费者；Memory→Prompt→LLM仅是新仓示例的流程。没有引入注册/发现、默认模块、强制六槽全跑、新权限或重试。S01、真实音频/模型、平台资源与全部崩溃窗口等历史边界不扩大。源码和语义达到本片停止线后即冻结，不追加穷尽调查。
