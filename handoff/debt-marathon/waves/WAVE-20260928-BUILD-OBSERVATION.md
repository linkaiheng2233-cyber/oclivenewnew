# 构建成本与缓存观测

**SSOT 范围**：K-BUILD-06 / K-BUILD-07 本轮测量、纠偏和下一实验的证据；状态只在 [主台账](../../TECHNICAL_DEBT_INVENTORY.md)，执行合同在 [ROUND-02-PLAN](../ROUND-02-PLAN.md#构建成本与缓存切片2026-09-28)。
**最后更新**：2026-09-28。

本轮 base `d30f47c75bc9ef7454fb308204f1997c884b3422`，开场工作树干净；仓库工具链首批已提交，目标 [CI 36403228999](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36403228999) 本轮取证时仍运行，不称远端全绿。用户授权持续推进；本轮不改变生产 Kernel/Host/DTO、鉴权、依赖、jobs/linker/profile、个人 `E:\Env` 脚本或旧冻结证据。

## 观测合同与可取得性

本机证据根 `E:\OCLive\oclivenewnew\.cursor\plans\debt-build-observation-20260928-r0\` 为 Git 忽略目录；本页保留摘要和身份，clone 不自动取得原始 JSON/一次性采集器，缺件应记 `needs-evidence-access`。`s0-inputs.json` 冻结四个构建输入及限定环境，未打印全环境、读取凭据、生产 DB 或模型内容。

只读枚举单次预算 60 s / 200 万项；Node 按 `lstat` 排除符号链接/目录 Junction，独立 .NET 枚举再按文件属性拒绝所有重解析点。只读长度/时间/链接数，不读产物内容；两次结果按文件/目录总数及全部 30 个分组精确比较。结果为分路径逻辑字节，不去重硬链接、不测压缩/稀疏/实际分配；顺序扫描也不是原子快照。

## 当前配置与冷热点口径

| 面 | 本轮实际核对 | 限制 |
|----|--------------|------|
| target | 仓库 `.cargo/config.toml` 与离线 `cargo metadata --locked --offline --no-deps` 均指向 `E:\OCLive\oclive-dev-artifacts\oclivenewnew-cargo-target` | 不是仓根 `target`；祖先/默认用户及实际 `CARGO_HOME=E:\Env\state\cargo-home` 均未发现额外 Cargo config；限定的 jobs/target/incremental/rustflags/linker/wrapper/profile 环境覆盖为空，不推断另一台机器相同 |
| 本地串行措施 | clippy、lib、workspace 集成及 all-tests 脚本带 `-j 1`；CLI 集成只有 `--test-threads=1` | 编译并发与测试线程是两件事；CLI 的嵌套 Cargo 工作另行评估，不借口纠偏改变命令 |
| CI 措施 | `ci.yml` 的 workspace tests 以 `CARGO_BUILD_JOBS=1`；其 clippy 步骤无同项覆盖 | 本地 package 与 CI 的命令/平台不同，不冒充完全等价性能对照 |
| profile | 根 Cargo dev 为 `split-debuginfo=unpacked`、依赖 `opt-level=1`；release 为 thin LTO、主包 codegen-units=1、依赖 16、strip symbols、panic abort | 未改 profile；本轮没有重建 release、尝试另一 linker 或测 response file |
| 声明的目标 | metadata：14 workspace packages、130 targets，其中 105 个 `kind=test`（Host 5、Tauri 62） | 这是声明的集成 test target，不是测试用例数、该命令实际链接数或存留 exe 数 |
| 已有综合日志 | 前批 `check:ci-local` attempt2 exit 0 / 557.66 s；其中 clippy 44.06 s、lib 构建 25.23 s、workspace 集成构建 4m35s、CLI 构建 7.22 s | `Finished` 行只计相应 Cargo 构建段，CLI 测试内又嵌套构建；不是各段完整墙钟或 linker 时长。没有前置缓存/峰值采样，不能称受控热基准 |
| 9 月 22 日旧耗时 | 外部审计自述 1,635 s / 27.3 分钟；deps 部分复用、incremental 冷，且包含被删除的 Host rlib 变体恢复 | 不能将全部耗时归因 `-j 1`，不能与本次不同输入/命令的 557.66 s 相减宣称优化收益 |

四个输入的 bytes/SHA256 前后相等，见 `s4-provenance.json`；`Cargo.lock` 未改，无依赖升级。7 月 10 日四份原始失败日志本机仍可取得，错误历史保留，不为中文乱码改写旧件；其存在不证明当前复现了 LNK1102。

## 当前产物枚举（2026-09-28 17:33–17:35，UTC+8）

Node 枚举 exit 0 / 3.91 s；.NET 独立核对 exit 0 / 4.02 s：**103,409 文件、7,041 目录、239,142,454,912 逻辑字节 = 222.719 GiB**，分组/扩展名/月度三种汇总与总量相等。读错 0、预算截断 0、观测重解析点 0，两次前后采样均未见 Cargo/rustc/link/lld-link 进程；这不构成整段持续进程监控。

| 分组（互斥） | 文件数 | 逻辑大小 | 含义 |
|---------------|--------|----------|------|
| `debug/deps` | 17,075 | 137.933 GiB | 最大分组；编译产物/元数据/调试符号并存，不能按后缀或月份盲删 |
| `debug/incremental` | 58,894 | 73.408 GiB | 增量目录的分路径逻辑大小；硬链接去重和实际可回收量未知 |
| `release/deps` | 2,965 | 4.369 GiB | 发布配置的编译产物；不是本次 dev 缓存的可替代副本 |
| `debug/build` | 3,556 | 2.179 GiB | build script 中间/输出文件；清除会改变后续重建工作量 |
| `debug/.fingerprint` | 10,826 | 0.005 GiB | 本次仅约 5.54 MB；为省空间盲删并无已证收益，还会丢失 Cargo 新鲜度判据 |
| 其余 | 见原始 30 分组 | 总量减上述项 | 含顶层 exe、资源、文档、bundle、wasm 等，本轮未混称全为缓存 |

有 **42,275 个文件路径 `nlink>1`**；总量与这些路径均未按文件对象去重。因此既不能称 target 实际占盘 222.719 GiB，也不能称删除 incremental 可回收 73.408 GiB。全树 `.pdb` 944 个 / 85,928,411,136 逻辑字节是另一种重叠分类，不能与目录表相加；mtime 仅为 UTC 月份分组，不能当最后使用时间或删除许可。

Host rlib 当前 dev **9 个 / 8.163 GiB**，release **6 个 / 0.314 GiB**；最大 dev 文件 1,002,923,714 B，近期写入的变体也并存。未比较产物内容或反解每个 feature/配置来源，不能称相同内容的重复文件，不能“仅留最新”。全树 exe 也包含依赖 build script 和多种配置的旧产物，不等于一次构建的独立链接次数。

## 复用缓存编译测量

按计划只在现有 target 上保持 `-j 1`、离线和 `--no-run`，不运行测试、真实 provider 或 ignored 场景。首次不预设为热缓存；Cargo artifact `fresh` 和实际编译输出决定是否复用。采样工作集/私有内存只覆盖采样到的所持 Cargo 子树；共享页合计不等于独占物理内存、采样峰值不等于真实峰值，未出现 linker 不称链接峰值已证。

workspace `warm-01` 到 **180.03 s** 触及 deadline：受管入口 exit 1、终止后的 Cargo native exit **-1**，没有 `build-finished` 行，**不是编译通过或内存耗尽复现**。只终止所持 Cargo 子树，收尾复查编译器进程空；未按原合同发第二次 workspace 尝试。628 条 JSON 中 565 个 compiler artifact 报告，512 fresh / 53 non-fresh；非 fresh 来自 runtime、Host 与 Tauri，另有 63 条 build-script 状态报告，compiler-message 0。源配置/命令不同阶段的触发原因未证，不预设是 `--no-run` 或某个源码缺陷。

272 次点采样观测到 **48 个 link.exe 的 PID＋创建时间身份**，不是 48 个已绑定 target 的通过结论。最大单 linker 工作集样本 **5,224,554,496 B（4.866 GiB）**，其私有内存样本 2,841,219,072 B；Cargo 子树合计工作集采样最高 5.111 GiB、私有内存 3.237 GiB，空闲物理内存采样最低 14.664 GiB。未触发内存止点；没有 response-file/输出目标逐项绑定，短进程也可能漏采，不能以此证明真正峰值或全矩阵不会 OOM。

停编译后再次完整枚举＋独立核对通过：103,410 文件 / 239,167,314,016 逻辑字节，比盘点前 **+1 文件 / +24,859,104 B**；其中 deps +46,743,552 B、incremental -22,105,104 B。仍未按硬链接去重，不把逻辑差当实际回收或唯一归因于本命令的磁盘空间；原始前后 JSON 均保留。

合同 1b 的独立 `host-target/` 首次在 **60.34 s** deadline 停止：受管 exit 1、Cargo -1；395 个 artifact 报告为 392 fresh / 3 non-fresh，日志显示 dom_query、tauri-utils、tauri-build/codegen 依赖重编，没有 compiler-message 或存活编译器。仅缩选 target 不保证 Cargo 的完整单位图/依赖特性集合不变，不能将它与 workspace 的命中率直接比较；该次也没有 `build-finished`，不称通过。

据此先结束原短预算尝试，再按合同 1c 复评为**最后一组定额**：独立 `host-target-completion/`，单目标首次上限 480 s、首次通过才复测（上限 60 s），保留所有旧失败且不重复 workspace。此为我在已有授权内的预算/粒度调整，参数中没有 jobs/linker/profile 变化；不是把旧日志换标签或删除失败。

| 同一单目标命令 | native / Cargo 完成状态 | 缓存判据 / 边界 |
|----------------|-------------------------|-----------------|
| completion `warm-01` | exit **0**、`build-finished.success=true`；受管含采样墙钟 **105.53 s**，Cargo `Finished` 为 1m45s | 464 artifact 报告：458 fresh / 6 non-fresh；包含前两次中断后的缓存状态，是完成目标基线的成本，不是空目标冷构建或稳态热耗时 |
| completion `warm-02` | exit **0**、`build-finished.success=true`；受管含采样墙钟 **0.72 s** | **464/464 fresh**，non-fresh 0；只报告一次完全复用观测，不据此算百分位、workspace 加速比或性能保证 |

两次 arguments 相同，唯一 exe 为 `a_turn_harness-a2119fd0d50f92be.exe`；Host lib/test 的实际 features 均为 `default, tauri-commands`、opt-level 0、debuginfo 2，第二次均 fresh。此基线包含所选 Host 的 Tauri command 依赖，**不是纯最小 Kernel 构建成本**。未启动 exe 或任何回合/ignored 场景，两次诊断报告为空且收尾未见编译器存活。

completion 首次 171 次采样、3 个 linker 身份，最大 linker 工作集样本 2,889,420,800 B；仍未逐一绑定输出/response-file。复测仅 1 次样本、未观测 linker；这不等于进程内存为零或已验证新链接的峰值。

本组结束后完整枚举与独立核对通过：**104,188 文件 / 242,072,309,440 逻辑字节**，相比本轮起点 +779 文件 / +2,929,854,528 B。编译器正常更新/替换产物，未人工清理；这再次说明配置/特性组合与缓存新鲜度应共同登记，而不是按日期保留最新一个变体。

## 保留规则与下一实验

本轮**撤回**旧“每月只清 incremental/build/fingerprint 与按月陈旧 pdb”作为已批准维护规则的措辞；它仍是待评估建议，不是既定删除白名单。9 月 22 日外部审计一边称 deps 全保留，一边登记删除其中 Host rlib 变体；外部原件保留，后者的过度操作已在主台账告诫不重做。

单 Host 目标完全复用基线已取得；下一步先诊断所选单位图/特性和文件新鲜度，再为一项受控重编绑定 linker 输出/response-file及其资源采样。“复用 target”“仅 incremental 未命中”“新空 target 但 registry 热”“删除后重建”分别命名，不使用笼统冷热标签；workspace 全量仍没有受控完成结果。

- 本轮不人工清 target，也不改旧冻结二进制、账本、日志、树；Cargo 正常更新其编译产物。Git 忽略不是可删证明。
- 维护候选须逐路径核关联配置/特性、当前 Cargo 新鲜度和冻结证据/调试用途；硬链接身份及实际分配空间未测，不承诺回收量。
- 清理或冷构建须先确定绝对路径范围、完整保全/回退、峰值/耗时/磁盘预算及失败止点，不能以年龄、后缀或最新一项代替准入。
- 保持现有并发措施。更换 linker、降低调试信息或拆 test 目标会改变实验条件，必须另有对照计划，不在本轮默选。

## 证据身份与出口

| 本机证据 | bytes / SHA256 | 能证明什么 |
|----------|----------------|------------|
| `s1-artifacts.json` | 13220 / `8B4D168C738FC114D0582C9AF857DEE2D3881F527E69C662BF7B029BEE6591BE` | 完整 metadata 枚举、分组、时间/硬链接/Host 变体 |
| `s3-independent-check.json` | 2639 / `7A1B3A33DEB72F57F7A0A01B28D4833AAABBB8CE5A6B018254FB76E2029E636D` | .NET 属性核与逐分组精确对照 |
| `s4-provenance.json` | 本机包含四输入及本轮/旧证据身份 | 配置输入不变；7 月原始错误、9 月外部清理与前批综合日志只作各自范围的历史 |
| `warm-01.exit.json` | 76296 / `2AF21CFF8887919E8BD96F1418598F3D4C8CAD0EA85052BECB68BB4407424B68` | workspace 原生终止码、deadline 与完整点采样；不是 PASS |
| `host-target-completion/summary.json` | 4159 / `C13661FE37E324639BEF54E5EBB5300F6AD9395D7218BB20FECC57F940828C6D` | 两次原生/完成状态、同参、Fresh、实际 Host 特性与各自边界；原始 stdout/stderr/exit JSON 同目录分别保留 |

**本轮执行结论**：完整只读盘点及单 Host 目标复用基准取得；workspace 和短预算尝试为 **Stopped · deadline**，保留其非零出口。K-BUILD-06/07 仍 OPEN，未建立或执行删除白名单、未复现 OOM、未验冷重建或替代并发/链接器方案。一次性采集器不进入 doctor/CI，不扩大 `check:toolchain` 的静态范围。

**文档出口 Locally verified**：默认链接 52 文件与本轮显式 4 文件、docs 旧路径、26 根文档/5 哨兵登记、12 auto plans 结构均 exit 0；四文 UTF-8 无 BOM、正常汉字且无替换符，diff 检查通过（Git 换行提示不等于失败）。不为纯测量/文档重复完整 `check:ci-local`。

首批交接动作是核目标 SHA CI，再诊断实际 Cargo 单位图/特性和所选产物新鲜度；已由下节接续。只读可安全重跑且使用新输出名；编译测量须新 attempt，不复用本轮日志，压力/冷构建另定计划。首批无技术债 Done 迁移，没有业务 run ID 或真实模型/网络/语音调用。

## CLI 集成测试启动器收敛（2026-09-28）

接续 base `6c28b1e2e5cb42214120e7a5903c85c4420e5365`，开场干净。首批工具链代码 `d30f47c75bc9ef7454fb308204f1997c884b3422` 的 [CI 36403228999](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36403228999) 已 **success，17/17 job 与 ci-gate 通过**；构建观测文档 base 的 [CI 36410519684](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36410519684) 接续时仍运行，两个 SHA 不混用。合同见 [ROUND-02-PLAN](../ROUND-02-PLAN.md#cli-集成启动切片2026-09-28构建债接续)。这是测试基础设施的 M 级切片，不关闭两个父债。

### 诊断与关联范围

| 触点 | 事实 / 改动 | 未扩大范围 |
|------|-------------|------------|
| 原测试启动器 | `tests/common` 与 explain/completions/dry-run/from-existing/monolith 中共 8 个命令构造点用 `cargo run -p oclive-cli` 启动自身；其余 scaffold/registry 等已有 `CARGO_BIN_EXE_oclive-cli` 写法 | 新 helper 复用该既有机制，直接运行本次 Cargo 测试提供的 CLI 二进制；不证明安装包或 `cargo run` 包装器本身 |
| 参数与消费者 | 新 `common::cli_command()` 保持 repo cwd；原 experimental 参数、输出捕获、OCLIVE_ROOT、正向与拒绝断言保持，6 个测试文件改动 | 使用父测试所选特性，而非再次隐式构建默认 CLI；当前完整行为验证是默认特性，all-features 由 clippy 编译审查，不冒充该特性的全行为测试 |
| 真实生成项目 | `cargo_build` / release / monolith helpers 与 CLI build/bench 仍做真实 Cargo 工作，测试单线程保留 | 不把所有嵌套 Cargo 都删掉，不减原测试或生成项目断言；原 package/CI 并发配置保持 |
| 内核溯源 | runtime build script 监测 `.git/HEAD`、HEAD reflog 与当前 ref，更新 manifest 的 git_commit / built_at；workspace tree 显示 Host 的 tauri-commands 由桌面包启用，而 Host 默认为空 | 提交会使该链重编是现行溯源语义；不删 Git 监测、不改变清单语义。单目标与 workspace 不预设相同依赖特性集合 |
| 生产关联端 | CLI 生产源码、Kernel/Host/Tauri/shared、角色包、公开 DTO/trait、鉴权、Cargo.lock、profile 和 target 路径均未改 | 无架构/安全取舍或真实业务回合；不向 Chat Pro 限定接入结论增加覆盖 |

### 已完成的实测

本机新证据根 `E:\OCLive\oclivenewnew\.cursor\plans\debt-cli-launch-20260928-r0\` 仍被 Git 忽略；11 份原始源码/配置快照、6 份实施字节冻结与各尝试原日志独立保留。一次性 runner 只对其 child 设置离线环境和独立 `OCLIVE_HOME`，隔离测试注册表写入；未改父环境或用户注册表。采样与内存止点沿用合同，不宣称持续完整进程追踪；原生退出与日志互证，测量器不进入 CI/doctor。

| 命令 / attempt | 原生结果 | 口径 |
|----------------|----------|------|
| 四定向目标 `before-smoke-01` | exit 0，**7 passed / 0 failed**；受管 25.61 s，Cargo 初始构建 7.72 s | 修改前的首次观测，含缓存状态和重编；不是纯热基准 |
| 同四目标 `after-smoke-01` | exit 0，**8 passed / 0 failed**；受管 51.17 s，Cargo 初始构建 49.55 s | 新增真实“PATH 只有空工具目录、CLI 仍能 explain”进程用例通过；旧用例保持。ring/rustls 等重编使总墙钟更长，不称整体编译提速 |
| 测试执行段 | 同一 completions 组 15.37→0.97 s、dry-run 0.64→0.02 s、from-existing 0.33→0.02 s；explain 2 条 0.65 s→含新负控 3 条 0.04 s | 测试框架分组时间，不是 compiler/linker 时间或统计性能保证；不同源码与缓存状态不据此计算全项目加速比 |
| 进程点采样 | 四目标旧/新分别观测 14 / 2 个 Cargo PID＋创建时间身份 | 含 rustup proxy 与原生 Cargo，短进程可能漏采，不能当精确启动次数；没有 Cargo 进程数＝性能保证的推论 |
| CLI 全测 `full-01` | `cargo test --locked --offline -p oclive-cli -j 1 -- --test-threads=1` **exit 0，82 unit + 47 integration = 129 passed / 0 failed / 0 ignored**，69.90 s | 15 个集成 target；真实生成项目标准/release/monolith 构建、bench 夹具与拒绝路径保留。stderr 的无效 monolith 报错属于通过的拒绝用例，不称零 stderr |
| CLI clippy `clippy-01` | `cargo clippy --locked --offline -p oclive-cli --all-targets --all-features -j 1 -- -D warnings` **exit 0**，75.38 s | 覆盖 CLI 所有目标及可选 diagnostics-host 的编译审查；不称可选特性的完整行为验证 |

完整 CLI stdout **27856 B / SHA256 `DC7584B96DF3865DE01F7BECCB0A362F7F67EA1F9600639F267A4E185334757E`**，stderr **9131 B / `F903917B156404CA09E4421946C63FA5BFB10370C893EE8CD3632DA646080855`**；`s3-full-summary.json` 为可重算派生摘要，不能替代原生 exit JSON。前测、后测和全测均收尾未见存活编译器；这个点检查不构成所有 detached task 已 join 的证明。

### 收口与下一动作（本地验证完成）

6 份测试代码在全测后逐项核对 bytes/hash 不变；CLI 全测与 all-targets/all-features clippy（诊断 0）、workspace fmt 与分层 ratchet通过（3/3、FQ 1/1）。最终静态出口为默认链接 52 文件、本轮显式 4 文、docs 旧路径、26 根文/5 哨兵登记、12 auto plans 结构、diff 均 exit 0；编码单独核 UTF-8 无 BOM、正常汉字和零替换符。外部 Bugbot 未运行，人工审查覆盖真实 executable、所有旧启动点、参数/cwd/环境、特性选择、生成项目保留及拒绝断言。公开生产 API 未改变，无新增 doctest 要求；门禁组合/前端无改动，不重复无关全仓工程链。

**自查失败保留**：一次性静态出口采集 `gates-01` 未将 `Get-Command` 的多条路径选成单一可执行文件，启动错误后还复用了上一条零退出码；其 Node 与 Git 检查为 **NOT_RUN**，整个汇总作废，不能引用为通过。原结果/空日志不覆盖，`s5-gates-observer-failure.json` 登记缺陷；原异常仅在本次工具会话中，未补造原始日志。修为选择实际首个应用、每步初始化 native、观察器异常停后续后，`gates-02` 八项真实检查全部 exit 0。该缺陷只在一次性采集层，仓库工具链 gate 和生产代码未受影响。

本切片执行结论为 **Locally verified**；停止写入后冻结本轮十文件并提交，待父基线 CI 终态再按既有授权一次推送，避免取消仍在运行的目标验收。目标 CI 与父 SHA 分列，不移技术债 Done。后续若继续研究链接峰值，先绑定具体输出目标和编译参数；缓存物理分配/硬链接去重/冻结用途仍未知，不删产物或更换 linker/profile，也不以本切片证明 workspace 完成成本。纯静态可安全新日志复跑；全测/采集仍按新 attempt 独立输出，不改旧件。

## 单 Host 链接目标绑定（2026-09-28）

CLI 切片已本地提交 `787aa5cbbfa7b1aa2a600e8bf63cf2ac8e3df1cb`，开场干净，尚未推送；父基线 CI 仍运行。按 [诊断合同](../ROUND-02-PLAN.md#单-host-链接目标绑定2026-09-28诊断接续) 仅做一次 `host-link-01`：既有单 Host compile-only 命令保持、12 份源码/配置输入不变、不 touch 源码、不清产物。新 Git 清单身份的正常重编不属于人为冷缓存，也不与旧短预算作加速对照。

本机证据根 `E:\OCLive\oclivenewnew\.cursor\plans\debt-host-link-20260928-r0\`，原始 JSON/一次性 observer 和 response-file 内容只在本机忽略目录保留；缺件仍记 `needs-evidence-access`。采集只存所持 compiler/linker 进程的命令和资源，不打印全环境或其他用户进程命令。正式测量前，两真实 observer 函数的 8 项离线探针覆盖两种带空格引号、UTF-16 response、缺目标/已消失文件和越界拒绝；不启动产品或 Cargo。

| 观测 | 实测 / 归属 | 边界 |
|------|-------------|------|
| 完成 | native **exit 0**、唯一 `build-finished.success=true`、diagnostic 0、observer error / stop reason 均 null；含观测墙钟 **9.93 s**、23 个采样 | 只编译，未启动 harness 或真实回合；收尾编译器进程空，未触发内存/时间止点，无第二次尝试 |
| 新鲜度 | 464 artifact：**461 fresh / 3 non-fresh**；非 fresh 为 runtime lib、Host lib、a_turn_harness | 源配置未变，只在新提交清单身份下重编；不可用来推断所有依赖不会重编或整个 workspace 耗时 |
| 目标绑定 | 唯一观测 `link.exe` PID＋创建时间身份，`/OUT` 正规化路径与 Cargo 唯一 executable **精确相等**：`debug/deps/a_turn_harness-a2119fd0d50f92be.exe` | target features 为 `default, tauri-commands`，opt 0、debuginfo 2；包含 Host/Tauri command 依赖，不是纯最小 Kernel 峰值 |
| response-file | 捕获 `debug/deps/rustcTnWkup/linker-arguments` **134994 B / SHA256 `B19516C43D92FCE3FDF5D022FA0529A65C0FB334F0997E4F794D0A3BE10FF630`**；494 行、493 非空参数，明确含 `/DEBUG`、`/OPT:REF,NOICF` 和该 `/OUT`，没有 `/INCREMENTAL` 参数 | 捕获时的只读文本快照，逐路径边界和 reparse 检查通过；不把没有显式参数等同于 linker 默认语义，亦不假设其他 targets 参数相同 |
| 绑定的内存样本 | 工作集最高 **3,233,972,224 B（3.012 GiB）**、私有内存最高 **1,793,667,072 B（1.670 GiB）**；所持树合计工作集样本最高 3.231 GiB、私有 2.038 GiB，空闲物理样本最低 15.991 GiB | 250 ms 为目标间隔，实际含 CIM 开销；这是 sampled peak，短进程仍可能漏采。工作集合计可重复计共享页；没有 OOM，不证明整体矩阵的真峰值或允许取消 `-j 1` |

**读取层自纠**：第一次离线夹具把两组字符串数组各压成一项，6 项通过并不覆盖预期 8 种输入；`s1-parser-check.json` 原件保留，改为显式双元素数组和总数断言后 `s1-parser-check-02.json` 为 8/8。派生 `s2-bound-summary.json` 的 `contains_debug=false` 用未去外引号的行首 regex，属读取错误；原快照与目标绑定不变，`s3-response-arguments.json` 按非空参数去外引号、精确对照后更正 `/DEBUG` 存在。均未重跑正式测量或改变源配置；不归因产品/原始数据损坏。

原生/采样及响应快照件 `host-link-01.exit.json` **80269 B / `615D32B6D8FB0B53FA9AA8506603D71E00F648DF223BC058B124141C6436C469`**，更正后的参数件 `s3-response-arguments.json` **1991 B / `69D40514B8450CE04BFE00F5FDDAA1C9467441EF1EBAE8C367525B16DB686555`**；文件身份汇总 `s4-evidence-inventory.json` 不替代原 native/Cargo 日志。

**结论与续跑**：该目标的资源样本已完成归属，诊断缺口在这一边界内关闭；K-BUILD-06/07 仍 OPEN。没有取得新 linker/profile/并发方案的等价证据，保持现有措施与清单溯源；缓存保留策略的物理分配、硬链接去重和冻结用途仍未证。后续优先依据已绑定参数提出单一候选的有界对照计划，或做逐路径保留核验，不重复已完成的观察以堆轮数。默认/改文链接、docs 旧路径、登记、债结构、diff 与四文编码已通过，12 输入与父环境不变且编译器收尾空；作为独立诊断提交，与 CLI 切片一起在父 CI 终态后同步。当前生产源码/门禁组合未变，不重跑无关全链。
