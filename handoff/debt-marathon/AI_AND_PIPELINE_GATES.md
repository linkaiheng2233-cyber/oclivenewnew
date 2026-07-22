# AI 辅助限制 + OCLive 流水线硬门禁（马拉松强制）

**凡执行 `handoff/debt-marathon/` 内任意 Stage，必须先读本文，再读对应 long-plan。**  
违反任一条 → 本 Stage **FAIL**，不得继续下一债。

SSOT 链（禁止在本文双写长表）：

| 主题 | 路径 |
|------|------|
| 七阶段流水线 | `~/.cursor/skills/dev-pipeline/SKILL.md` |
| OCLive 定制 | `.cursor/skills/oclive-dev-pipeline/SKILL.md` · `discipline-checklist.md` |
| G1–G17 | `handoff/AI_CHANGE_BOUNDARIES.md` |
| 改代码索引 | `AGENTS.md` · `handoff/AI_READING_INDEX.md` §9 |
| 数字核实 | `handoff/AI_VERIFICATION_PROTOCOL.md` |
| 命名 | `creator-docs/NAMING_CONVENTIONS.md` §4.2 |
| 状态台账 | `handoff/TECHNICAL_DEBT_INVENTORY.md` |

---

## 1. 尺寸与阶段（流水线）

| 规则 | 要求 |
|------|------|
| 债 Done / main CI / 发版证据 | **强制尺寸 L** |
| M/L | 须有 Plan 或 **Ready 长计划书**；禁止无书乱改 |
| 七阶段 | ①对齐→③实现→④审查→⑤纪律→⑥文档→⑦总审；S 可合并，L 不跳⑤⑥⑦ |
| Stage 粒度 | **一次子会话 = 一债 × 一 Stage** |
| `todo completed` | **≠** TECHNICAL_DEBT Done |

完成状态只用：`Implemented` / `Locally verified` / `Done-eligible`（见 dev-pipeline）。

---

## 2. AI 改动边界（摘要 · 详读 BOUNDARIES）

| 禁 | 说明 |
|----|------|
| G1 | 角色任务不改 `slot_registry` / 官方 fixture |
| G3 | 不以 `handoff/archive`、旧 checklist 当 truth |
| G5 | 角色路径只用 SSOT 解析函数，禁瞎猜 `roles/` |
| G6 | 编排只进 `process_message` / `turn_pipeline/` |
| G7 | 回复字段 **`reply`**；错误码走生成链 |
| G11 | 无 RFC/决策 **不新建顶层 `.md`**；计划书只在 `debt-marathon/` |
| G14 | **链接代替复制**；不抄 MODULE_MAP / PLUGIN_V1 长表 |

Implementer 常量：`reply` · `plugin_backends` / `slot_registry.type` · 蓝图 **`steps[]` 不调度首轮**。

---

## 3. 工程硬约束（每次 Stage）

```
- 必读：本文件 + MARATHON_QUEUE + long-plans/<ID>.md（仅当前 Stage）
         + AGENTS.md + AI_CHANGE_BOUNDARIES + oclive-dev-pipeline
- grep 复用；NAMING_CONVENTIONS §4.2
- DTO→oclive_kernel_types；trait→oclive_kernel_contracts
- 禁生产 unwrap/expect；build_prompt 返回 String（非 Result）
- Cargo.lock → cargo audit + KNOWN_VULNERABILITIES 中英
- 新 AppError → generate-kernel-error-codes + drift + apiErrors + ERROR_CODES
- 六槽禁止第二套 parser；集成测用临时夹具
- 禁止把无关 dirty（语音/立绘等）混入债 PR
```

---

## 4. 验收（只跑 applicable）

Plan/Stage 勾选的命令必须有「因何 applicable」。欠债默认参考：

| 变更面 | 最小门禁 |
|--------|----------|
| 纯文档 | stale-paths docs · markdown links（若触 human-docs）· diff --check |
| 中英契约 | + check-doc-mirror |
| Rust | 定向测或 check:rust · layering（若分层） |
| 门禁脚本 | dimension5 --ci |
| Cargo.lock | audit + KNOWN |
| L Done | **远程 ci.yml 硬门禁 success** + TECHNICAL_DEBT Verification（SHA+run） |

禁止：无 CI 写 Done · 用 cancelled workflow 冒充绿 · 复制整张命令表全勾。

---

## 5. 马拉松专用

| 规则 | |
|------|--|
| runner | 只跑 `auto`；`human`/`skip` 不假装 Done |
| merge main | **默认禁止**（除非用户明文） |
| 超前台账 | 禁止（VOICE-04 教训） |
| Full vs Minimal | 只做本书声明档；另档另书 |
| 上下文 | 将满即停，Wave 写续跑坐标 |
| Cursor 长跑 | Cursor IDE 父 Agent 用 stop hook 续轮；一个子 Agent 只做一个 Stage；Cloud/CLI 不假定 lifecycle hook 可用 |
| 单写者 | 只有父 controller 更新 QUEUE / session；子 Agent 只返回结构化结果与代码 diff |
| 隔离 | 一次 claim 一个 Cursor worktree；禁止 `stash` / `switch` / `reset` / `clean` 共享工作树 |
| claim 范围 | 每次 claim 前 worktree 必须 clean；checkpoint 校验 base SHA 后全部 committed/uncommitted/untracked diff，禁止先 commit 绕过 Stage scope |
| 熔断 | 同一前置条件、权限或确定性失败不得无限重试；两轮无 checkpoint 自动停止 |
| 正常收口 | 最后一轮必须 terminal `done` checkpoint，且机器确认无 runnable auto；`pr-open` 只等待外部事件，不重复 claim |

---

## 6. Stage 出口检查清单（子 Agent + 父 controller）

子 Agent 返回前自检前三项；父 controller 校验返回、写 Wave 后完成后三项：

- [ ] 只动了本 Stage「文件范围」
- [ ] 已读本文 §2–§3
- [ ] applicable 验收命令 PASS/FAIL 已列
- [ ] 父 controller 未升错误 Done
- [ ] 父 controller 未合 main（除非授权）
- [ ] 父 controller 已更新 MARATHON_QUEUE、Wave 与 checkpoint
- [ ] Wave 已记录 claim、base/head SHA、changed files、最后命令、下一条精确命令与 retry_safe
