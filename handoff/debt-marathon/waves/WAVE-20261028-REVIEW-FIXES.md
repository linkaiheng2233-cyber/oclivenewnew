# 2026-10-28 · 复核问题分批修复

**入口**：[第三方清单复核](WAVE-20261027-DEEPSEEK-RECHECK.md)。用户已授权按该报告建议顺序实施。基线 `c9360444d0f5f4d5f87e2a5632c62731a7e2cd7e`；开工已有一份未提交的复核报告，保留原件。尺寸 L：涉及持久化与权限。controller 自查，不派 Agent；尚无新的远端写入授权，不推送、不发版、不把技术债改成 Done。

## 计划与冻结点

| 批次 | 范围与最小完成条件 | 状态 |
|---|---|---|
| A 数据与权限状态 | KER-02 导入同址/重叠拒绝、先复制校验后替换及恢复；KER-03 删除失败与重启可判定；KER-10 grant/revoke 落盘失败不改变内存，持久化原子替换 | 定向本地验证通过；整合门禁待最终冻结 |
| B 外部输入 | SEC-06 标准 URL 解析及拒绝回归；KER-13 实际展开字节/条目预算；SEC-08 最终路径边界；条件深链预览确认，不扩大默认功能 | 定向本地验证通过；整合门禁待最终冻结 |
| C 默认发行可靠性 | UI-07 通过认证 Host 读取快照与 turn freshness；UI-01 生产诊断/局部恢复；UI-08 取消后呈现与恢复约束 | 核对已有差量后接续 |
| D 已确认文档错误 | README、中英兼容说明、实际失效代码锚点、OOCP 工作目录、缓存数字；Studio 归属保留待决，不盲改入口 | 待 C 验收 |

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

A 本地提交：`d0697780`。前置只读复核报告单独提交 `20c7a30b`，未推送。

## B 实施记录

- RPC 改真实 URL host 解析，拒绝 userinfo；本机 HTTP 池不跟随重定向、不继承代理，创建失败保存错误并在调用时拒绝，不再退回可能宽松的客户端。静态资源核最终 canonical 范围及各路径组件，保留扩展名回退。
- ZIP 限 20,000 项／单项 256 MiB／总计 1 GiB，实际读流有硬预算，预览 4 MiB；tiny ZIP/reader 夹具验证计数、单项、累计和实际读取超限，没有制作大体积炸弹。
- 深链保留默认功能开关，新增 HTTPS 来源校验与安装前明确确认，取消不装，异步确认后重查开关。没有新增域名白名单或可信作者体系，未声称能在安装前完整审查 manifest 权限；安装后仍有既有源码/权限提示与独立高风险授权。
- Host lib 首跑 671/0/0，通过本机 redirect/proxy、userinfo 拒绝和 Windows junction 夹具。条件入口 11 个测试通过。最初调用了不存在的 Vitest config，后在 node 环境访问 window 失败，均为本轮测试启动/夹具问题；改用 workspace 脚本和 jsdom 后通过，原日志保留。clippy 首次发现测试 socket 未处理 read 长度，已改有界读取循环，待最终重跑。
