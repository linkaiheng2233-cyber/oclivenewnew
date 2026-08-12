---
name: oclive-dev-pipeline
description: >-
  OCLive/oclivenewnew project layer for the seven-phase pipeline: G1–G17, SSOT
  routing, S/M/L sizing, check scripts, remote CI Done evidence. Use for
  oclivenewnew, A.I.Live, kernel/, distros/, pack-editor, vscode. Triggers:
  按 oclive 开发流水线, 按 oclive 流程, oclive dev pipeline, 技术债收口, 小改跳过 Plan.
---

# OCLive Dev Pipeline（项目定制层）

**通用框架**：`~/.cursor/skills/dev-pipeline/`（含 `task-sizing.md` 尺寸分流）  
**本文件**：OCLive 门禁 · SSOT · 验收 · 开场白  
**不重复**通用七阶段正文。

> `.cursor/skills/` 常被仓库 gitignore：**本 Skill 可能只在本机**。共享请 un-ignore 或把关键门禁写进已跟踪 `AGENTS.md` / handoff。不要擅自改 `.gitignore`。

## 用户开场白（复制即用）

**L · 债收口 / CI 恢复**
```
按 oclive 开发流水线走。尺寸 L。
任务：……
验收：npm run check:ci-local PASS；push 后远程 ci.yml 对目标提交 success；台账含 HEAD SHA。
```

**M · 标准功能**
```
按 oclive 开发流水线走。尺寸 M。
任务：……
验收：npm run check:rust（或更窄测）+ applicable 纪律脚本。
```

**S · 小改**
```
小改跳过 Plan。尺寸 S。
任务：只改……（≤3 文件，无契约/编排）。
验收：……（窄测命令）。
```

## 启动

1. 加载通用 **dev-pipeline** + 本文件 + [discipline-checklist.md](discipline-checklist.md)。
2. 定 **S/M/L**（通用 task-sizing）；触及 TECHNICAL_DEBT Done / main CI / 发版 → **强制 L**。
3. 所有尺寸先读精简 [`AGENTS.md`](../../../AGENTS.md)；M/L 再按 [`AI_READING_INDEX`](../../../handoff/AI_READING_INDEX.md) §9 选择场景路径，禁止无差别全读。
4. 姊妹仓：读该仓 `AGENTS.md`，流程仍回本仓本 Skill。

## 项目规则（全程）

- Rules：`.cursor/rules/oclivenewnew.mdc`
- G1–G17：[`AI_CHANGE_BOUNDARIES.md`](../../../handoff/AI_CHANGE_BOUNDARIES.md)
- 模块：[`MODULE_MAP_AND_HANDOFF.md`](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
- 关键路径：[`BUS_FACTOR_NOTES.md`](../../../handoff/BUS_FACTOR_NOTES.md)
- 数字核实：[`AI_VERIFICATION_PROTOCOL.md`](../../../handoff/AI_VERIFICATION_PROTOCOL.md)

**常量**：回复字段 **`reply`** · 六槽 `plugin_backends` / `slot_registry.type` · 蓝图 **`steps[]` 不调度首轮**。
- **中文编码红线**（写中文文件前必读）：[AI_AND_PIPELINE_GATES §7](../../../handoff/debt-marathon/AI_AND_PIPELINE_GATES.md) —— 禁管道传中文 / `-Encoding Ascii`；只走 apply_patch 或 .NET UTF-8 无 BOM；写后自查汉字数>0 且无 `\?{3,}`。

## 场景路由（① 必做）

见 [`AI_READING_INDEX` §9](../../../handoff/AI_READING_INDEX.md#9-按任务选阅读路径)。技术债收口 → 本 Skill + TECHNICAL_DEBT + AI_VERIFICATION_PROTOCOL。

## OCLive 阶段增量

### ① Plan
- [plan-template.md](plan-template.md) + 通用模板；必填 G 约束 · 场景路径 · 尺寸 · 验收勾选。
- 用户已明确要求“实现/修复/优化”即为当前范围授权；只有产品取舍、外部写入或范围扩张才再次等待确认。
- todo `completed` ≠ Done。
- Plan 必填 **CI 节奏**：开发切片窄测、预计冻结点、里程碑全量门禁与是否需要远端 CI；不确定工作量时先按 M，触及契约/编排/权限/迁移/Done 再升级。

### ③ Implementer 硬约束（写入 dispatch）

```
- grep 复用；NAMING_CONVENTIONS §4.2
- 编排仅 process_message / turn_pipeline/；Tauri api/*.rs 薄封装
- DTO→oclive_kernel_types；trait→oclive_kernel_contracts
- 禁生产 unwrap/expect；build_prompt 返回 String
- 角色路径 chat_pro_roles_dir() / resolve_project_roles_dir()
- Cargo.lock → cargo audit + KNOWN_VULNERABILITIES（中英）
- 新 AppError → generate-kernel-error-codes + check-error-codes-drift + apiErrors + ERROR_CODES
- 六槽禁止第二套 parser；集成测用临时夹具，不改官方 slot_registry
- G17：先列生产者→契约→适配/权限→消费者→状态/回退→测试；逐项核对 kernel/Tauri/shared/发行版/插件/角色包/姊妹仓
```

### ④
- 默认 Bugbot；不可用时人工语义 diff review，仍输出 findings。permissions / network / process:spawn / MCP → Security 专项。
- 公开 DTO/trait → ⑤ 含 doctest。

### ⑤
1. 通用 discipline-review + **全部** [discipline-checklist.md](discipline-checklist.md) applicable 项  
2. 按 diff 跑脚本（见下表），**不得跳过 applicable**  
3. G FAIL 阻塞进⑥；Partial→Done 见 checklist「Done 证据」

**按变更面选门禁，不按焦虑程度全跑：**

| 变更面 | 最小验收 |
|--------|----------|
| 纯文档（非中英契约镜像） | `check-markdown-links`（若在默认范围）· `check-stale-paths --docs-only` · `check-doc-registry`（顶层/SSOT）· `git diff --check` |
| creator-docs 中英契约 | 上述 + `check-doc-mirror` |
| Rust 内核/分层 | 定向测试或 `check:rust` + `check-domain-layering` |
| 公开 DTO/trait/re-export | Rust 验收 + `cargo test --workspace --doc`；错误码另跑 drift |
| Chat Pro / 目录插件 / 插槽 | `npm run check:module-compat` + Vue/iframe/Bridge/RPC 受影响行为测 |
| 门禁脚本/CI | 脚本自测 + `dimension5 --ci`；影响集成链时再跑 `check:ci-local` |
| Cargo.lock/供应链 | `cargo audit` + KNOWN_VULNERABILITIES 中英 + applicable deny |
| `oclive-cli` crate 全测 | `cargo test --locked -p oclive-cli -- --test-threads=1`；其 E2E 会嵌套 Cargo，禁止默认并行争抢 package cache |
| L：发版/main 恢复/债 Done | 项目要求的全量本地门禁 + **目标提交远程 CI** |

**CI 推送节奏（硬规则）：**

1. 开发切片只跑受影响窄测并本地提交，不用远端 CI 发现本地可发现的问题。
2. 多个关联切片完成后，在计划标记的里程碑统一跑全量本地门禁；通过后冻结 HEAD、一次推送。
3. 未冻结但需要远端备份/协作时，优先推送无 ready PR 分支；若已有 PR，只推逻辑完整切片，不为每个小提交等待全量矩阵。
4. 远端失败先读失败 job 并定点修复；禁止无代码变化反复 rerun 掩盖确定性失败。
5. 远端已绿后不追加“只回写 CI run ID/状态”的证据提交；先记 PR 评论/交付报告，随下一次实质提交入账。
6. 发版、main、L 结案和技术债 Done 的最终冻结 SHA 仍必须远端 CI 成功，不得以节奏优化为由跳过。

### ⑥
- handoff/README §文档分责；G11；MODULE_MAP / PLUGIN_V1 不互拷  
- `check-doc-registry` · `check-markdown-links`（**默认仅** `human-docs/modules`，不扩到全历史文档）· `check-stale-paths` · `check-doc-mirror`

### ⑦ Ask
对照：六槽/编排泄漏/记忆三套/错误码 SSOT/第二套解析/冻结项/姊妹仓/台账+远程 CI。Ask 不可用时在当前会话停止写入并只读总审。

**L 结案状态**：未获 push/外部写权限时可交付 **Locally verified**，但 TECHNICAL_DEBT 保持 Partial/OPEN；只有冻结目标提交的远程 CI success 才是 **Done-eligible**。绿灯后证据先记 PR 评论/交付报告，不为回写编号制造第二个未验证 HEAD。

## 验收命令（Plan 只勾 applicable）

| 命令 | 何时 |
|------|------|
| `npm run check:rust` | Rust workspace / 分层；不作为纯前端 TS 的替代 |
| `npm run check:ci-local` | 集成链 / 债收口 / 会影响集成行为的门禁改动 |
| `npm run check:release` | 发版级 |
| `cargo test --workspace --doc` | 公开 API |
| `node scripts/dimension5-acceptance.mjs --ci` | 项数以脚本 `PASS (N checks)` 为准 |
| `node scripts/check-error-codes-drift.mjs` | error / apiErrors / ERROR_CODES |
| `node scripts/check-domain-layering.mjs` | kernel 分层 |
| `node scripts/check-stale-paths.mjs` | 路径引用 |
| `node scripts/check-markdown-links.mjs` | **仅** `human-docs/modules` 默认范围 |
| `node scripts/check-doc-mirror.mjs` | 中英文 |
| `npm run test:unit` | shared / chat-pro |
| `npm run check:module-compat` | Chat Pro / 目录插件 / 插槽注册表 / Vue 与 iframe 入口 |
| Tauri `--test …` | 定向集成 |
| **远程 `ci.yml` success** | L：有 push 授权后 · 台账 Done · main 恢复；无授权则报告 Locally verified |

**链接门禁边界**：默认不扫全仓历史 md，避免无边界清理；扩范围须用户显式授权。

**命令纪律**：Plan 中每条命令都写“因何 applicable”；禁止复制整张命令表后全部勾选，也禁止用一个大命令掩盖缺少的专项检查。

**invoke 条数**：以 [`INVOKE_HOTPATH_MATRIX.md`](../../../handoff/INVOKE_HOTPATH_MATRIX.md) 为准，勿在 Skill 写死数字。

## 禁止区速查（⑤）

| G | 速查 |
|---|------|
| G1 | 角色任务无 `slot_registry` / `plugin_backends` / `runtime_config` diff |
| G3 | 无 archive / `04_4.6` 当 truth |
| G5 | `roles/` · `src-tauri` · `join("roles")` |
| G6 | 编排仅 `process_message` / `turn_pipeline/` |
| G7 | `reply` · 错误码 SSOT（checklist G7b） |
| G11 | 无擅自新顶层 `.md` |
| G17 | 无只改生产者或消费者；汇报列已改、无需改、回退和跨边界测试 |

## 技术债马拉松（长计划书 · 分阶段子 Agent）

多 Stage / 长时间债偿还：另载 [`oclive-debt-marathon`](../oclive-debt-marathon/SKILL.md)。  
计划书与波次日志 Git SSOT：[`handoff/debt-marathon/`](../../../handoff/debt-marathon/README.md)。

触发语另见该 Skill（「债偿还马拉松」「长流程计划书」等）。普通短债收口仍用本文件即可。

## 触发语

「按 oclive 开发流水线走」「按 oclive 流程」「oclive dev pipeline」「技术债收口」
