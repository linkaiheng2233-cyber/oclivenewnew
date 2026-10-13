# 我的 Memory 消费工程 / My Memory consumer project

本工程由 OCLive `kernel-base --base-example none` 生成，再显式添加共享 runtime，复制现有 Memory 示例。它拥有独立 workspace 和锁；不启动参考 Host、桌面、模型或回合调度。

This foundation project explicitly adds OCLive runtime and the canonical Memory example. It has its own workspace/lock and starts no Host, desktop, model or turn scheduler.

先运行 / Run first:

```sh
cargo check --locked
cargo test --locked
cargo run --locked
```

初始案例预期 binary 测试 3 项通过；运行第一行 `external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0`。空白 library/doctest 的零项不表示功能通过；有意修改行为后按自己的承诺调整对应断言。

The original example passes three binary tests and prints the first output line above. Zero tests in the blank library/doctest are not functional evidence. Intentional behavior changes need corresponding expectations based on your own promises.

先改 `src/memory.rs` 中自己的 Memory 实现；材料、查询、消费者绑定和对应断言在 `src/main.rs`。示例 `literal:` 语法是作者约定；实际异步实现需要作者提供执行器。本路径只调用 Memory，不规定六槽回合顺序。

Edit your implementation in `src/memory.rs`; caller inputs, consumer bindings and assertions are in `src/main.rs`. Literal-query syntax belongs to the author. Real async implementations need the caller's executor; this example calls only Memory.

完整步骤与边界位于所用 OCLive 源码 checkout 的 `examples/developer-sdk-memory/README.md`。记录该 checkout 的 SHA/dirty 状态与 Rust/Cargo 版本，保留当前派生 Cargo.lock；不要用源码根锁覆盖自己的既有锁。移动目录后核对 Cargo.toml 的所有 path。

The full guide is `examples/developer-sdk-memory/README.md` in your OCLive checkout. Record source/tool inputs, keep the derived lock and recheck paths after relocation. Do not overwrite an existing project lock with the source workspace lock.

该指南第5节提供“两条改一条”的作者副本练习，并定位 `retrieve` 的消费者绑定、`main` 的输入及 `tests` 的断言。第6节区分官方配方回归、自己的模块测试与目标Host验收：在本工程运行Cargo只检自己的实现，不自动让目标Host采用它，也不是任意模块自动验收工具。

Section 5 of that guide walks through a two-to-one author-copy exercise and identifies bindings, inputs and assertions. Section 6 separates the official recipe regression, your module tests and target Host acceptance. Local Cargo commands neither wire your implementation into a Host nor automatically accept arbitrary modules.

可选接线查看：从所用OCLive源码根执行 `cargo run -p oclive-cli -- doctor wiring --manifest-path <本工程>/Cargo.toml`。声明格式查该checkout的 `creator-docs/cli/OCLIVE_CLI_GUIDE.md` 中 `doctor wiring`；初始空声明保持未知。更换实现后由作者同步元数据，不把声明边当成实际绑定或调用证据。

Optional wiring view: run the command above from the OCLive checkout. Follow its CLI guide to maintain your own metadata. Empty declarations remain unknown; declared edges do not establish bindings or calls.

消费者调用通过不能代替检索质量、持久化、性能或目标 Host/发行版验收。原源码来自 OCLive；同目录 LICENSE 随本配方复制，自己的变更由作者维护。

A passing consumer call does not certify quality, storage, performance or a target distribution. The source came from OCLive and the recipe copies its LICENSE here; maintain your own subsequent changes.
