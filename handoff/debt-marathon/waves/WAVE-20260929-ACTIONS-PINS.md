# K-SUPPLY-10 · Actions 固定来源与维护

**SSOT 范围**：2026-09-29 这一切片的来源、差量、验证与可复现命令。当前债务状态唯一由 [主台账](../../TECHNICAL_DEBT_INVENTORY.md) 维护，执行范围见 [ROUND-02-PLAN](../ROUND-02-PLAN.md#k-supply-10--仓库-actions-固定引用2026-09-29)。这不是发版或完整供应链可信证明。

## 起点与实际写集

base `e71e1c5fcd48fdb931a8e7c110bcafda4113af83`，开场干净；父目标 CI 尚在运行，本地先实施，待终态后同步，避免取消父验证。5 个实际 workflow 中 4 个有外部 Actions，另一个不含外部引用。仓库 72 处加 CLI 模板 14 处，共 86 处，涉及 10 对原上游/ref。

只改 `.github/workflows/{ci,cargo-audit-lockfile,nightly-advisory,release-kernel-checksums}.yml` 的直接 Action 引用、Rust Action 的显式 stable 输入，补 `.github/dependabot.yml` 的 github-actions 更新组；CLI 的 `kernel/crates/oclive-cli/src/ci_cmd.rs` 同步固定既有模板并增真实生成器合同测试。全链首轮暴露真实消费者 `kernel/crates/oclive_ci_plan/tests/repository_contract.rs` 的旧 tag 精确文字断言，将它改为同一所选 SHA+原版本注释，保持该断言为硬失败。现有权限、事件、runner、needs/if/env/run、输出/秘密引用、原主版本和实验开关均保留；lock、profile、默认 linker、`-j 1`、API 与旧冻结证据未改。

## 上游身份

均从原上游 GitHub API 读取 ref → commit → action metadata；保留原始 JSON/YAML、stderr/native exit 和解析结果。表中 SHA 只绑定直接 Action 代码，不代表镜像、下载脚本、runner 或编译器版本也不可变。

| 原引用 | 所选完整 commit SHA | 仓库 / CLI 次数 |
|--------|--------------------|-----------------|
| actions/checkout@v4 | `11d5960a326750d5838078e36cf38b85af677262` | 0 / 5 |
| actions/checkout@v7 | `3d3c42e5aac5ba805825da76410c181273ba90b1` | 24 / 0 |
| actions/download-artifact@v7 | `37930b1c2abaa49bbe596cd826c3c89aef350131` | 1 / 0 |
| actions/setup-node@v4 | `49933ea5288caeca8642d1e84afbd3f7d6820020` | 0 / 1 |
| actions/setup-node@v7 | `820762786026740c76f36085b0efc47a31fe5020` | 14 / 0 |
| actions/setup-python@v6 | `ece7cb06caefa5fff74198d8649806c4678c61a1` | 2 / 0 |
| actions/upload-artifact@v7 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | 6 / 0 |
| dtolnay/rust-toolchain@stable | `f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f` + `toolchain: stable` | 14 / 5 |
| EmbarkStudios/cargo-deny-action@v2 | `3c6349835b2b7b196a839186cb8b78e02f7b5f25` | 0 / 1 |
| Swatinem/rust-cache@v2 | `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` | 11 / 2 |

Rust Action 上游明确要求固定 master 历史提交，不能固定会被回收的独立 stable 分支身份。取证时 master `02cb101e` 的运行步骤比原 stable 多了两个 `--force-non-host` 参数，故未采用。所选 `f3510ffd` 经官方 compare 证明为该 master 的可达祖先；除 toolchain 输入 required/default 的适配外，完整 action metadata 与原 stable 深比较相等。19 处都显式传 stable，不变更组件/目标。一条 metadata API 的 EOF 失败保留原件，新 attempt 一次重试成功；不是所有来源查询首次都成功。

## 配置与维护边界

已有 npm/cargo 更新配置逐字段未变。新增 github-actions 周三 03:00 Asia/Shanghai 周更，兼容 minor/patch 合组、最多 3 个打开 PR，主版本另定；没有自动合并。固定代码后仍需审更新 diff、来源与 CI。

Dependabot 扫描 workflow，不扫描 Rust 模板字面量；模板内原 v4/v2 身份继续保留，相关更新须人工同步并跑生成器合同。配置存在不证明实际 bot PR 已产生、全部引用已自动维护或外部账户功能已启用。更换主版本、统一模板来源/引入更新器另定，不把这些取舍混入本切片。

官方依据：[安全使用 Actions](https://docs.github.com/en/actions/reference/security/secure-use)、[自动更新 Actions](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/auto-update-actions)、[Rust Action](https://github.com/dtolnay/rust-toolchain)。

## 验证与证据

本机原始材料在忽略目录 `.cursor/plans/debt-actions-pins-20260929-r0/`：`s0-inputs.json`、`s1-resolved-actions.json`、`s1a-selected-actions.json`、`upstream/`、`s2-source-replacements.json`、`s3-structure-audit.json`、`gates/`。路径存在不等于他机可携带；接手时取不到原件则记 `needs-evidence-access`，不得补造日志。

- **已执行**：5 workflow 前后解析树只出现受信 SHA/显式 stable 变化；CLI 生产模板只出现相同允许变化，npm/cargo 更新项不变。9 个同函数变异（tag、短 SHA、未知 SHA、他源、缺 stable、nightly、权限放大、run 变更、if 变更）全部被拒绝。
- **已修自引问题**：首次 fmt 检查 native 1 仅报新增断言换行；失败日志保留，只格式化 `ci_cmd.rs`，未格式化其他源文件。
- **已执行**：CLI 生成器真实 3 项通过、Shadow 21 场景模拟通过；默认/7 篇改文链接分别 52/7 篇通过。第一次 `check:ci-local` 运行 588.73 s，Dimension 5、前端/构建、Rust clippy 与 workspace 库测试通过，workspace 集成的 `oclive_ci_plan` 5 项里 4 过、1 项旧 tag 断言失败，整条 native 101；原始日志保留，不能报全链绿。只把该断言改为已核上游 SHA/版本注释，定向 5/5 native 0。另 `git diff --check` 对 `ci.yml` 既有混合换行的改行报尾部空白；先完整 LF 化验证 YAML 相等，再保留 581 条未改行原换行、64 条改行 LF，最终 diff 收回 64+/48− 且 `diff --check` 返回 0，两个转换与原始失败证据均保留。
- **第二轮非零**：`check:ci-local` attempt2 107.55 s 在 `cargo fmt --check` 停止，新增 `repository_contract` 断言换行未符合 rustfmt；尚未进入本轮 Rust 测试。只格式化该测试文件，定向 `cargo fmt --all -- --check` 新 attempt native 0，未改测试语义。两次全链失败与两次格式失败各留独立原始 stdout/stderr/native，不重命名成成功件。
- **第三轮全链通过**：`check:ci-local` 原生 **0 / 744.91 s**，Dimension 5 `--ci` 所有实际步骤 PASS（其内按设计 skip 慢速样本库测试，后续 `check:rust` 仍跑 workspace 库测试），前端 lint/typecheck/build、Rust fmt/clippy/lib、workspace 与 CLI 集成均按原命令链通过。`s5-closeout.json` 对本轮 28 项受影响输入/依赖 **28/28 字节与 SHA256 匹配**，7 篇文档均 UTF-8 无 BOM、无替换字符；`ci.yml` 全文仅改行行尾，前后 YAML 完全相等。Shadow **21 模拟场景**通过，但模拟不替代实际远端 CI。
- **待收尾**：上述第三轮后仅追加这段验证文字及主台账/计划/事件的本地状态，须重新做文档/镜像/债结构、最后输入哈希与 diff 的验证。对应新目标正式 CI 单独绑定，不以本地成功替代；父 `e71e1c5f` 的远端 CI 未结束前不推送。

定点复现：`cargo test --locked --offline -p oclive-cli --bin oclive-cli ci_cmd::tests -j 1 -- --test-threads=1`；全链 `npm run check:ci-local`。本机 runner 仅对子进程设置已有 `OCLIVE_VOICE_PYTHON` 的已核解释器、Cargo offline 和 jobs 1，不修改父环境/PATH或安装依赖；限时/非零原样登记。不执行 ignored CP-INT 场景、真实模型/语音、发布或凭据读取。

## 验收出口

本切片完成条件为固定来源、范围内本地 gates 与目标 SHA 正式 CI；父债还保留模板升级维护与实际 bot 更新证据。远端 pending 不写 Done，不新增证据专用提交来填 run ID。下一次实质工作同步远端结果；K-BUILD-06/07 的独立实验与默认配置边界保持。

**目标 SHA 远端终态（下一实质提交时入账）**：`245d6ca98edfa8ebb354e2609d556a455336cc73` 的正式 [ci.yml run 36528350305](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36528350305) 为 `completed/success`，17 个 job 全部 success，含 Windows/Linux Rust 与稳定 `ci-gate`。只读原始 `gh run view --json status,conclusion,headSha,jobs` 保存在本机 `.cursor/plans/debt-actions-ci-20260929-r0/run-36528350305.json`（40644 B，SHA256 `2D2028467789B1DE51C9588A210CA99143986C133E335B4C4858FFE76E684879`）；查询时逐项断言 HEAD、终态、17 项及 `ci-gate`。后续三个文档整理提交和本轮登记检查代码没有改固定引用的 workflow、Dependabot 或 CLI 生成器；该旧 SHA 的绿灯不替代它们的新目标 CI。固定引用与范围内测试已获远端实证，但 Rust 模板人工同步及实际 bot 更新 PR 未验证，K-SUPPLY-10 仍 Partial，不宣称维护机制全覆盖或父债 Done。

## 2026-10-01 · K-SUPPLY-10 rust-toolchain 同步合同

本轮 base `c2dd6112c60f0aed6e0687b5d3654f863e8266f8`，开场 `main` 干净；不升级依赖、不改生产 workflow、生成字节或 CI 编排。针对真实 Dependabot PR #184 只更新 workflow Rust Action、CLI 模板可能漏同步的缺口，仅在 `kernel/crates/oclive-cli/src/ci_cmd.rs` 的 `#[cfg(test)]` 增加真实扫描合同：读取仓库 `.github/workflows/*.yml/*.yaml`，要求所有 `dtolnay/rust-toolchain` 引用唯一、40 位十六进制且显式 `toolchain: stable`；Library/Kernel 两类真实 `render_ci_yaml` 输出必须含同一 pin 与 stable。缺 Action 的单个 workflow 合法，但仓库总体缺失和任一模板缺失均拒绝。

同一 collector 的内存负控已覆盖：workflow-only 改为 PR #184 新 SHA、单处仓库 pin 漂移、模板侧漂移、缺 stable、短 SHA、缺 Action；均按预期错误原因拒绝，未改工作树 workflow。定向 `ci_cmd::tests` native 0、5 passed；完整 CLI 测试 native 0、16 套 132 passed；最终 fmt、Clippy `-D warnings`、Dimension 5 `--ci` 29/29、默认及改文链接、docs-only stale paths、doc registry、doc mirror、debt-marathon 全部 native 0。首次 fmt 和 Clippy 的新增测试辅助代码问题及修复后日志分存本地 `.cursor/plans/debt-toolchain-sync-20261001-r0/`。CLI 生产代码前缀、workflow 与 Cargo.lock 差量为零；完整 CLI 测试的临时项目构建访问 crates.io 索引。K-SUPPLY-10 继续 Partial，实际依赖升级与模板同步尚未完成，尚未建立自动维护，本轮目标 SHA CI 尚未取得。

## 2026-10-01 · PR #184 的实际 Action 更新与 CLI 模板同步

接续上一节合同，当前主线以 [Dependabot PR #184](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/184) 为更新来源：该 PR 仅在四份 workflow 的 14 处把 `dtolnay/rust-toolchain` 从 `f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f` 更新到原上游提交 `02cb101ec7c40f2c49e1d9714d64511d8e1b74de`；CLI 的五处生产模板由本轮一并同步。PR 旧 head 的 CI 17/17 成功只证明旧 base 上 workflow 变更，不证明主线与模板同步。本轮在当前主线作配对修改，不直接合并、修改或关闭旧 bot PR。

[上游 compare](https://github.com/dtolnay/rust-toolchain/compare/f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f...02cb101ec7c40f2c49e1d9714d64511d8e1b74de) 为 7 个提交；Action 运行差量是 `rustup toolchain install` 与 `rustup default` 增加 `--force-non-host`。先前选择等价祖先是当时冻结的审慎结论，不回写；本次是独立的兼容更新。现有 14+5 处都显式选 `stable`，Action 没有非宿主工具链、组件或 target 输入；aarch64 target 由工作流后续命令单独安装。此条件下没有新增非宿主选择，仍须目标 SHA 的实际 CI 证明当前正常路径，不能宣称上游实现完全等价或覆盖独立 nightly/release 执行。

源码范围仅为上述 19 处 40 位 SHA 代换；`toolchain: stable`、其它 Action、workflow 的事件/权限/runner/命令/失败策略、Cargo.lock 与测试逻辑保持。字节替换保留原磁盘换行，Git diff 仅列出预期 19 行。既有 `ci_cmd::tests` 正控和漂移负控在新 pin 上原生 **5/5、exit 0**；`cargo fmt --all -- --check` exit 0。独立只读复核确认 14+5 差量、无遗漏生产引用及负控在新 pin 上仍有效；其范围是 D1/L2 差量审查。

本地完整 `npm run check:ci-local` **attempt1 exit 1**：子进程的 `OCLIVE_VOICE_PYTHON` 被误指向 `python.cmd` 包装器，Dimension 5 的语音 registry 检查失败；保留原日志，不归因为源码。以此前已核的 portable `python.exe` 仅修正子进程环境后，**attempt2 native exit 0**：Dimension 5 **29/29**，其后的前端 lint/typecheck/build、Rust fmt/Clippy/库测试与工作区/CLI 集成测试均沿原链通过。原始日志在 `.cursor/plans/debt-rust-toolchain-update-20261001-r0/`，attempt2 `ci-local-attempt2.log` 315653 B / SHA256 `44C45ACC3CE4C99AEE0159D020198DA3C5FE919B1A1E9B3A8D085BC953B1A06C`。默认/四份改文链接、docs-only stale paths、登记、镜像、债结构及 `git diff --check` 均 exit 0。父提交 `ed6f6457` 的正式 CI 尚未终态时不推送新主线；新目标 SHA 的正式 CI 也仍待证。K-SUPPLY-10 保持 Partial：这是一项真实更新的人工同步样例，不等于建立模板的自动更新器。
