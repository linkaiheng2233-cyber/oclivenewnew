# 2026-10-28 · 复核问题分批修复

**入口**：[第三方清单复核](WAVE-20261027-DEEPSEEK-RECHECK.md)。用户已授权按该报告建议顺序实施。基线 `c9360444d0f5f4d5f87e2a5632c62731a7e2cd7e`；开工已有一份未提交的复核报告，保留原件。尺寸 L：涉及持久化与权限。controller 自查，不派 Agent；尚无新的远端写入授权，不推送、不发版、不把技术债改成 Done。

## 计划与冻结点

| 批次 | 范围与最小完成条件 | 状态 |
|---|---|---|
| A 数据与权限状态 | KER-02 导入同址/重叠拒绝、先复制校验后替换及恢复；KER-03 删除失败与重启可判定；KER-10 grant/revoke 落盘失败不改变内存，持久化原子替换 | 定向本地验证通过；整合门禁待最终冻结 |
| B 外部输入 | SEC-06 标准 URL 解析及拒绝回归；KER-13 实际展开字节/条目预算；SEC-08 最终路径边界；条件深链预览确认，不扩大默认功能 | 定向本地验证通过；整合门禁待最终冻结 |
| C 默认发行可靠性 | UI-07 通过认证 Host 读取快照与 turn freshness；UI-01 生产诊断/局部恢复；UI-08 取消后呈现与恢复约束 | 定向本地验证通过；整合门禁待最终冻结 |
| D 已确认文档错误 | README、中英兼容说明、实际失效代码锚点、OOCP 工作目录、缓存数字；Studio 归属保留待决，不盲改入口 | 定向文档检查通过；进入最终冻结 |

不将 56 项全数变成修复清单；签名体系、公开发布、分支保护、Studio 产品迁移与冻结功能保持原状态。SEC-03/04/05/07 等更广信任策略沿已有边界记录，不能借小修宣称全面安全。

## 关联链与验证

- A：角色输入→Host import/delete/DB→既有 Tauri/HTTP 调用→角色库；错误沿现有 AppError/CommandError，不新增六槽 DTO/trait 或业务编排。授权 store→持久化→现有 native/HTTP 消费者；拒绝路径不能失效。
- B：ready/asset/archive/deeplink→解析与权限→RPC/安装/UI；允许/拒绝都要验证。插件入口要跑 module-compat。
- C：snapshot 生产者→现有认证 Rust client→Tauri/shared→Chat Pro/Theater 状态；轮询角色/轮次/卸载边界；错误落地复用已有脱敏层。
- D：只改各自 owner 文档，中英联动，不复制模块表或新造版本真源。

开发切片跑定向 Host/前端用例，保留实际失败首因。A 与 B 为独立高风险验收点，可分别冻结；全部预定范围稳定后统一跑适用本地整合检查。Rust 分层、错误码、module-compat、fmt/clippy 与受影响测试按差量执行；公开 API 变化补 doctest。文档跑默认/显式链接、stale、编码、mirror 与 registry。未获 push 授权时交付最多 Locally verified，远端 CI 保持未验。

原始日志与合成夹具置于 `E:\OCLive\oclive-dev-artifacts\review-fixes-20261028\`。不对用户角色、授权、模型、旧包或工作树做故障注入／清理。

## A 实施与验证记录

- 授权改为复制快照→同目录临时文件完整写入/sync→原子替换→提交内存；grant/revoke 都遵循同一顺序，不修改原授权 JSON 格式、权限 ID 或调试开关。拒绝／落盘失败保留原值。
- 导入使用 roles 根的跨进程文件锁、暂存新包、原 loader 校验和旧包备份。恢复只检查本机制的记录，源目标重叠拒绝、遍历错误不再吞掉。纯蓝图包首轮回归失败是本轮暂存目录名 `next` 违反既有 R4 校验；现改为 `next/{id}`，未放宽校验。原失败日志保留，修正后 15 个角色定向测试通过。
- 删除使用既有 app_settings 保存恢复记录，无新 SQL migration；删除数据和 committed 标记在同一 SQLite 事务中。提交前错误/取消/中断留可恢复文件；提交后清理失败保留记录并返回可选 `cleanup_pending`，不再报告成删除已回滚。重试/启动读标记决定恢复或清理。源码缓存只在提交成功后清理。
- Host lib 本轮实跑 664 passed / 0 failed / 0 ignored，包括 Windows 目录 sharing denial、原授权文件 sharing denial、DB abort、提交前恢复和提交后不恢复；均为合成夹具。不是实际进程断电/kill 故障注入，也不证明所有硬件持久性或跨进程聊天并发窗口。
- 已通过 domain-layering、error-codes drift（22 静态码对齐）、module-compat、全项目 vue-tsc、Host all-targets/all-features clippy -D warnings、中英镜像及本轮文档链接/编码检查。clippy 首次因本轮删除分支留下可合并的嵌套 if 而失败，折叠后通过，没有放宽 lint。最终 Host lib 再跑 664/0/0；整合门禁与最终总审待后续批次冻结，不入账 Done、不发新包。

## B 实施与验证记录

- A 已本地提交 `d0697780`；复核报告独立提交 `20c7a30b`。B 不扩大默认高级功能。
- RPC 标准 URL/IP 判定拒绝 userinfo；本机 HTTP 客户端 no_proxy 且不跟随重定向，构造失败闭合，不退回可能走代理的客户端。私有传输状态承载构造错误，公开构造签名不变。
- 资产返回最终 canonical 路径并拒绝链接组件；实际 Windows junction 夹具验证逃出根被拒绝。角色压缩包加入条目/单项/总展开限制、实际读取预算与小型预览元数据预算，未制作大体积压缩炸弹。
- 深链仍先入队；安装前展示 HTTPS 仓库来源并确认，拒绝携带凭据/query/fragment 的来源，确认返回后再查高级开关。取消不安装。完整 manifest 预审及可信作者体系仍未实现，不宣称解决已有签名/信任债。
- Host lib 初跑 671/0/0，包含本机代理/重定向、URL 分类、ZIP 预算及 junction 测试。后续 clippy 的读取长度提示已按实际读取处理，不抑制 lint。
- 前端首个命令误指定不存在的 vitest.config.ts；随后实际配置下因新夹具没有 window 而失败（Node 环境），加 jsdom 声明后 11 个确认/拒绝/队列用例通过。两次失败日志保留，不归因产品缺陷。
- 最终 Host lib 671/0/0、Host all-targets/all-features clippy -D warnings、vue-tsc、B 的 ESLint、module-compat、中英镜像和 5 份改文链接通过。无 Cargo.lock、公开 Rust DTO/trait 或高级功能默认开关变化。
- 在 C 开工前发现三个 C 源文件和两个新测试存在未记录在当前工具轨迹中的变更，未覆盖/删除；原件及哈希保存在 artifact 的 `C-existing-716b185961c344a3aeb21e0a1a85e34f`。已询问是否有并发工作；当前 Agent 树和 app 列表只见本任务活跃。B 提交明确排除这五个文件，C 先核对既有差量及稳定性。

## C 实施与验证记录

- B 已本地提交 `a7645ffb`。既有五份 C 文件在保全后多次核对哈希未再变化，内容与已授权方案一致，按同一任务的已有差量审查接续；不冒称由本轮从零编写。原件保留，后续增加取消提示不覆盖原来的 revision getter。
- 快照复用 `get_role_info` 原生命令及其认证 Rust 路径；不新增 Kernel HTTP 路由/DTO，不向 JS 暴露 token。后台错误在公共脱敏诊断中抑制重复，避免每 8 秒弹前台错误。请求回来后核角色、场景、轮次、poll generation 和卸载状态。
- 主工作区域增加无正常态额外 DOM 包装的渲染边界，Chat Pro/Theater 入口还有根边界；重开区域不自动业务重试。Vue、window error/unhandledrejection 记录到现有 bounded session diagnostics，卸载释放事件监听，不上传或持久化聊天。捕获 falsy thrown values 也显示恢复卡片。
- 取消/被新发送替代时，清除临时助手气泡并在原角色/场景放一个不落 transcript 的系统提示；不声称服务器已取消，不自动重发，也不伪造成功事件。已登记的服务端取消/恢复剩余窗口保持原状态。
- 定向 36 测通过。首轮 1 个失败来自本轮把取消断言插到 batch-presentation 用例的错误位置；移到实际取消用例后通过，不作为旧产品红证据。lint 首轮失败为本轮导入排序/排版，按规则修正后全量通过。
- 全量前端单测：shared 411、Chat Pro 139、Theater 53 全过；全量 ESLint、vue-tsc、Theater prompt/skeleton smoke 通过。后续生产构建、浏览器与全仓整合门禁在最终冻结执行，不用这些 mock 结果冒充 native Windows/真实模型证据。
- C 生产构建通过；构建脚本顺带重写无源码差量的 tracked plugin-bridge 产物，先保全到 artifact，再仅恢复该文件的原始 Git 字节，未覆盖用户差量。默认链接 52、本轮显式 6、中英 mirror、活跃编码、shared i18n 和 docs-only stale 检查通过。API polling 沿用既有原生 transport 的超时，不声称 browser 5 秒 AbortSignal 已保留。

## D 文档修正记录

- C 已本地提交 `0defa235`。README 中英首例改为依黄金路径准备角色包并编辑实际人设文件，不再暗示只新建错误路径即可载入。
- COMPATIBILITY 移除重复当前版本快照、旧主程序推荐与强制 CLI/内核一起 bump 的说法；当前值回链现状 SSOT，保留历史最低兼容表和公共兼容解释原范围。Studio 与现有附带编写器的交付对照仍待确认，不改入口、不称替代已验。
- 修正 repository trait、命令错误桥及 chat migration 三处代码锚点；保留复核判明有效的 handoff 相对链接。OOCP 明确 prefix/工作目录，未直接向用户运行中的服务执行该文档命令。移除 AGENTS/AI 索引中的缓存热路径数字和 CLAUDE 的索引行数；不更改测试实际条数。
- `CLAUDE.md` 实为本机已有、被 Git 忽略的入口。已保留并修正本机两处文字，不以 `git add -f` 改变跟踪策略；该差量单独保存到 artifact，不作为受检 Git 提交内的文件。第一次批量 add 因此返回非零，未运行 commit；其它暂存的 D 文件经核对后继续提交。
- 现状中英同步说明新修复只有源码，已有 rc.2 安装包不含这些修改；中英 Unreleased 记录实际行为，未加稳定版 tag。债变动事件只登记局部进展，主台账、QUEUE 与冻结项状态不变。

## 最终冻结与整合证据

首次在 `f120feb3` 的整合于 `runtime_event_stream_contract::stage_a2_review_fixture_freezes_non_runtime_boundaries` 失败：夹具仍把旧角色删除的两个实现 token 当当前证据。没有删断言或填注释伪造匹配，而是把审查证据扩为 4 项、明确要求当前服务与恢复实现两个锚点，并同步 RFC 中英对“当前实现”的描述；设计 stage、production=false、Session 擦除域、先阻断 ingress/lease、未实现 executor 的 known gaps 均保持不变。该关联修复不改 Kernel 公共 DTO/trait 或生产 Stream。保留首轮完整失败日志，定向通过后重新冻结并重跑整合。

本页上文“待冻结/待整合”为各切片当时状态；最终命令、退出码、完整 Git SHA、源码文件哈希和未测边界集中保存在本机 artifact 的 `final-verification.json`，不通过只回写结果的提交再移动已测源码。未取得该文件中的成功证据不能把计划当作通过。

- 冻结后执行一次 `npm run check:ci-local`：本轮改 Host 持久化、权限/目录插件和桌面接线，需要全仓本地整合，而不是只依赖定向单测。
- 显式 `cargo test --locked --workspace --doc -j 1`：补公开消费者示例的兼容检查；不把 lib/clippy 当作 doctest 证据。
- 已通过的前端单测/lint/typecheck与冻结源码哈希对账；独立构建 e2e 产物后，以新空闲端口、安装的 Edge 跑浏览器壳回归。生产构建与 e2e 构建顺序执行，避免互相覆盖 dist。
- 最终文档/编码/差量复核及工作树清洁检查；任何构建脚本生成的无关产物先保全再恢复，仅限本轮自己产生且原本干净的文件。
- 当前没有 push 授权，不取得远端 CI 或修改保护；没有重打安装包、实际用户数据故障注入、收费模型、真机升级或签名验收。即使本地全绿，结论也只为限定的 Locally verified。

A 本地提交：`d0697780`。前置只读复核报告单独提交 `20c7a30b`，未推送。

## B 实施记录

- RPC 改真实 URL host 解析，拒绝 userinfo；本机 HTTP 池不跟随重定向、不继承代理，创建失败保存错误并在调用时拒绝，不再退回可能宽松的客户端。静态资源核最终 canonical 范围及各路径组件，保留扩展名回退。
- ZIP 限 20,000 项／单项 256 MiB／总计 1 GiB，实际读流有硬预算，预览 4 MiB；tiny ZIP/reader 夹具验证计数、单项、累计和实际读取超限，没有制作大体积炸弹。
- 深链保留默认功能开关，新增 HTTPS 来源校验与安装前明确确认，取消不装，异步确认后重查开关。没有新增域名白名单或可信作者体系，未声称能在安装前完整审查 manifest 权限；安装后仍有既有源码/权限提示与独立高风险授权。
- Host lib 首跑 671/0/0，通过本机 redirect/proxy、userinfo 拒绝和 Windows junction 夹具。条件入口 11 个测试通过。最初调用了不存在的 Vitest config，后在 node 环境访问 window 失败，均为本轮测试启动/夹具问题；改用 workspace 脚本和 jsdom 后通过，原日志保留。clippy 首次发现测试 socket 未处理 read 长度，已改有界读取循环，待最终重跑。
