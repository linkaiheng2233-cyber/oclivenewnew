# 构建成本与缓存观测

**SSOT 范围**：K-BUILD-06 / K-BUILD-07 本轮测量、纠偏和下一实验的证据；状态只在 [主台账](../../TECHNICAL_DEBT_INVENTORY.md)，执行合同在 [ROUND-02-PLAN](../ROUND-02-PLAN.md#构建成本与缓存切片2026-09-28)。
**最后更新**：2026-09-29。

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

## 缓存分配与文件身份去重（2026-09-28）

接续 base `83af77bb74daf9eed6092770b80824c5272ce1f8`，开场干净；[只读合同](../ROUND-02-PLAN.md#缓存分配与文件身份去重2026-09-28只读接续) 限定原 target、60 s / 200 万项。本机独立新证据根 `E:\OCLive\oclivenewnew\.cursor\plans\debt-cache-allocation-20260928-r0\` 被 Git 忽略，缺件仍记 `needs-evidence-access`。没有运行 Cargo/产品/业务、读取缓存文件内容或删改产物；12 份源码/配置含已验证 CLI 字节与限定父环境在前后完全一致。

**接口与口径**：使用 metadata-only 的 `CreateFileW`（desired access 0、OPEN_EXISTING、共享读/写/删除、拒绝 reparse），每个句柄在读取后释放并核最终路径边界；按 `GetFileInformationByHandle` 的卷序号＋文件索引识别同一文件对象。`GetFileInformationByHandleEx(FileStandardInfo)` 的 `AllocationSize` 用于报告文件分配量，`EndOfFile` 和 `NumberOfLinks` 分列；不能用逻辑长度回退掩盖读取失败。接口定义见微软的 [文件身份](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle)、[FileStandardInfo](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_standard_info) 和 [元数据访问](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)。本次为本地 NTFS、默认数据流；没有命名流枚举、卷元数据/共享块盘点或删除实验，**不是卷实际占用或可回收空间证明**。

正式扫描前在忽略证据根创建全新普通/空文件及两个硬链接，11 项真实夹具全部通过：同一对象身份相等、普通与空对象不同、路径与对象逻辑大小分开、分配量不重复累加、根外链接仍保留对象、越界路径拒绝等。夹具保留，不链接已有缓存或用户文件；采集器只是一次性本机工具，不进入仓库正式门禁。

| 当前扫描量（2026-09-28 19:26 本地） | 实测 | 意义 / 边界 |
|-------------------------------------|------|-------------|
| 原生与扫描 | child **exit 0**，受管墙钟 8.15 s；实际元数据扫描 **5.68 s**，stop reason null、错误 0 | 完整枚举，未触发 60 s / 200 万项预算；不证明目录是原子快照 |
| 文件集合 | **105144 文件路径 / 7205 子目录**；文件对象 **83570**，重复路径 **21574** | 依 NTFS 文件身份去重，不以文件名、内容相似或 rlib 后缀判重复 |
| 按路径逻辑大小 | **243830499516 B（227.085 GiB）** | 与同次独立枚举相符；旧 225.447 GiB 为此前时间点，不能混作当前分配基准 |
| 唯一对象逻辑大小 | **219659452382 B（204.574 GiB）** | 所有内部路径只计同一对象一次；不是文件系统分配量 |
| API 报告的默认数据流分配量 | 按路径 **244007383752 B**；按对象 **219792700104 B（204.698 GiB）** | 两者差 **24214683648 B（22.552 GiB）** 是同一对象被路径重复累加的量，**不等于删除可回收量** |
| 硬链接与特殊文件 | `NumberOfLinks > 1` 路径 **43145**；按观察路径少于 link count 的对象 **0**；压缩/稀疏对象均 **0** | 仅这次读取的计数与属性；不推出所有时点根外无链接、无命名流或各变体可删 |
| 独立对照 | Node 逐项账本重算全部相符；libuv 第二次枚举 **1.42 s**，路径集合/数量、逻辑大小、link count 与修改时间全部相符 | 无遗漏、额外项或观察到的元数据变化；仍不是锁定整个目录或卷的快照 |

逐项原账本 `target.metadata.tsv` **28698237 B / SHA256 `1481DCC373910317C32BAD9D373C674F77933D4310DE1097453084F3AB8564F9`**；原扫描 `s2-allocation.json` **1407 B / `9E5A8402A0D2B834895F48CE2826C25D63FB619CD94518FC2065E4E7659AF3BD`**；独立核算 `s3-independent-verification.json` **1059 B / `E600903F948C8636FC8520D68500F17121988E549CC9730BD29ACB836FB61E95`**。夹具结果、源码、原 stdio/native exit 和身份汇总分别保留，不以派生 JSON 代替原始事实。

**采集层登记**：launcher 的两条 `CopyToAsync` awaiter 产生 `VoidTaskResult` 文本，仅多打印到 launcher stdout；结构化原生 exit 与扫描账本没有混入该对象，独立核算相符。保留原工具字节/结果，不据此补跑扫描或改写旧证据。编译器前后点检查为空，不能据此承诺期间绝无其他短进程。

**本地出口与接续**：有界元数据基线完成，K-BUILD-06/07 仍 OPEN；按对象分配量的认识更新不构成维护策略验收。下一步形成逐路径候选之前，必须核缓存的实际复用/重建成本、配置变体及冻结证据用途；不按日期、最新文件或统一后缀自动删除，也不改 jobs/linker/profile 或运行压力构建。本轮默认/改文链接、旧路径、文档登记、债结构与 diff 六项 exit 0，四文编码通过；已测 CLI/生产源码未变，目标 CI 独立绑定，在父运行终态后同步分类提交。

## 缓存用途与保留准则（2026-09-28）

接续 base `2da472cae97638af9cd4141bf4001f9b3352dfb4`，开场干净；该 SHA 的 [CI 36416485540](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36416485540) 已 **success，17/17 jobs 与 ci-gate 通过**，不替代本轮未来提交的 CI。合同见 [ROUND-02-PLAN](../ROUND-02-PLAN.md#缓存用途与保留准则2026-09-28元数据接续)。新证据根 `.cursor/plans/debt-cache-retention-20260928-r0/` 为本机忽略目录；缺原件仍记 `needs-evidence-access`。

本次复用上节 19:26 的冻结元数据账本，先核原 hash，再按目录组与文件对象归集；没有再次扫描或把旧时点写成实验后的当前大小。另读两次已完成 Cargo stdout 的 artifact filenames、实际命名消费者及从 head34 的 `additional_protection_sources` 可达 JSON。每文件 4 MiB、累计 64 MiB / 60 s；10 个保护源 JSON（连同两份 Cargo 日志读入 1,416,704 B）完整读取、错误 0、耗时 0.94 s。三行纯内存共享对象夹具通过；采集不进入正式 gate，也不代替原保护链全量核验。

| 旧扫描组 / 用途观测 | 结果 | 判读 |
|----------------------|------|------|
| `debug/deps` | 17147 路径/对象；对象分配 149698431128 B | 编译库与测试目标变体；不以相同包名或后缀认定冗余 |
| `debug/incremental` | 60030 路径 / 38883 对象；对象分配 61505094392 B | 增量编译辅助状态；内部格式与 Cargo 版本相关，不自行改写索引 |
| `debug/build` | 3749 路径 / 3452 对象；对象分配 1269357080 B | build-script 产物可能参与当前构建，不能统一按月删除 |
| 跨目录组硬链接 | **129 对象**共享 2586464256 B | 组别分配量不能直接相加；整根去重仍是旧扫描的 219792700104 B，不当可回收量 |
| 显式消费者关联 | **991 路径 / 991 对象**均能对应旧扫描，分配 3693051904 B；遗漏路径 0 | 只证明这两次构建及命名路径的显式引用；其余路径未分类，不能以未引用作为删除许可 |

Cargo 的 [build cache 说明](https://doc.rust-lang.org/cargo/reference/build-cache.html) 可辅助理解 `deps`、`incremental`、`build` 的角色，但内部布局可变化。下面按证据/消费者分类，不把目录结构当永久 API，也不生成清理白名单。

### 历史冻结路径与独立副本

head34 的 `/frozen_host_binary` 登记日常 `target/debug/oclive-kernel-server.exe` 为 **31366656 B / `460442009DC2004D3A0F3885D7E161D84DB3E3061F7154A6D7F1F763375B1F5D`**。本次该可变路径为 **31375360 B / `FDCF61DDF10B92F8EF7A1D4F62E83D8A7D5E545CF9A124F3AD2EAE1C5449056B`**，身份已不同；不推断具体哪次构建改变，也不声称旧 checker 对当前 HEAD/status 可以通过。

原独立副本 `.cursor/plans/cp-int-b10-tools/frozen-oclive-kernel-server-46044200.exe` 实测 **31366656 B / `460442009DC2004D3A0F3885D7E161D84DB3E3061F7154A6D7F1F763375B1F5D`**，精确匹配历史身份，保留原件。此为旧程序字节仍可取得的证明；没有回改 head34 路径、补造原始链完整性或把可变缓存升级为冻结证据。当前交付案例的限定验收不因这次只读关联扩大。

### 四类保留与维护准入

| 类别 | 当前处理 | 维护前必须补齐 |
|------|----------|----------------|
| **不可变证据**：冻结快照、日志、账本、二进制及关联 hash | 保留独立原件及身份；不要只依赖会被 Cargo 更新的 target 路径 | 另定保全/可携带性任务；旧头与正文不回改，新位置只能声明式登记并逐字节核对 |
| **显式消费者**：当前 Cargo artifact、调试/测试所用 exe/PDB、工具实际引用 | 保留路径及同一对象全部别名；先核进程、配置、特性、profile 与实际消费者 | 完整列出关联者及替代路径，取得消费者停用/切换证据；不能按 mtime 代替使用状态 |
| **可重建辅助状态**：incremental、build、fingerprint 等 | 暂保留，不自动维护；本机有界基准仅覆盖所列目标 | 新尝试须核源码/工具链/特性、冻结引用、重建正确性与时间/磁盘/内存预算、失败停止与回退 |
| **未分类/范围外/读取失败** | 保留并标未知；无引用、Git ignore、相同后缀均不改变准入 | 补实际用途与文件对象关联；未能证明不使用时不删除，不扩大扫描或默选保留“最新一个” |

任何后续实际维护必须逐路径验证绝对范围与 reparse、按文件对象列全部硬链接、保全所需不可变字节、限定操作和重建预算，再取得对应范围的准入；本轮没有删除、压缩、搬移、链接改写或清理任务。Windows 中英文附录撤回默认整夹删除、全量 `cargo clean`、按名杀进程及全目录安全软件排除，改为按具体报错文件和自己的会话诊断。

**收口**：K-BUILD-07 仍 OPEN；用途分类和维护前提已落文，实际维护频率、删除集合、可回收量和冷重建仍未验。K-BUILD-06 保持独立开放，单目标 linker 候选按另一个合同对照，不因本页静态规则自动更改默认构建配置。静态可用新日志安全复核，旧记录/DB/运行树/二进制不改；本机派生摘要不能替代原日志或远端目标 SHA CI。

派生用途件 `s1-retention-observation.json` **9754 B / `1A09983808A2BA5570C3549F0E68C25BF4CB9889AA955F861E93185FB47209D9`**；`s2-source-recheck.json` 独立复核两份 Cargo 日志与 10 个源 JSON 的 bytes/hash 全同。原 28 MiB 元数据账本不改，历史二进制内容只为精确身份匹配读取；没有触碰原 DB 或业务树。

## 单 Host 末端 LLD 候选（2026-09-28）

测量固定在 base `2da472cae97638af9cd4141bf4001f9b3352dfb4`，当时仅本轮文档 dirty；上页保留规则已另提交 `96b67fd4`。本节不把后来文档提交身份写回旧二进制。本机新根 `.cursor/plans/debt-host-linker-ab-20260928-r0/` 与 `candidate-02/` 独立保留；合同及参数修正见 [ROUND-02-PLAN](../ROUND-02-PLAN.md#单-host-末端-linker-候选对照2026-09-28独立实验)。没有旧 response-file 重放：其 461 个绝对输入中 258 个临时输入已不存在，读取历史参数不等于可运行复验。

**实际命令**：A 为 `cargo rustc --locked --offline -p oclive_kernel_host --features tauri-commands --profile test --test a_turn_harness -j 1 --message-format=json`。修正后的 B 仅附加 `-- -C linker=<随 Rust 安装的 rust-lld.exe 绝对路径> -C linker-flavor=lld-link`。依据 [Cargo rustc](https://doc.rust-lang.org/cargo/commands/cargo-rustc.html)，额外参数只给所选末端目标；不是 workspace 命令，也没有全图 `RUSTFLAGS`、依赖/锁文件、默认 profile 或并发修改。新包选择的实际非末端 artifact 集合由本组 A/B 比较，不预设与旧 workspace 完全相同。

### 原生结果与等价范围

| attempt | 实际完成 / 新鲜度 | 口径 |
|---------|--------------------|------|
| A · 默认 MSVC | native **0**、Cargo 完成 true、diagnostic 0；受管 **288.54 s**；441 artifacts：406 fresh / 35 non-fresh | 此次单包选择与提交溯源触发正常重编；包含依赖成本，不是纯热链接耗时。保存独立 exe/PDB 后才进入 B |
| B · 初版 `msvc-lld` | native **101**、Cargo 完成 false，**0.51 s**；一个编译器 error、linker 观测 0 | 稳定 rustc 拒绝该值，要求 `-Z unstable-options`；不是链接器/产品失败。保留原 S0/stdout/stderr/exit，不把本行改成通过 |
| B · `candidate-02` / `lld-link` | native **0**、完成 true、diagnostic 0，受管 **9.24 s**；441 artifacts：440 fresh / 1 non-fresh | **440 个非末端**的 package/target/features/profile/文件集合与 A 精确相等、non-fresh 0；仅所选 test 重编。实际 driver 包含 `-flavor link`，没有开启 unstable/nightly/BOOTSTRAP |
| 两份独立 binary | 两次 list **94 tests / 0 benchmarks** 且逐行相等；各自默认 **75 passed / 0 failed / 19 ignored / 0 measured / 0 filtered**，native 均 0 | 当前源码匹配固定 base；全部 ignored/live parent 未运行，未消费 CP-INT 身份、未触发真实模型/网络/音频。默认测试不证明全部业务/平台/崩溃窗口 |

前置观察器两真实函数 8 项离线探针通过。初版版本探针只走 `--version`，没有检查真实编译的参数接受性；失败后先核 [rustc 稳定接口](https://doc.rust-lang.org/rustc/codegen-options/index.html#linker-flavor)，在新根进行最小 metadata 编译三项正负控：`lld-link` exit 0 并生成 rmeta；旧 `msvc-lld` 和非法值各 exit 1、不生成 rmeta。补订仅一次 candidate-02 / 120 s，未重跑 A或继续扩预算；原 B 失败继续入账。

**计数与编排自纠**：第一次辅助检查错误沿用历史 89，exit 1；控制调用未先处理该非零，仍启动了 A 的默认测试。真实 list 两边都是 94，A 已完成的默认结果为 75/19；纠正比较为“实际两清单逐行相等＋当前 Host 跟踪源码匹配 base”后才启动 B 默认测试。该缺口属于本次读数/编排，不称旧检查通过，也不把实际测试改写为 89/70；原异常只在工具会话，未补造原始日志。两次执行始终未传 `--ignored`，原结果与 count 更正都保留。

### linker 样本、调试信息与证据

每边一个 PID＋创建时间身份，其 `/OUT` 正规化后精确绑定对应 Cargo executable。两份 response-file 均 134994 B、493 非空参数，保留各自 SHA；均有 `/DEBUG` 和 `/OPT:REF,NOICF`，不声称整份输入路径/临时对象字节相同。实际 test features 为 `default, tauri-commands`、opt-level 0、debuginfo 2；不能将依赖或 Host 包选择差异忽略。

| 绑定目标的点采样 | MSVC `link.exe` | Rust 随带 LLD 22.1.6 |
|------------------|----------------|---------------------|
| 最高工作集样本 | **3010666496 B（2.804 GiB）** | **2250588160 B（2.096 GiB）** |
| 最高私有内存样本 | **1615962112 B（1.505 GiB）** | **771567616 B（0.719 GiB）** |
| 首末可见点间隔 | 2.20 s | 1.70 s |
| exe / PDB | 34249216 / 269701120 B | 34245120 / 301629440 B |

250 ms 只是目标采样间隔，实际含 CIM 开销；表中间隔不是完整 linker 耗时，最高点不是真峰值，不推断 OOM 根因/全矩阵峰值或计算稳定加速率。A 总墙钟包含 35 个单位重编，B 非末端全部复用，不能用 288.54/9.24 算 linker 加速倍数。LLD PDB 增加 **31928320 B（30.449 MiB）**；保存和加载默认测试不证明调试器/PDB符号解析等价，此项在采用前单独验证。

四份 exe/PDB 独立副本累计 **639824896 B**，低于 1.5 GiB。A exe **`5D59BCFA21F47FF9FB667EFCCCC91789952CC39CA4CB71D23A0BBFEA35D4E7ED`**；B exe **`8348E805A0E3ACF0B7E19FB8B1A804C7F738250ECB5EE3E2CF0F4474D80DB687`**。原 native/点采样 A `baseline.exit.json` **266560 / `5563385C66B29A30B00A339D7CA6F887CFCE24496F08545625D26A7FE2B7C6CA`**；B `candidate-02/candidate.exit.json` **79372 / `C916618BAB6FB342AF3608A1EFE97C17E2BECC560E3F70A5BBD213252971B3E0`**。派生 `s4-comparison-summary.json` **10146 / `85505AA9E01D71CAC1F0753C69D61A013F6E9AA77FD7CBC7BBA6B83D0B20973A`** 含 22 份输入/原日志/退出身份，不替代这些原件。

**收尾与采用边界**：12 份源码/构建/CLI 输入、当前缓存 server 与旧独立冻结副本 bytes/hash 均不变，限定两项父环境不变；收尾编译器点检查空，未触发内存/时间止点。默认 Host/CLI/桌面构建、profile、`-j 1`、lock、生产 API与门禁组合没有修改。K-BUILD-06/07 仍 OPEN；本目标候选可编译且默认非 ignored 行为对照通过，不能据此全局采用、取消串行或宣布历史 OOM关闭。后续只选一个明确目标/特性/调试检查与新预算，避免重跑本组以堆样本；副本获取失败仍 `needs-evidence-access`。

**本地文档出口**：默认/四篇改文链接、docs 旧路径、文档登记、债结构与 diff 六项 native exit 0，四文 UTF-8 无 BOM/无替换符。Windows 镜像已在前一提交通过，此次未改；没有增加工程门禁、生产配置或公开 API，不为纯测量再跑无关全仓 Rust/业务链。新目标 SHA CI 仍独立登记，父 CI 绿不替代本提交结果。

## 已冻结 Host PDB 的离线符号消费（2026-09-29）

**起点与对象**：开场 `c60a04c99459025f7279a923676b1397fa9c8ba6`、工作树干净；该 exact SHA [CI 36424090472](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36424090472) completed/success，17/17 jobs 含 ci-gate 成功。本节消费上节固定 base `2da472ca` 的四份独立 exe/PDB，没有把文档 HEAD 冒写为构建身份；Host 跟踪源码相对构建 base 的 `git diff --quiet` 为 0。合同见 [ROUND-02-PLAN](../ROUND-02-PLAN.md#已冻结-host-pdb-的离线消费2026-09-29)。

**工具范围**：PATH 和有限安装范围（Build Tools、Windows SDK、VS Code 扩展、Rust bin）未找到 CDB/LLDB，不外推整机不存在；实际找到 MSVC `dumpbin.exe`、DIA DLL 与系统 DbgHelp。本次使用绝对 `C:\Windows\System32\dbghelp.dll`，**2307568 B / `CBB0C77B812E09F6AE66F482E8EAE94FD2E232BBBC3825D12668F3D2ED97E78E`**，版本 `10.0.26100.9444`；SDK `10.0.26100.0` 头文件核 x64 ABI，模块/行/符号结构为 1680/40/88 B、符号名偏移 84。DIA 的存在不是另一消费者的执行证据。

**真实调用**：本批一次性 helper 以独立非零会话标识调用 `SymInitializeW(..., false)`，仅静态读取指定 PE，不执行受测 exe、枚举进程模块或附加用户进程。`SymLoadModuleExW` 后读取模块，再用 `SymEnumSymbols("*dto_json*")`、`SymGetLineFromAddr64` 定位。每个 helper 串行、60 s 上限、独立 native/stdout/stderr/report；四次 `SymCleanup` 都成功、所持 child 都退出。依据 [SymSetOptions](https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/nf-dbghelp-symsetoptions)，采用精确匹配和行信息，忽略 CodeView 原路径及环境符号路径，搜索根仅本批本地目录；不启 deferred/load-anything/导出回退，不配置 `srv*` 或在线符号路径。模块注册成功也不等于加载了符号，实际类型和路径必须另核。

### 正例与真实负控

| case / helper native | 实际符号状态 | 独立核算 |
|----------------------|--------------|----------|
| MSVC 正例 / **0** | `SymPdb=3`；实际 PDB 路径精确为原独立副本，`PdbUnmatched/DbgUnmatched=false`，GUID `0996c52a-7cbe-4c3c-b936-af224cc5decc` / age 1 | 唯一完整函数名 `a_turn_harness::http_idempotency::dto_json`，`SymTagFunction=5`、477 B、RVA `0x93870`；源码 `http_idempotency.rs:327`、位移 0 |
| LLD 正例 / **0** | `SymPdb=3`；实际 PDB 路径精确为 candidate-02 独立副本，匹配标志同上，GUID `e521997b-f0a4-df97-4c4c-44205044422e` / age 1 | 同一完整函数名、函数大小/RVA/源码行与 MSVC 相同；这是一处函数定位对照，不是全部符号表等价 |
| LLD 缺失 PDB / **0**（预期拒绝通过） | 保留同字节 PE 的新普通副本，搜索目录无 PDB；模块登记非零但 `SymNone=0`、实际 PDB 路径空、无行/符号 | 没有从原 CodeView 缓存或其他原件找到正确 PDB；不把模块 `load_error=0` 称为 PDB 成功 |
| LLD 错配 PDB / **0**（预期拒绝通过） | 新普通 PE 副本旁放 MSVC 原 PDB 的同字节副本，改名为 LLD 预期文件名；仍 `SymNone=0`、路径空、无行/符号 | 两份 PDB 的 GUID 不同，错配副本 hash 精确等于 MSVC 原件；没有以文件名、导出符号或匹配标志 false 冒充成功 |

两次 `dumpbin /headers` 各 native 0，PE 的唯一 RSDS GUID/age 与正例实际 PDB 信息一致。独立 Node 核算从原 report/exit/headers 读取，**4 cases 通过**；它精确选择完整函数名及第 327 行，不以 `dto_json` 的两个闭包或泛型 `map_err` 作为替代。宽掩码实获 5 符号，其中 3 项落在该源文件 327–331 行；摘要中的“函数”计数是唯一完整函数 1，不混用两种计数。

**身份与预算**：新根 `.cursor/plans/debt-pdb-symbol-20260929-r0/` 输出 create-new；负控三份新普通文件累计 **338191360 B（322.524 MiB）**，低于 700 MiB，不使用硬链接、不删除或改名原件。整个 prepare/probe 流程 native 0 / **12.41 s**；独立核算 native 0，**49 次输入身份核对**（含重复引用，不称 49 唯一路径）全同，涵盖原 22 项原始证据、四份 binary/PDB、源码/构建/CLI、两个受保护 server、系统 DLL/SDK及 helper。源码 **49955 B / `8FDF1FAEA02EA6A7DD49186B71552E15FD335315BFC9EA24A14AACE2801B49DA`**；无 Cargo/Rust 重编译或业务回合/身份、真实模型/语音/业务网络。

`s0-inputs.json` **12092 B / `F59EC9CF023841D026D766D7FE4D6DEE80EFE306ADE58213AFAE71203B8B5E25`**；独立派生 `s3-independent-summary.json` **20629 B / `E3F6724046F482B593E5733202A6FE4088CD74ABEB2342864E898690531304F3`** 含原件身份，不替代原生 stdio/exit。四份 case report 分别为 **4374 / 4434 / 1451 / 1454 B**；原始及 helper 仍在忽略目录，无法携带时保持 `needs-evidence-access`。

**收口与下一决策**：本目标两份固定产物通过本机 DbgHelp 的精确 PDB、所选函数及源码行消费；[符号枚举](https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/nf-dbghelp-symenumsymbols)和[行定位](https://learn.microsoft.com/en-us/windows/win32/api/dbghelp/nf-dbghelp-symgetlinefromaddr64)是本次调用范围。`TypeInfo/GlobalSymbols` 元数据标志没有被外推成类型/局部变量实际读取；未设置断点、观察调用栈/运行中局部变量或运行 CDB/LLDB，不声称完整调试器兼容。K-BUILD-06/07、默认 MSVC/profile/`-j 1` 与队列不变；全矩阵 OOM及实际缓存维护仍未关闭。若评估采用，先限定一个实际构建入口并确定保留/调试取舍；安装工具或默认切换另行准入，不为了已有正例重复编译/业务场景。
