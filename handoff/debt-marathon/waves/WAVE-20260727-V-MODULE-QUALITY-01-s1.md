# WAVE-20260727-V-MODULE-QUALITY-01-s1

> 计划书：[`../long-plans/V-MODULE-QUALITY-01.md`](../long-plans/V-MODULE-QUALITY-01.md) · 本 Wave 只证明 Stage 1，不关闭父债

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | V-MODULE-QUALITY-01 |
| **Stage** | 1 · Versioned fixture contract and deterministic offline scorer |
| **分支** | `codex/k-plugin-sec-01` |
| **日期** | 2026-07-27 |
| **Claim** | Codex 手工受控执行；Cursor 历史 session 为 inactive，未启动或伪造 Cursor claim |
| **Base** | `045ab3b25173dacdafb74cd1b628f1d840521f60` |
| **Head** | `675561d5c1a9f3f461fe9e0012667041cb4c4ccd` |
| **实现提交** | `675561d5 test(modules): add deterministic behavior quality scorer` |
| **状态三态** | **Locally verified** · 未推送、无目标提交远程 CI，父债保持 OPEN |

## Changed

- `scripts/module-quality-harness.mjs`
- `examples/module-quality-harness/fixtures/suite.v1.json`
- `examples/module-quality-harness/fixtures/observations.reference.v1.json`

没有修改内核、发行版、角色包解析、聊天编排、replay、性能 bench 或公开响应 DTO。

## 合同与行为

- Suite 固定 `role_id + scene_id + replay`，并分别声明 memory / emotion / prompt / LLM 的可判定期望。
- Observation 严格要求四类模块身份、版本与逐用例观察；缺字段、额外字段、重复/缺失 case 均 fail closed。
- 报告使用规范化 JSON 的 SHA-256 绑定 suite 与原始 observation；不复制原始 prompt / reply 到报告。
- 汇总保留四维独立分数，不制造一个“客观总质量分”。
- 用户复述检测使用 Unicode 字母/数字三元组重叠率，不依赖中文或英文标点切分。
- 固定样例对应仓库现有 `mumu/home`、`phoebe-chubi/default`、`doro/default`，情绪标签对齐内核现有枚举。

## 验收证据

| 检查 | 结果 |
|------|------|
| `node --check scripts/module-quality-harness.mjs` | **PASS** |
| `node scripts/module-quality-harness.mjs --self-test` | **PASS** · 同输入输出一致；畸形 observation 被拒绝；四维回归均失败 |
| reference `--json` 连续运行两次逐字比较 | **PASS** · deterministic |
| reference report | **PASS** · 3/3 cases；memory 7/7、emotion 3/3、prompt 9/9、LLM 11/11 |
| suite digest | `6fb61a37b1fa19e772350fe174d8d075de558f125ab65106e2b1181e5fe7e900` |
| observation digest | `c9f2191ab163c14cacc3dc40274bdaf129ee8ce47d5c2b79c6acb4ecbe84d52d` |
| `npx prettier --check ...` | **PASS** |
| `node scripts/check-debt-marathon.mjs --id V-MODULE-QUALITY-01 --stage 1 --require-ready` | **PASS** |
| `npm run check:module-compat` | **PASS** · 10 slots、9 manifests、7 UI contributions、6 plugins |
| `git diff --check` | **PASS** |

全 `handoff/` Markdown 扫描仍有 41 条既有断链基线；本轮四个计划/台账文件的定向扫描为 PASS，未将无关文档债混入本 Stage。

## 刻意没做

- 未连接真实或 mock 内核输出；这是 Stage 2。
- 未把质量字段写入 `oclive bench`，避免破坏现有性能报告语义。
- 未注册发布级 CI gate；必须等真实采集与跨模块比较成立。
- 未写 Done，未推送，未合 main。

## GATES §6

- [x] 实现只修改 Stage 1 声明的三个文件；Wave / QUEUE / 台账由父 controller 收口
- [x] 已读 GATES §2–§3、AI 开发辅助限制与目标计划
- [x] applicable 验收命令及 PASS 结果已列
- [x] 父 controller 未错误升级 Done
- [x] 父 controller 未合 main
- [x] 父 controller 已更新 MARATHON_QUEUE、Wave 与计划/台账；Cursor session inactive，使用 Wave + Git SHA 作为 Codex 手工 checkpoint
- [x] Wave 已记录 claim 状态、base/head SHA、changed files、最后命令、下一条精确命令与 retry_safe

## 续跑坐标

- **最后命令：** `npm run check:module-compat`
- **下一条精确命令：** `node scripts/check-debt-marathon.mjs --id V-MODULE-QUALITY-01 --stage 2 --require-ready`
- **下一 Stage：** 2 · Existing-kernel observation adapter
- **retry_safe：** yes；Stage 1 已独立提交，Stage 2 失败时保留离线合同与评分器即可
