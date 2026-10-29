# 2026-10-30 · Chat Pro rc.3 安装器与内测包

**范围与授权**：用户要求清理旧包并准备新的安装器、内测包。基线为最新评审候选 `44cfa7f3ec0aa50a68adaa5414d5cfb7797df409`；开始时工作树干净。复用当前 checkout，新建本地 `release/chatpro-0.6.0-rc.3-20261030` 分支；不覆盖原 main 或既有主页工作树。尺寸 L（持久化修补与发包），controller 实施及语义自查，不派 Agent。

## 阶段与停止条件

1. 保全旧 rc.2 的交付记录，桌面目录与 ZIP 按明确路径移入带逐文件 SHA256 的归档；不清理角色、模型、聊天数据、编写器安装或视频。旧隔离安装只用于受控升级验证，完成后另归档。
2. 保留远端失败证据：最新候选的 Linux `reference-host-default` 在迁移恢复后的重新连接处返回 SQLite `database is locked`；原 run 为 `40857166402`、job 为 `123392117491`，680 passed / 1 failed。先复现与定点修补，保留主迁移错误、真实备份恢复断言、WAL/SHM 拒绝边界；不以睡眠或放宽断言代替因果核实。
3. 版本按现有 [RELEASE_VERSIONING](../../../creator-docs/development/RELEASE_VERSIONING.md) 升为桌面 `0.6.0-rc.3` / MSI `0.6.0.3`，CLI 与内核版本不联动。同步 manifest、锁文件、CHANGELOG 中英和现状 SSOT；锁变按供应链纪律审计。
4. 开发期只跑定向恢复/连接测试和适用 fmt/clippy/文档检查；范围稳定后分类提交并冻结目标，统一执行发行与集成门禁、模块兼容、供应链与浏览器回归。一次推送既有评审链，当前 SHA 的远端检查通过后再打包；正式 `ci-gate` 与草稿反馈分别记录，不擅自合并 main 或公开发行。
5. 构建前保存资源占位文件；构建后还原原始字节，核源码哈希及内核随包 manifest。产出 NSIS / MSI，执行隔离安装、旧 rc.2 升级、原生启动与健康探测、安装目录完整性及 MSI 解包检查。模型可用时进行隔离真实聊天，收费服务或缺失配置不默选。
6. 复用 rc.2 的内测包结构，提供 TXT / 离线 HTML / Markdown 说明、校验和修复工具；可选旧编写器如继续附带须明确来源与未重建。验证无模型权重、数据库、密钥或本地状态，ZIP 逐文件一致后交付桌面。最终只读总审并报告新基线、已测、未测和回退位置。

## 影响链与门禁

SQLite 连接/恢复 → Host builder → 原错误与恢复 marker → 桌面启动/帮助恢复；前端、六槽、角色包、插件协议无预期改动，仍核调用兼容。未增加数据库 migration，不修改冻结能力。

适用：恢复/连接行为测试、Host 库及 clippy、`check:release`（含 doctest）、`check:ci-local`、`check:module-compat`、cargo audit/deny、前端浏览器路径、默认与改文链接/镜像/登记/旧路径/编码。重复验证只因新差量或未解失败；日志记录首因及受检完整 SHA。真实模型、全新 Windows、签名与公开发行按实际证据分别报告，旧 rc.2 的验收不代证 rc.3。

机器证据位于 `E:/OCLive/oclive-dev-artifacts/chatpro-internal-rc3-20261030/`，旧包归档位于 `E:/OCLive/oclive-dev-artifacts/legacy-chatpro-archive-20261030/`。本计划不把未执行门禁或新包写成已完成。

## 定向核验与实施记录

- 桌面 rc.2 目录与 ZIP 已归档，44 个文件逐项核对。原隔离安装暂留供升级验证，不触碰用户实际数据或已有模型。
- Windows 原用例重复执行在第 30 次复现 WAL sharing violation。单独新备份启动用例未复现；调整最小连接数仍在第 21 次失败，该假设已撤回，连接池配置恢复原字节。这些探针不作为修复成功证据。
- 恢复操作和原 marker 写入统一移入 `spawn_blocking`，原主错误与恢复结果保持；builder 与同一 helper 的测试消费者同步等待。当前线程运行时因此可完成连接归还/句柄释放，不通过增加无限重试掩盖持续占用。新增 Windows 受控句柄由 Tokio task 释放的测试，旧同步等待会阻止该 task 运行。
- Windows 修正后迁移模块 21 测通过，原真实恢复用例独立重复 160 次通过。WSL Ubuntu 24.04 的原候选 200 次窄测未复现 GitHub Ubuntu 22.04 的偶发失败，保留这一差异；修正代码的 Linux 迁移模块 18 测与 200 次真实恢复重复通过。不是用 Windows 结果冒充 Linux、也不把窄测等同完整并发套件；最终仍核本轮目标 SHA 的远端 CI。
- 供应链在候选版本同步后的锁文件实跑：cargo audit 0 漏洞 / 10 报出警告（698 依赖、1386 公告）；npm production 0，full 为 4 low / 0 moderate / 0 high / 0 critical。未改 ignore、审计阈值或依赖锁定图。分类提交、最终全量门禁与安装验收继续按上述阶段执行。
