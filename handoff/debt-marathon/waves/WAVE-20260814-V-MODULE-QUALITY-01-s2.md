# WAVE-20260814-V-MODULE-QUALITY-01-s2

> 计划书：[`../long-plans/V-MODULE-QUALITY-01.md`](../long-plans/V-MODULE-QUALITY-01.md) · 本 Wave 只证明 Stage 2，不关闭父债

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | V-MODULE-QUALITY-01 |
| **Stage** | 2 · Existing-kernel observation adapter |
| **分支** | `closeout/continuity-module-quality` |
| **日期** | 2026-08-14 |
| **Claim** | Codex 手工受控执行；未伪造 Cursor claim |
| **Base** | `0e6254d86fa6bc2646c8cc873966c6a8aead4ed1` |
| **实现 Head** | `c883ceae` |
| **实现提交** | `c883ceae test(modules): capture kernel quality observations` |
| **状态三态** | **Locally verified** · 尚无目标提交远程 CI，父债保持 OPEN |

## Changed

- `scripts/module-quality-runner.mjs`
- `scripts/lib/module-quality/capture.mjs`
- `scripts/lib/module-quality/contracts.mjs`
- `scripts/lib/module-quality/fixture-roles.mjs`
- `scripts/lib/module-quality/kernel-client.mjs`
- `scripts/lib/module-quality/observation-sidecar.mjs`

没有修改生产响应 DTO、`process_message` 编排、角色包源文件、评分 fixture 或性能 bench。运行器先复制 fixture 涉及的角色包到临时目录，再只改临时副本的四槽 backend。

## 合同与隐私边界

- 通过既有 `/chat/storage` 导入固定 replay 历史，通过 `/chat` 执行最后一个用户回合，通过 `/llm/user_settings` 切换到本地 `oclive_jsonrpc` sidecar。
- 逐例强制观察 `memory.rank`、`emotion.analyze`、`prompt.build_prompt` 与 `llm.generate[_stream]`，缺任一链路即失败。
- sidecar 只收集 suite 注入的 `mq-*` fixture memory；非 fixture memory 不进入安全 Prompt 或报告，自测显式验证此边界。
- 临时令牌、数据库、角色副本与内核进程树都限定在单次运行生命周期，结束后清理。
- 采集器按入口、合同、fixture、sidecar、内核客户端和 orchestration 拆分；单文件 57–225 行，没有新增上帝文件。

## 验收证据

| 检查 | 结果 |
|------|------|
| `node scripts/module-quality-runner.mjs --self-test` | **PASS** · fixture memory 保留、私密 memory 剔除、用户输入不回显 |
| real HTTP/remote-slot capture | **PASS** · 3/3 cases；memory 7/7、emotion 3/3、prompt 9/9、LLM 11/11 |
| suite digest | `6fb61a37b1fa19e772350fe174d8d075de558f125ab65106e2b1181e5fe7e900` |
| observation digest | `d1296ac84d7ae71d90b99310d7a6cad9c7934a730911702593e2b530bc223996` |
| `node examples/oocp-test-suite/run.mjs --required-only` | **PASS** · S0/S0b/S1–S12/S15/S16 |
| `cargo test -p oclive_kernel_host --lib -j 1` | **PASS** · 467/467 |
| `cargo test --workspace --doc -j 1` | **PASS** · 6 doctests |
| `node scripts/check-debt-marathon.mjs --id V-MODULE-QUALITY-01 --stage 2 --require-ready` | **PASS** |
| `git diff --check` | **PASS** |

## 刻意没做

- 没有把完整生产 Prompt、私密记忆或 API token 写进制品。
- 没有用 mock 直接伪造 `/chat` 结果；固定 sidecar 是实际 remote 模块 provider，聊天与 replay 仍走内核公开链路。
- 没有把行为分数与延迟混成总分。
- 没有关闭父债：Stage 3 仍需至少两套明确模块配置对比、双语创作者文档和发布入口；Stage 4 仍需精确提交远程 CI。

## 续跑坐标

- **最后命令：** `cargo test --workspace --doc -j 1`
- **下一条精确命令：** `node scripts/check-debt-marathon.mjs --id V-MODULE-QUALITY-01 --stage 3 --require-ready`
- **下一 Stage：** 3 · Cross-module comparison report and creator documentation
- **retry_safe：** yes；Stage 1 离线评分和 Stage 2 采集提交均独立，Stage 3 可单独回滚
