# K-LLM-ENV-02：DB 读取交错的隔离回归

**范围**：基线 `19293fe8c66dbbdccb6f9d823d22736e16bb67a6`；按[第二轮计划](../ROUND-02-PLAN.md#k-llm-env-02--db-读取交错的隔离回归2026-10-02)只新增 `oclive_kernel_host` 集成测试与本轮证据记录。生产锁、DB、缓存、环境应用实现均未修改。当前债务状态仍见[主台账](../../TECHNICAL_DEBT_INVENTORY.md)。

## 为什么需要这条测试

旧风险是先读到旧 DB 快照的调用被卡住，新设置已写入并开始应用后，旧调用最后才写进进程环境，把新设置覆盖。已有修复用进程级异步锁串行化整个 DB→env 事务；此前单测只检验应用版本与 dirty 位的两个结果，没有强迫两个真实应用调用在 DB 读取处交错。

新 `user_llm_env_transaction.rs` 是单测试用例的独立测试进程。内存 `AppSettingsPort` 捕获旧 `OLLAMA_BASE_URL` 后阻塞第一次读取；随后测试把值从 `http://127.0.0.1:11111` 改为 `...:22222`，让 32 个新调用全部越过启动屏障并报告已尝试进入应用。旧读取释放前，DB 读取次数必须仍为 1；释放后 33 次调用都完成，最终 `OLLAMA_BASE_URL` 必须为新值，backend 为 `ollama`。测试开始时保存该进程会改动的七个环境变量，结束时恢复。没有真实模型、网络、凭据或用户数据库。

这个例子直接区分“锁覆盖整个读取与写入”与“只锁最后写环境”：后者允许 32 个新调用先读／应用新值，再由迟到的旧调用覆盖，读取次数和最终值断言至少一处失败。该测试不复制产品实现，而是调用公开的 `apply_user_llm_env_from_db`。

## 验证与边界

- 定向集成测试最终字节连续 5 次 exit 0，每次 1 passed；`cargo clippy --locked -p oclive_kernel_host --test user_llm_env_transaction -- -D warnings` exit 0；`cargo fmt --all -- --check` exit 0。第一次格式检查只发现新测试的三处换行，定点 rustfmt 后通过，失败事实保留在本记录。
- `cargo test --locked -p oclive_kernel_host --lib -- --test-threads=1`：634 passed / 0 failed。分层、文档检查和 `npm run check:ci-local` 均 exit 0；综合链含 Dimension 5、前端 lint/typecheck/build、Rust fmt/Clippy/lib、workspace 与 CLI 集成。新目标提交的远端 CI 另核，不以前一提交结果替代。
- 这是同一进程内、内存设置替身的可控交错，不是完整 `AppState` 的 version/dirty 冲突实验，更不是 save/chat/theater/canonical sync、跨进程、真实 DB 或真实 provider 压力。K-LLM-ENV-02 保持未结案；后续如扩大验证，只补已明确的剩余路径，不无界穷尽低概率交错。

## 2026-10-03 · 完整 AppState 的内存 SQLite 刷新边界

基于 `b912237324fd9521968084cbf25f6c7dd267cc6e` 的干净基线，新增独立测试进程 `user_llm_env_state_refresh.rs`。它使用真实内存 SQLite 和完整 `AppState`，仅注入不触发模型 I/O 的 `MockLlmClient`，角色目录位于临时目录。先写入本地 provider 与旧 URL，通过生产 `reload_llm_user_env_impl` 刷新；再直接把 DB 改为新 URL，调用 `apply_user_llm_env` 时因版本未标脏，环境仍是旧 URL；显式 `mark_user_llm_env_dirty` 后再次调用才得到新 URL。七个相关进程环境变量由测试保存和恢复，没有用户数据库、真实模型或网络调用。

**原因与例子**：DB 中的新设置不自动等于“已应用到进程环境”。若某调用路径直接写库而漏掉标脏/刷新，旧环境会继续生效；该负例把这个责任边界变成可观察测试。它只覆盖真实内存 DB 和公开刷新入口，不证明 file-backed 持久库、并发版本竞争、save/chat/theater/canonical sync 的每个调用者均正确标脏，也不补长时间压力。

**本地验证**：定向测试 1 passed / exit 0；Host lib 634 passed / exit 0；定向 Clippy `-D warnings` exit 0。首次 fmt 检查仅发现新测试一处链式调用换行，定点 rustfmt 后重新验证。`npm run check:ci-local` exit 0，包含新增集成测试及工作区、CLI 测试；目标提交的远端 CI 另按 SHA 核验。父债保持未结案。

## 2026-10-03 · 保存设置主路径的标脏责任

在上一切片的真实内存 SQLite／完整 `AppState` 测试末尾加入合成角色，用生产 `save_llm_user_settings_impl` 保存第三个 Ollama Base URL。调用成功后，DB 与 `OLLAMA_BASE_URL` 都是第三个值。与前一阶段“只改 DB 而未标脏，旧环境继续生效”的负例构成对照：保存设置入口确实履行标脏和应用责任，而任意直接写库不能因此自动获得相同保证。测试仍使用 `MockLlmClient`，没有真实模型、网络或用户库。

**范围与止点**：定向测试 1 passed / exit 0；fmt、定向 Clippy 和 `npm run check:ci-local` 均 exit 0，综合链含工作区与 CLI 集成测试。只证明本地 provider 的保存设置主路径，不证明云端 token、LoRA、保存失败的事务性，也不外推 chat/theater/canonical sync、持久 DB 或完整 AppState 并发交错。目标 SHA 的远端结果另核，父债保持未结案。

## 2026-10-03 · 对话主入口应用待刷新 DB 值

在同一隔离测试中，保存设置的第三个 URL 应用完成后，直接向内存 DB 写入第四个 URL 并标脏，再调用生产 `process_message` 走合成角色和 `MockLlmClient` 的正常对话。模拟回复成功返回，进程 `OLLAMA_BASE_URL` 同时成为第四个值，证明 chat 主入口在本场景生成前消费了待应用配置。这与“不标脏时直接写库仍沿用旧环境”的负例相邻，便于区分调用者责任。

**边界**：定向测试 1 passed / exit 0；第一次 fmt 仅新增 import 顺序不符，定点修正后通过；定向 Clippy、`npm run check:ci-local` 均 exit 0，后者覆盖工作区与 CLI 集成。目标远端 CI 另核。该用例不证明流式/断流、真实 provider、Theater/canonical sync、持久 DB、并发版本或跨进程环境一致性，K-LLM-ENV-02 仍未结案。

## 2026-10-03 · Theater 主入口消费待刷新 DB 值

在同一独立测试进程中，为已有合成角色构造有效 Theater 场景；chat 第四个 URL 验证后，把内存 SQLite 值改成第五个 URL 并标脏。模拟 LLM 的 `generate_tag` 记录**被调用时**的 `OLLAMA_BASE_URL`，返回可解析的结构化场景。生产 `generate_scene` 返回生成后的台词，且本次新增的标签调用恰好一次、调用时 URL 为第五个值。这比只在函数返回后检查环境更能确认 Theater 的这条生成路径在调用 LLM 前消费了待应用配置。角色和 DB 都在测试临时环境，未连真实模型或网络。

**本地证据与边界**：最终字节的 `user_llm_env_state_refresh` 定向测试 1 passed，定向 Clippy `-D warnings`、fmt、分层检查均 exit 0。第一次 fmt 仅发现新测试两行链式调用缩进，定点修正后通过。与下节同主题切片合批运行的 `npm run check:ci-local` 最终 exit 0，包含 Dimension 5、前端、Rust 工作区与 CLI 集成；目标提交的远端 CI 尚待按 SHA 核验。只证明内存 DB、本地 provider、有效场景和模拟生成器的 Theater 路径；不证明真实 provider、canonical sync、完整 AppState 并发交错、持久 DB、流式/断流或进程级压力。K-LLM-ENV-02 保持未结案。

## 2026-10-03 · 桌面 canonical seed 的隔离文件库回归

新增单用例独立进程 `canonical_llm_env_sync.rs`，把 `OCLIVE_APP_DATA` 指向临时目录；其中的 `app.db` 只建立与本路径相关的最小 `app_settings` 表并写入 local provider 与新 URL。内存 UI shell 先通过生产 reload 应用旧 URL；调用生产 `seed_shell_llm_from_canonical` 后，shell DB 与进程 `OLLAMA_BASE_URL` 均成为 canonical 新值，backend 仍为 `ollama`。文件库 pool 显式关闭，环境原值由 RAII 恢复；无用户数据库、真实模型或网络。

**本地证据与边界**：定向测试 1 passed，fmt、定向 Clippy `-D warnings`、分层均 exit 0。首轮综合链在 Dimension 5 的旧目录名规则处止步：新测试夹具用了 `.join("roles")`；改为仅测试用的 `role-fixtures` 后，定向测试与 stale-paths 检查通过，第二轮完整 `npm run check:ci-local` exit 0。失败只涉及夹具路径名，没有放宽规则或修改生产行为。该测试证明这条 local-provider seed 路径会标脏并应用复制来的值；它没有执行完整迁移表、真实桌面启动、反向 shell→canonical 写入、云端 token、并发写入或失败恢复。目标提交远端 CI 尚待按 SHA 核验；K-LLM-ENV-02 不升 Done。
