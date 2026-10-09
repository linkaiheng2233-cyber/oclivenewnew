# 路线一第一片：外部 Memory Base 接入

**SSOT 范围**：本片实施、验证和接手记录；模块关系只链接 MODULE_MAP，当前债务状态仍由台账维护。
**最后更新**：2026-10-09。**实施 / 语义自查**：controller，`independent=false`。

## 目标、结果与停止线

受众是外部 Rust 模块实现者或新 Host 开发者，当前槽位是 Memory。一页 [接入点清单](../../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)先定位六槽的公共请求 / 返回 / 共同失败、资源绑定、参考实现、消费者与当前参考 Host 的“有 / 无”接线；不把六种签名当成六阶段流程，也不把早期 B1 说明当成当前 Host 未接线证明。

本片的 [example crate](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs) 与 runtime 库分开编译，只使用 public imports。作者自己的 [Memory 实现](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs) 经现有 `MinimalRoleBaseConsumer` 真实调用成功，runtime 库、公共契约和参考 Host 都没有改动。不是“仓库外发布的独立软件包”；没有新增 workspace member、依赖或锁文件。满足这一范围后停止，不补其余五槽或发行版接入。

为什么选这个例子：原 `KeywordMemoryBase` 会返回所有字面匹配，新实现按调用方声明的材料顺序只取最近 N 条。两者接同一个公共接口，但各自的检索要求须遵守各实现的明确约定，不能假定任意 query 语义自动兼容。例子的 `literal:<exact substring>` 是本实现的约定，不是 Kernel 新协议。

输入中故意包含主体、否定、引述、不确定性与 CRLF。新实现只选原文，不总结 / 改写，不读取隐含数据库或保留旧候选。共享消费者不重试或补默认值；未支持要求保留完整 `BaseCallError`。其它引用复用已有参考实现，LLM 仅为拒绝生成的私有夹具，本片没有模型、音频、监听器或用户库。

## 复现与已取得证据

cwd 为仓库根；所有 Cargo 命令使用现有锁文件。

```powershell
cargo run --locked -p oclive_kernel_runtime --example external_memory_base
cargo test --locked -p oclive_kernel_runtime --example external_memory_base
cargo test --locked -p oclive_kernel_runtime --test minimal_role_six_slots
```

| 实际执行 | 结果与边界 |
|---|---|
| 编译红：只写 main，尚无案例私有 `memory.rs` | exit 101，唯一 E0583；不证明旧公共 API 缺陷，也不为制造红灯更改公共符号 |
| 新案例测试 | exit 0，3 passed；替换与原文保全、空结果 / 完整失败、当前调用材料范围；通过真实消费者执行，不是只测算法本体 |
| 实际 `cargo run` | exit 0，`selected=2 baseline=3 memory_calls=1 llm_calls=0`；两者使用同材料、各自的检索约定 |
| 既有 `minimal_role_six_slots` | exit 0，5 passed；独立调用、完整错误、借用 / pending、无隐含调度和原替换案例保持 |
| 分层 / module-compat | exit 0；分层 ratchet 未提高，module-compat 输出与现有能力集吻合 |
| Host `--lib minimal_` | exit 0，4 passed / 633 filtered；只是有限相关回归，不冒充 Host 全测 |
| runtime Clippy `--all-targets -D warnings` / workspace fmt | 均 exit 0，无 Clippy 诊断；包含新的 example target |
| 显式 workspace `--doc` | 资源准备后 exit 0，10 个库文档测试组共 51 passed / 0 failed / 0 ignored；输出包含 runtime / Host / 桌面。wasm cdylib 提示不支持 doctest，不能宣称所有产物都覆盖 |
| 文档门禁 | 默认入口、四份改文和七份合入文档的显式链接 / docs-only 旧路径 / 镜像 / 登记 / 债结构均 exit 0；结构 PASS 不改变债务状态 |

本机原始 stdout / stderr / native 退出回执在隔离工作区的 `.cursor/plans/external-memory-base-20261009-r0/`，忽略产物不随 Git 自动转让；上面的命令和源码是可转让复现入口。本文件冻结前记录定向出口；主题里程碑随后在不改源码的条件下执行一次 `npm run check:ci-local`，正式远端结果绑定最终完整 SHA，回传报告，不另做绿灯回写提交。不能以父 SHA 或这张表宣称新 SHA 的完整链已经执行。

## 基线、范围和自主处置

- 文档分支 `894047c52780375c9c01d596e442ff59231c596d` 的七份 Markdown 已按授权 FF 到本地主仓 main，尚未因此推送或新建文档 CI；原代码点 `2ef5af05` 正式 CI 保持其自己的身份。本片基于 894 开工。
- 原审查工作区有一处与本片无关的文档空白编辑，原样保留。App 工作树工具因会话 cwd 是 Git 仓库父目录而拒绝，改用 Git 建立新的隔离工作区；不还原、暂存或提交那处外部修改。
- 先红验证初次还暴露了我误用不存在的 `BaseCallError::new`，修为既有结构体字段后重验，唯一 E0583 红证据另存；不把这个实施错误解释成契约缺口。安装命令初次数组参数串成一个参数也已修正，两个失败原输出保留。
- 离线 `npm ci` 成功，deprecated 提示如实保留；不把离线缓存审计摘要当一次实时漏洞库查询。未更改任何依赖清单 / 锁文件。
- 显式 doctest 首次因新工作区未准备 Tauri 资源失败；按仓库已有 debug bundle 步骤生成隔离构建资源后通过。属于构建前置缺口，无需更改契约或降低门禁。一次文档检查用错既有黄金路径文件名，改为从 894 的真实七路径清单取参数后通过；原失败日志保留。
- 六路径写集：MODULE_MAP、本计划 / DCL、当前 wave、两个 example Rust 文件；旧记录只保全，主债台账 / 队列不改。无公共契约变更，无需维护者裁定新权限、调度或强制义务。

## 未承诺项

本例的轮询 helper 只驱动立即就绪的内存实现，不是任意异步 executor；无独立插件发现 / 配置式 Host 替换、持久记忆、跨发行版、真实模型或性能结论。使用一个可选六引用消费者不要求每轮六槽全跑。既有 H04 限定产品验收、S01 历史 FAIL、H03 真实音频 / TTS、平台实机、所有崩溃窗口和独立浏览器形态均不受本例改变。签名 / TLS / 双核 / Full 韧性 / 多 Agent / 高级情绪记忆 / Production Stream / unified remote wire / CI 分层不解冻。
