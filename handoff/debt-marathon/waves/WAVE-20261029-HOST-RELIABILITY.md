# 2026-10-29 · Host 运行时与持久化可靠性续接

**范围**：[DeepSeek 复核](WAVE-20261027-DEEPSEEK-RECHECK.md)中的 KER-01 / KER-11 / KER-12。接续基线 `3609005bab74d8a8d24ac2191deb2157a04f512a`，开工工作树干净。用户授权继续修补，并把重新打包与内测包准备放在最后。本波次先完成这三项源码与验证，不提前替换已有包。尺寸 L（并发与持久化）；controller 实施/自查，`independent=false`，不派 Agent、不推送、不修改冻结功能或债务 Done 状态。

## 方案与验收

| 切片 | 实施约束与最小验收 | 当前进度 |
|---|---|---|
| KER-01 同步桥接 | 保留已有 generic future 的借用、非 Send 和输出约束；当前线程 Tokio 中的同步 RPC 在发请求前返回现有错误，不嵌套 runtime、不随意移线程；无 runtime、多线程及异步路径回归 | 已实施、窄测通过；整合见最终冻结证据 |
| KER-11 迁移恢复 | 保留原迁移错误及 `DB_MIGRATION_FAILED`；记录备份恢复成功/失败和 marker 写失败，不用新 JSON 协议替代既有 marker；用合成 DB/文件故障验证 | 已实施、窄测通过；整合见最终冻结证据 |
| KER-12 镜像顺序 | SQLite 仍为真源；同一存储根的镜像 IO 共用进程内队列，进入队列后读取 DB；默认后台任务不再拿旧增量覆盖重建；原子替换、取消中的写入及删除路径一起核对 | 已实施、窄测通过；整合见最终冻结证据 |

关联链：同步 adapter → JSON-RPC → runtime helper → 原 fallback/error 消费者；启动 builder → migration runner → 关闭 pool → 备份恢复/marker → 既有错误展示；聊天提交/编辑/删除/导出/清理 → mirror → 文件 IO。Tauri/shared/Chat Pro/Theater 的公开 DTO、六槽与角色配置预计无需改，仍须核对消费兼容。进程内镜像队列不等于跨进程锁，不承诺完整 Session 擦除或所有用户数据故障恢复。

CI 节奏：实施期只跑三项定向测试和相关 Host 测试，保留首因与原日志；写集稳定后统一执行 Host 库测试、适用 clippy/fmt、分层/错误码/模块兼容及文档检查，再分类提交、冻结 HEAD，执行一次 `npm run check:ci-local` 与显式 workspace doctest。前端无差量时复用前轮对应源码证据，不重复浏览器或真实模型测试。失败仅按根因修补，不能用删测试或放宽门禁完成。L 无远端 CI 时仍只交付 Locally verified。

原始日志、源码哈希及合成夹具证据置于本机 `E:\OCLive\oclive-dev-artifacts\host-reliability-20261029\`。不对用户角色、实际授权、聊天 DB 或旧安装包注入故障。重新打包、隔离安装/升级与内测包清单将绑定最后确认的源码基线，不能借用旧 rc.2 的验收。

## 实施记录

- KER-01：保留旧 generic helper 的公开形状和明确的 panic 契约；生产同步 RPC 改用 crate-private checked bridge，在不支持的 runtime 中先返回现有 `REMOTE_SERVICE_UNAVAILABLE`。新增无 runtime/多线程的借用非 Send future、当前线程拒绝且不 poll、不连接的测试。异步 adapter 未迁移、未新增线程调度器。
- KER-11：关闭 pool 后才恢复；缺失备份明确报失败，返回原迁移原因并附恢复结果。既有 marker 仍只有 `failed_at` / `message`，复用同目录原子写入；marker 写失败也留在返回错误和日志里。覆盖真实无效 SQL、恢复后原始行、缺失备份及恢复/marker 双失败，均为临时夹具。
- KER-12：去掉仅在旧默认路径使用的增量发布，排队后重新从 DB 取当前快照。进程内 canonical-root 队列共用于后台、手工重建、导出与删除；blocking IO 持有队列到结束，避免取消后提前解锁。关联核对进一步发现正常角色删除的文件隔离也须等待该队列，已补接并增加受控等待测试；恢复记录冲突、跨进程和完整 Session 擦除边界不扩大。
- 首轮 Host lib **682/0/0** 通过（包含新增桥接/迁移/镜像测试）；随后补正常角色删除的关联队列检查，当前尚待最终重跑。代码冻结、分类提交与全仓整合仍未完成，不能把此首轮结果用于新差量。
- 关联接线首次编译因本轮把内部队列函数声明为 `pub(super)` 却以 `pub(crate)` 重导出而失败；已改成 crate 内可见，不扩大到公开 API。原失败日志 `host-lib-associated.log` 保留，属于本轮接线错误，不归因于旧产品。
- 关联接线修正后 Host lib **683/0/0**、Host all-targets/all-features clippy 通过。语义复核补核授权拒绝的优先级：同步 RPC 先检查既有网络授权，发送前仍由共享 async sender 再查，避免 runtime 拒绝先触发 builtin 回退而掩盖授权拒绝；增加同一无连接测试中的授权负控，待最终窄测。
- 授权负控 1 测通过。最终库重跑出现 **682 pass / 1 fail**；补足错误正文后定点复现：pool close 后移除 `app.db-wal` 返回 Windows os error 32，而恢复诊断如实报告 `Backup restore failed`。本机 SQLx 0.8.6 源码核到异步归还与 worker 关闭路径，不能仅据 pool close 已返回就声称所有句柄即时释放。修补限定为 Windows 实际 sharing violation 32 每操作最多 200 ms 重试，其他 IO 错误立即返回；新增临时占用后释放与持续占用不破坏原 DB/备份的两项 Windows 实例测试。不是降低测试断言或无限重试，首失败与诊断日志均保留。
- 限定重试后迁移模块 **20/0/0**；Host 库 **685/0/0**、Host all-targets/all-features clippy、fmt 和错误码对齐（22 静态码）通过。镜像/删除受控等待及取消测试保持断言，SQL 失败恢复测试仍要求真实快照恢复成功，未改成“任意恢复结果均算通过”。

## 最终冻结与验收证据

上述数字分别属于各切片当时的测试。源码与文档分类提交后，冻结目标执行一次 `npm run check:ci-local`，再显式执行 `cargo test --locked --workspace --doc -j 1`；若失败则保留原日志，按首因修补并另存重跑记录。最终完整 SHA、命令、退出码、源码/日志哈希与复用范围保存在本机 artifact 的 `final-verification.json`。没有成功结果时不能把本计划当作验收通过，也不通过“只回写结果”的新提交移动已测 HEAD。

首次整合冻结 `faa1cd71191ab4e08057c7521b54af529b519b67` 在 Dimension5 的 stale code paths 检查失败：新增角色删除测试把隔离目录命名为 `.join("roles")`，违反既有 monorepo 路径规则。只将该临时夹具改名 `fixture-role-library`，不改检查器、不改生产路径；保留 `ci-local-first.log` / exit 1，定点检查后重新冻结重跑。不是以父提交的测试证明修正后 HEAD。

第二次冻结 `407a0942250abb910c6129df6438e1b41eb61aca` 的整合通过静态/前端/Clippy 后，在相同真实迁移恢复用例出现 **684 pass / 1 fail**：等待 worker 关闭期间，SQLite 已自行清掉 WAL，恢复删除随后返回 os error 2。修正仅让 WAL/SHM 的 NotFound 按“已清理”处理，取消观察后再删除的 TOCTOU；备份缺失、目录冒充文件、持续占用及其他错误仍拒绝。原 `ci-local-fixed.log` / exit 101 保留，继续要求真实 DB 原始行恢复，再冻结重跑。

幂等清理修正后，迁移模块 **20/0/0**、完整 Host 库 **685/0/0** 与 Host all-targets/all-features clippy 再次通过，分别保存在 `migration-sidecar-fixed`、`host-sidecar-final`、`clippy-sidecar-final` 日志。随后只冻结这份实际修正重新执行完整门禁，结果以本节指定的最终证据为准，不覆盖前两次整合日志。

语义自查覆盖 runtime/授权拒绝优先级、migration 主/次错误与 Windows 有界占用、镜像队列/DB 读取/取消/删除顺序。未改公开 DTO/trait、六槽、角色内容、SQL migration、锁文件、前端或 CI；这些保护项需与基线逐项 diff 证明。前端使用前轮相同源码的单测/浏览器证据，不宣称新的原生窗口、真实模型或新包验收。父债及冻结项保持，签名/密钥/外部角色身份隔离/根迁移协调仍属各自后续范围。
