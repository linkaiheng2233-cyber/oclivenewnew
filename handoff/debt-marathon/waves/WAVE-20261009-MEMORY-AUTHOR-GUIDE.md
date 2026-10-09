# 路线一接续：Memory 实现者的人类入口（2026-10-09）

**状态**：Locally verified。起点 `51f8f26b184b7c13d55756667909b9bd7e492f19`；M / 轻，controller 实施和语义自查，`independent=false`。本报告只记录本片的修改、证据与接手；当前模块定义仍归 [MODULE_MAP](../../MODULE_MAP_AND_HANDOFF.md)，债务状态仍归主台账。

## 问题与交付

维护者选择的受众是模块 / 插槽实现者，第一片已用现有契约完成外部 Memory 的消费者调用。现有 [人类 Memory 开工包](../../../human-docs/modules/slots/memory.md)却只介绍丰富 `MemoryRetrieval`、蓝图后端、STM/LTM 与回想流程；这会使新实现者误以为基础检索也要接入这些参考 Host 设施。

本片将已有案例放到该入口前部，并在 [英文摘要](../../../human-docs-en/modules/slots/memory.md)同步同一条路径。读者先选择基础实现或参考 Host 维护，再找到作者实现、已有消费者绑定、原运行 / 测试命令和结果判读。§1–§6 原丰富路径保持，写入与编排明确归参考 Host，不借文档重设 Kernel 权力。

例如，只想从本次材料中检索最近两条匹配文本的作者，可以先运行已有 `external_memory_base`，再替换作者自己的算法；不需要先修改 SQLite 或蓝图。`literal:` 仍是这个实现的约定，不是 Base 查询标准。案例调用成功不代表找到所有事实，也不意味着自定义实现已注册为参考 Host 后端；立即就绪 helper 不承诺任意异步 executor。

## 写集与证据复用

写集仅五份 Markdown：中英 Memory 开工包、本报告、原 ROUND 计划追加和 DCL-79 追加；不新增顶层 SSOT。两份指南负责阅读路由和操作 checklist，公开合同链接 [现有接入点清单](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，不复制六槽定义表。

原 [Memory Wave](WAVE-20261009-EXTERNAL-MEMORY.md)记录 `cargo run` 的实际输出与三项案例测试；[合流报告](WAVE-20261009-AI-GUARDRAILS-FOLLOWUP.md)另记录三项定向回归及导入文件 Git 内容核对。原正式 CI [37887351660](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37887351660)仅保持 `f94069ddd7b26e7661c81dd310175c0b6fc9e857` 的身份，不改绑本片。文档命令和输出与原 source / 回执对照；本片没有再执行 Rust / 模型 / 产品场景。

原 Kernel / runtime / Host / Tauri / 前端 / example / 测试 / 依赖 / 锁 / 脚本与 hard gate 保持，旧 DCL 与 ROUND 正文保留；中英路由、命令、输出、资源与未承诺项做语义自查，不把镜像工具 PASS 当作翻译准确性证明。台账和队列没有状态迁移。

## 适用验证与收口

原始 stdout / stderr / native exit 保存在本机工作树忽略目录 `.cursor/plans/memory-author-guide-20261009-r0/`，不随 Git 自动转让。适用检查均 exit 0：默认链接 52 份、五改文显式链接、docs-only 旧路径、镜像、登记、债结构（147 行 / 12 计划，非债务状态判定）、五份编码及 diff。精确写集、旧计划与 DCL 前缀、DCL 编号唯一性、中英旧 Host 正文保全、原运行与测试回执对照共六项核对通过；两条路径、失败、资源与边界另由 controller 语义自查。纯文档入口不触发 workspace doctest、Dimension 5 或全量 CI，已有证据不冒充新 SHA 正式通过。

首次回执对照 exit 1，暴露了文档命令缺少原执行的 `--offline`，以及临时核对脚本误把 Windows 回执中的 `cargo.exe` 写成 `cargo`。两处已修正，并补充依赖缓存前提；原失败与修正后通过的输出分别保存，没有改动 Rust 或重跑业务。LF→CRLF 提示不作零 stderr 声明。

达到读者可选择正确路径并运行现有案例即停止。适用门禁通过后本地提交；主树仍为起点且干净才安全 FF，并可按既有授权备份无 PR 分支，不推远端 main。没有新的架构问题，也没有新增公共接口、配置式替换、独立发布包、持久化或六槽固定顺序。

后续实现片须有具体模块 / Host 对象或实际受阻动作；本片不为了延长调查再补其余五槽。既定冻结、H04 有限验收和 S01 / 真实音频 / 平台 / 崩溃 / 浏览器边界保持；执行结论与债务状态分别报告。
