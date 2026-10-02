# K-ENCODING-01：活跃文档编码防回退

**范围**：基线 `d7040c37cc95966764318a3f8aa6481c9a831b1e`，开场工作树干净；按[第二轮计划](../ROUND-02-PLAN.md#k-encoding-01--活跃文档编码防回退2026-10-02)仅增加活跃 Markdown 的只读检查与 Dimension 5 接线。当前状态只见[主台账](../../TECHNICAL_DEBT_INVENTORY.md)。归档原件、JSON、产品源码、角色包与外部目录不在本轮写集。

## 实现与影响链

`scripts/check-doc-encoding.mjs` 默认递归扫描 `handoff/`、`creator-docs/`、`human-docs/`，跳过各级 `archive/`，拒绝活动目录中的符号链接；使用 fatal UTF-8 解码，分别报告 BOM、无效编码、U+FFFD 与连续三问号。`--file` 接受显式文件，供精确写后复核与测试。检查器只读取字节，不自动修复。`scripts/check-doc-encoding.test.mjs` 以真实临时文件调用同一 CLI，覆盖正常中英文、四项拒绝和归档排除；Dimension 5 将负控与当前活跃文档检查合为一个硬步骤。

开工前扫描基线为 307 份活跃 Markdown；新增本 Wave 后为 308 份。没有向英文文档强加汉字数量要求。新检查不能识别单个或两个问号造成的有损替换，亦不能判断语义或替代中文写后逐文件核对。因此 K-ENCODING-01 仍 OPEN，长期执行观察继续。

## 验证与失败归因

- `node --test scripts/check-doc-encoding.test.mjs`：6/6 通过；`node scripts/check-doc-encoding.mjs`：最终 308 份通过。
- `node scripts/dimension5-acceptance.mjs --ci` 首次运行：新步骤通过，但语音 ratchet 因本机缺 Windows `py` 启动器而失败；已有 portable `python.cmd` 不适于该脚本的 `spawnSync` 可执行路径。仅在命令子进程设置 `OCLIVE_VOICE_PYTHON` 指向已安装 Python 3.12 真正的 `python.exe` 后，第二次运行 30/30 通过；没有改语音脚本或全局环境。
- `npm run check:ci-local`：exit 0；Dimension 5、共享前端静态检查与构建、Rust 分层/格式/Clippy/lib/workspace/CLI 集成测试均完成。该命令同样仅在子进程临时指定上述 Python 可执行文件，退出后环境恢复。
- 改文链接 5/5、文档旧路径、文档注册表及债务结构检查通过；目标提交的远端 CI 待提交推送后核对，不能以父提交 CI 代替。

## 纪律与剩余

G1–G8、G10–G11 与产品/契约边界均未触及；G9 复用 Dimension 5 的现有步骤与 Node 测试结构；G12–G16 仅同步现有状态 owner、计划、门禁说明和变动事件；G17 无生产者/消费者运行链，影响链限定为文档字节 → 检查器 → Dimension 5 → CI。旧历史原件不改，错误文件须从可信源恢复，不猜测受损文字。本轮不关闭父债，不扩成全仓所有文本或 JSON 的隐式检查。
