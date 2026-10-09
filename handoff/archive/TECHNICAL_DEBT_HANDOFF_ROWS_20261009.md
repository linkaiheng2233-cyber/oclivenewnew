# 技术债接手待验原行快照 · 2026-10-09

**状态**：历史原文；基线 `566a8a69b2433b5e13e1ac6b9f1a1274d4e7843f` 的两条权威行。当前状态只读 [技术债台账](../TECHNICAL_DEBT_INVENTORY.md)，修正依据见 [DCL-82](../debt-marathon/DEBT_CHANGELOG.md#dcl-20261009-82--模块实现者里程碑与两条过期待验对账)。

下面 fenced block 的原行文本逐字保留；相对链接按原台账目录解释。换行统一为 LF，不改变行文字。本快照不将历史待验描述当当前状态，也不是新的验收来源。
## K-LLM-ENV-02

```text
| **K-LLM-ENV-02** | `apply_user_llm_env` 在 DB snapshot 读取后才取得进程环境锁，并把调用结束时的最新版本直接标记为已应用；并发旧调用可能覆盖新环境却清除 dirty 标记 | **P1** | 串行化完整的“读设置 → token/cache → env/provider → 版本提交”事务，或改为不可变配置快照；版本变化时必须重试而非误报已应用；用可控交错测试证明 last-writer-wins，覆盖保存设置、chat、theater 与 canonical sync 调用链 | **Done · 原并发缺陷的有限验收**（单一异步锁覆盖 DB 读取至环境写入，实际读取版本被提交，旧读取不能在新设置后覆盖环境又误清 dirty；[可控交错与主路径证据](debt-marathon/waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--原缺陷的有限收口)覆盖保存设置、chat、Theater、桌面 canonical seed 及临时文件库重建。目标 `86cbb2d5` 的[正式 CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37100060646) 17/17 success，本地综合链 exit 0。真实 provider、跨进程环境、全部失败交错与长时压力不在本次有限结案；新反例另登记。历史完整状态见[冻结快照](archive/TECHNICAL_DEBT_LLM_ENV_STATUS_20261003.md)）  2026-10-09 原环境事务结论保持；Windows CI 暴露独立 `slot_resolution_chain` 夹具隔离缺口，本片保护四个初始化/读取用例并恢复原 backend 值，原断言不放宽。当前仅修补待验，不把旧 SHA 的绿灯当本片验收，见 [DCL-69](debt-marathon/DEBT_CHANGELOG.md#dcl-20261009-69--slot-resolution-测试进程环境保护补齐)。 |
```

## K-SUPPLY-12

```text
| **K-SUPPLY-12** | npm 开发工具链 audit 命中与 ESLint peer 契约漂移 | **P1** | 对 ESLint/`brace-expansion`、WebDriver/`fast-xml-parser`、旧 Vue/PostCSS SFC loader 逐条做可达性与升级/移除；`npm ls eslint eslint-plugin-unicorn` 退出 0，完整/生产 `npm audit` 无 high，lint/typecheck/unit/build 与 Linux/Windows CI 全绿；禁止 `--force` 或无证据 override | **Partial · 上游安全链已采用，临时例外撤销片局部通过**（2026-10-09：官方 concurrently 9.2.5 精确采用 shell-quote 1.12.0；仍用原 ^9.2.1 声明，锁只改父/叶两个节点，移除 overrides.concurrently，其余四条例外保持。原 npm ci / 全树、生产展开、良性子进程成功与失败传播和 production build 均 native 0；前后 production 0、full 4 low／0 moderate／0 high／0 critical，KaTeX low 仍 OPEN。完整本地与本次新 SHA 正式 CI 待冻结后执行，不借旧基线绿灯结案。当前撤销范围见 [DCL-73](debt-marathon/DEBT_CHANGELOG.md#dcl-20261009-73--上游安全补丁与临时-override-撤销)；旧批准与失败保持在 [DCL-53](debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-53--shell-quote-临时兼容补丁与撤销条件)和[安全滚动记录](../creator-docs/security/KNOWN_VULNERABILITIES.md#2026-10-07-shell-quote-临时兼容例外)，SDK 根声明引用化的 [DCL-64](debt-marathon/DEBT_CHANGELOG.md#dcl-20261008-64--tauri-sdk-override-的根声明引用化)保持，不升级/解除其它例外） |
```
