# CI 三层边界与首片集成交接 · 2026-10-12

**状态**：Locally verified；适用文档与受测实现身份核对通过，等待集成后的目标SHA正式CI，不提前记远端通过。
**基线**：干净开发HEAD `3dd2e934c9d3dbbe47ad04a53ca7fdd2ed67e4ac`；main及实际远端 `24f62303e9bdea37061fe44acc7a0830dc519f98`。controller实施与语义自查，`independent=false`。
**范围**：[架构三层SSOT](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)、[CI执行归属SSOT](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-responsibility-map)及[有限集成计划](../ROUND-02-PLAN.md#ci-三层边界确认与关联切片集成2026-10-12)；本报告记录事实，不另定义架构。

## 为什么做与本片差量

维护者采用“纯净Kernel / 特化运行工程的Host / 基于该工程的发行版”组织后续工作。已有方向与它一致，本轮把该口径落到现有主图：工程组合可以称特化内核，公共职责仍不受模型/产品策略扩张。共享消费者在小Kernel外，自有Host合法接入保留。例子是Memory合同、Host材料绑定、ChatPro会话呈现分别负责，不要求所有发行版复制ChatPro。

本轮只有六个Markdown路径；公共Rust、workflow/catalog、已有npm检查、锁文件和全部原测试不改。5份当前原件和19项未写摘要保全于本工作树ignored `.cursor/plans/ci-three-layer-integration-20261012-r0/`；PLAN/DCL历史字节前缀保持。写集是MODULE_MAP、CI设计中英、PLAN、DCL和本Wave。

## 关联切片与本地证据复用

| 本地提交 | 责任与证据 |
|---|---|
| `a74e249c` | 首片责任准备；纯文档。 |
| `2b8d03ae` | 有限 `check:kernel-base` 开发反馈，真实四步及失败关闭。 |
| `c166af79` | 分类基线、主CI执行策略与迁移序列。 |
| `5e483aa6` | 架构边界/当前入口与原13个验证job责任对照。 |
| `3dd2e934` | 完整公共检查执行所有权；新Linux/Windows组、gate、trusted旧目录回退、完整目标集合和控制面回归。 |

受测实现的原始命令、结果和失败位于[所有权首片报告](WAVE-20261011-CI-CONTRACT-OWNER.md)。真正执行的公共完整入口为77单测/集成＋28doctest，原完整本地 `check:ci-local` G24c native exit0（2084.39秒）；前置缺资源失败和debug bundle准备独立保留。新文档没有改变受测源、命令、依赖或控制面，因此不再重复该完整链、Rust/业务/额外workspace doctest。文档、编码、历史保全及组合差量仍按实际影响检查。

本地scope核对和Node注入不替代真正Cargo输出；影子21场景也不替代真实Compare。新工作流全量配置19个实际job，旧main为17，正式两平台结果以最终SHA run终态判定。K-CI-IMPACT-01继续In progress，不将本片成功变成全部CI分层完成。

本轮G1–G9各自保存native exit0：默认/六路径链接、docs-only旧路径、登记、镜像、编码、债结构、5份原件/19项不变摘要/历史前缀/六路径与21路径合批集合、diff检查。新图沿既有公共依赖、自有Host和共享支撑关系，没有新增执行顺序、API或运行权力；旧报告的历史状态不回写。

## 集成与后续停止线

五个关联切片加本轮文档闭环在适用本地检查通过后冻结，复核main/远端身份，再FF和一次push。正式CI原始run/ci-gate和SHA在最终交付或本机证据中披露；不为绿灯修改本报告再造未验HEAD。远端失败读真实job首因定点处理，不无变化重跑。其他支线与冻结项保持。

下一片只看当前Host/发行版的直接检查入口、资源前置和目标集合，上限60分钟或4组。能提出覆盖不减的一片就停止；不穷尽逐test归属。没有明确表达障碍不改公共契约，未判明feature/平台或新增选择权限的拆分先记待决，普通MIXED继续保留。S01、真实TTS/平台、全部崩溃窗口和独立浏览器的历史边界均不扩大。
