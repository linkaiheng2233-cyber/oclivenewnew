# 2026-10-28 · 复核问题分批修复

**入口**：[第三方清单复核](WAVE-20261027-DEEPSEEK-RECHECK.md)。用户已授权按该报告建议顺序实施。基线 `c9360444d0f5f4d5f87e2a5632c62731a7e2cd7e`；开工已有一份未提交的复核报告，保留原件。尺寸 L：涉及持久化与权限。controller 自查，不派 Agent；尚无新的远端写入授权，不推送、不发版、不把技术债改成 Done。

## 计划与冻结点

| 批次 | 范围与最小完成条件 | 状态 |
|---|---|---|
| A 数据与权限状态 | KER-02 导入同址/重叠拒绝、先复制校验后替换及恢复；KER-03 删除失败与重启可判定；KER-10 grant/revoke 落盘失败不改变内存，持久化原子替换 | 定向本地验证通过；整合门禁待最终冻结 |
| B 外部输入 | SEC-06 标准 URL 解析及拒绝回归；KER-13 实际展开字节/条目预算；SEC-08 最终路径边界；条件深链预览确认，不扩大默认功能 | 实施中 |
| C 默认发行可靠性 | UI-07 通过认证 Host 读取快照与 turn freshness；UI-01 生产诊断/局部恢复；UI-08 取消后呈现与恢复约束 | 待 B 验收 |
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
