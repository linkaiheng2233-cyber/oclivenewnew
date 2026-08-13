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
- 中文文件：禁管道传中文 / `-Encoding Ascii`；写盘只用 apply_patch 或 .NET UTF-8 无 BOM；写后自查（§7）
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

---

## 7. 中文编码红线（2026-08-13 · 4 起事故：K-EMO-01 词表乱码 / 台账 CJK 行 / M2 Plan / deepseek 蓝图锚点）

**根因**：Windows PowerShell 5.1 默认 `$OutputEncoding` = US-ASCII。中文经管道传给原生进程（`@'...'@ | python -` 等）时，每个非 ASCII 字符在到达目标前被替换为字面 `?`；`-Encoding Ascii` / .NET `Encoding.ASCII` 同理。结果文件仍是合法 UTF-8（`?` 是 ASCII），JSON 可解析、git diff 正常、现有门禁查不出 → 静默损坏。

**红线**（任何 AI 会话写含中文的文件）：

1. 禁止中文经管道传给解释器执行或写盘：`@'...'@ | python -`、`python -c "中文"`、`node -e "中文"`、`... | node -`。
2. 写中文文件只用两种方式：`apply_patch`；或 `[System.IO.File]::WriteAllText($path, $text, [System.Text.UTF8Encoding]::new($false))`（UTF-8 无 BOM）。禁止 `>` / `Out-File`（UTF-16 BOM）与 `-Encoding Ascii`。
3. Python/Node 读写显式 `encoding='utf-8'`；含中文的脚本先落盘 UTF-8 `.py` 再执行。

**写后必检**（含中文的文件写完必须自查，作为该提交的验收项）：

- 汉字数 > 0：`[\u4e00-\u9fff]` 匹配计数
- 连续问号串 = 0：`\?{3,}` 匹配计数
- 无 BOM：首 3 字节 ≠ `EF BB BF`
- JSON 文件须能正常解析

**已发生时**：先找干净源（git 历史提交 / 备份副本）恢复，再按本红线重写；不得把 `?` 当原文猜测回填。

---

## 8. 大文件拆分红线（2026-08-13 · 轮次 29：13 个 Rust/TS 巨文件拆分）

**背景**：本轮拆 `dto.rs` / `post.rs` / `co_present` / `kernel_attach` / `backend_registry` 等 Rust 模块与 `ModelManagerBody.vue` / `useVoiceAutoTts` / `useTheaterShell` 前端巨文件，实际发生三类事故：① prettier 与仓库 ESLint 风格冲突（一次 516 errors）；② TS 拆分后对导入的 `let` 绑定重新赋值（TS2632 批量报错）；③ 拆分脚本在错误 cwd 裸 `npx` 临时下载新版本工具。

**红线**（任何「拆大文件 / 拆组合式函数 / 模块化」任务）：

1. **前端验收以 ESLint 为准，禁止对仓库文件跑 `prettier --write`**（semi/quotes/operator-linebreak 与 prettier 默认相反）。手改后只用 `npx eslint --fix` 收尾，此后不再重跑 prettier。
2. **拆 TS/JS 模块前，先对每个待外移标识符 grep 全部使用点并区分「读 / 赋值 / 方法调用」**：ES 模块导入绑定只读——对导入的 `let` 直接赋值是 TS2632。只有「模块自包含状态 + 读写它的函数」可整体外移；共享可变状态留在声明处，或改由导出的 accessor 函数变更。
3. **公共 API 保持原路径**：拆分后原入口文件 `export { ... } from './new'` 再导出；动手前先 grep 所有 import sites，确认路径别名与目录解析仍可用。
4. **机械拆分（行号切片）允许，但必须**：切片前对每个边界行做 `startsWith` 断言；原文件被覆盖后用 `git show HEAD:<path>` 恢复干净源再重切；拆分后依次跑 eslint --fix → typecheck → 对应 workspace 单测（缺 export 会被 TS2459 批量暴露，属于级联而非新 bug）。
5. **每条 shell 命令开头 `Set-Location` 到仓库根**；跑工具用仓库 `npm run` 脚本或已安装本地 bin，禁止在错误 cwd 裸 `npx`（会按错误 package.json 临时安装新版工具）。
6. **PowerShell 原生进程 stderr 回显为 `NativeCommandError` 是假错误**（如 vitest 日志）；成败判定只看 `$LASTEXITCODE` 与工具自身的 PASS / `test result` 行。

**拆完必检**（每个文件拆完的提交级验收）：eslint 0 error · typecheck 0 · 对应 workspace `test:unit` 全绿 · 一次拆分 = 一个独立提交（提交后 worktree 干净）。
