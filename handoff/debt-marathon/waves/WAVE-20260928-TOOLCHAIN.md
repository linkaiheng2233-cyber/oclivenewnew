# 仓库工具链首批切片

**SSOT 范围**：K-TOOLS-01 / K-VERIFY-01 本轮仓库实施与验证证据；父债状态在 [主台账](../../TECHNICAL_DEBT_INVENTORY.md)，依赖和接手事件在 [DEBT_CHANGELOG](../DEBT_CHANGELOG.md#dcl-20260928-04--仓库工具链首批实施)。
**最后更新**：2026-09-28。

本轮为 M 级普通工程切片，不启动自动马拉松；使用 [ROUND-02-PLAN](../ROUND-02-PLAN.md#工程切片启动2026-09-28) 已核范围。base `32cd11a81a6fe7641b228fa8da2d39676d2791d8`，开场工作树干净。用户明确授权开始工作，未改 Kernel/Host/DTO/鉴权、依赖版本、`-j 1`、用户 PATH、个人环境脚本或旧冻结证据。

## 实际差量与消费者

| 节点 | 改动 / 结论 |
|------|-------------|
| 必需命令与计数 | [gate-toolchain](../../../scripts/lib/gate-toolchain.mjs) 只服务当前原生命令调用；显式处理缺失、执行错误、超时、信号、非零及坏输出。`rg` exit 1 无匹配才返回 0；NUL 文件名/计数分隔，保留 Rust glob 和匹配行语义 |
| 实际 ratchet | [分层](../../../scripts/check-domain-layering.mjs)、[Host re-export](../../../scripts/check-host-reexport-imports.mjs) 使用同一个 helper；排除 `domain/mod.rs` 按完整路径，避免盘符切坏；正常仍 3/3、FQ 1/1、75/75 |
| 项目体检 | [check-toolchain](../../../scripts/check-toolchain.mjs) 实跑版本、两个 ratchet、Git worktree 和 `cargo metadata --locked --offline --no-deps`；存在/可用/实际链通过分列，未运行 null。失败非零，不安装依赖 |
| 综合门禁 | `package.json` 新增 `test:toolchain` / `check:toolchain`；Dimension 5 调用同实现的 17 项合同测试。删除无调用的 `rgCount`；原先三处源码存在不等于三处实际调用，见 DCL 更正 |
| 无需改的关联端 | Kernel/Tauri/shared/发行版/插件/角色包没有使用该脚本内部 API；未新增 AppError 或六槽 parser，原门禁/基线文件保持。CI 的 Dimension 5 已安装 ripgrep、Node、Git 和 Cargo，沿用现有入口，无 workflow 改动 |

`check:toolchain` 的 PASS 只在 `scope=oclive-static-gates` 有效，不代表完整编译、MSVC/链接器、cargo-audit/deny、前端、真实音频或发版准备。历史个人环境报告不能替代它，反之也不自动更正个人报告。

## 实测与失败保留

本机证据根：`E:\OCLive\oclivenewnew\.cursor\plans\debt-toolchain-20260928-r0\`（Git 忽略，本机原始日志不可随 clone 获得；本页命令和仓库测试可复跑）。

| 命令 / 证据 | 实测 | 边界 |
|-------------|------|------|
| `s0-baseline.json`：两个原 ratchet 正常 / 子进程空 PATH | 正常 exit 0，3/1 与 75；缺 `rg` 均 exit 1、裸 ENOENT | 原源码 SHA 已登记；仅子进程改 PATH，父环境未变 |
| `node --test scripts/lib/gate-toolchain.test.mjs scripts/check-toolchain.test.mjs` | attempt1：16 passed / 1 failed；attempt2：17 passed / 0 failed，exit 0 | 首因是我将特殊 `process.env` 对象与普通对象比较；修为只比 PATH。失败日志对被打印的凭据脱敏，`s1-attempt1-redaction.json` 登记原/保留 hash，不称原字节保全 |
| `npm run test:toolchain` | 17 passed / 0 failed，exit 0 | 真 rg 匹配/无匹配/坏 regex/缺文件，真子进程缺 PATH/不可用文件/exit 7/超时，Windows 路径解析、实际 CLI 拒绝及体检正负控；不是另写一套模拟控制流 |
| `npm run check:toolchain` | exit 0；4 工具 present/usable/gate_passed 全 true，4 实际静态项目 gate PASS | Node v22.23.2、ripgrep 15.2.0、Git 2.55.0.windows.4、Cargo 1.97.1；新机器/完整工具链不由这些数字证明 |
| 里程碑综合门禁 attempt1 | `npm run check:ci-local` exit 1，67.02 s；Dimension 5 FAIL (29 checks)，只失败 voice TTS ratchet | 本机缺默认 `py -3` 启动器；其余 28 项通过（含设计内 SKIP），后续 `npm run check` / Rust 集成未执行；原始日志与 native exit JSON 独立保留 |
| 定点 Python 配置复核 | 实际解释器 `E:\Env\portable\python\cpython-3.12.14-windows-x86_64-none\python.exe`；`check-voice-tts-ratchet` exit 0 | 使用脚本已有 `OCLIVE_VOICE_PYTHON` 指定原生解释器，命令 finally 恢复原缺失态；与 [巡检手册](../../RECURRING_OPTIMIZATION_PLAYBOOK.md) 已知环境前提一致。不改语音源码/个人脚本、不安装；四工具静态 doctor 不承诺覆盖 Python |
| 里程碑综合门禁 attempt2 | `npm run check:ci-local` **exit 0，557.66 s**；Dimension 5 **PASS (29 checks)** → lint/typecheck/build → Rust fmt/clippy/workspace lib → workspace/CLI 集成全链通过 | 8 个工程输入按 `s2-source-freeze.json` 冻结；显式 Python 命令配置 finally 恢复为缺失，父 PATH hash 一致。日志 315409 B / SHA256 `47C2B20541AFAFF5C49A563E6CDFA9A86B7E43AFA9543272787D8D307F0DBAA3`；CLI 注入错误输出不等于测试失败 |
| 收口自测目录安全检查 | 两个测试文件仅补“清理前绝对路径仍在命名 Temp 根＋指定前缀”断言；17 passed / 0 failed，exit 0 | 综合测试时的两份原测试字节另留 `s2-input-*.test.mjs`；6 个非测试工程输入逐项不变，补丁定向复跑及最终 Dimension 5 分列，不把旧字节说成已测新字节 |
| 最终 Dimension 5 / 文档出口 | 收口字节 `dimension5 --ci` **PASS (29 checks)、exit 0**；4 本轮 Markdown 链接、文档旧路径及编码通过，登记/债计划结构和默认链接由最终 Dimension 5 实跑覆盖 | `s3-final-source-freeze.json` 与 `s3-final-dimension5.exit.json` 绑定最终 8 输入/日志；未重复无改动的 Rust 全链。Git LF→CRLF 提示与 CLI 预期注入日志不写成“零 stderr” |

人工语义审查覆盖最终命令调用、退出码和错误归因、Windows 路径排除、三态与未执行状态、父债边界。未运行外部 Bugbot；没有生产架构/安全语义改动。原 Dimension 5 文件混用 CRLF/LF，保留未改行的字节，新增行按 LF 写入；不为格式化扩大 diff。

## 交接与剩余条件

当前执行结论：**Locally verified（仓库切片）**；窄测、静态体检和配置明确的综合链通过。父债均 Partial，没有 Done 迁移。基线文档提交 `415d9592990002923ab1d2285ecad9bc998a01bd` 的 [CI 36396145057](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36396145057) 已 success，它只证明旧 SHA，不能代替本轮代码提交。最终提交 SHA 由 `git log -1 --format=%H -- scripts/lib/gate-toolchain.mjs` 取得，远端 CI 在交付报告绑定该 SHA，不为回写 run ID 再造未验证提交。

最后工程动作：综合链 exit 0，收口目录断言定向 17 项、最终 Dimension 5 29 项和文档出口通过。提交/推送后下一步核该 SHA CI；后续独立工程候选是 K-BUILD-06/07 的只读测量，先复用 [计划](../ROUND-02-PLAN.md#工作顺序与独立条件)，不删缓存、不改并发。本轮没有业务场景身份；静态检查可重试，失败尝试必须新日志。新环境完整链与个人体检联动仍按 owner 独立安排，不向本机证据补造外部通过。
