# Codex 视觉轨测试手册（Track V · 需要多模态）

**版本**：v2.1  
**前置**：Track A 已产出 `test-reports/codex-track-a-*.md`（建议）；至少执行过 **`npm run build:e2e`**。  
**适用模型**：GPT-4o、Claude 3.5+/4（vision）、Gemini 1.5+/2（vision）等 **能读 PNG/WebP**。  
**不适用**：DeepSeek 纯文本、无 vision 的模型 — **勿执行本文件**。

---

## 0. Agent 规则（视觉轨）

1. **只执行本文**；不重跑 Track A 的 clippy / 全量 `cargo test` / OOCP（除非 V 步骤写明）。
2. **终端退出码仍是 ground truth**；vision 只填 `visual_assessment`：`ok` | `regression_suspected` | `inconclusive` | `infra` | `test_bug` | `n/a`。
3. 报告 → `test-reports/codex-track-v-<YYYYMMDD-HHmm>.md`。
4. 若 Track A 有 `handoff_to_track_v`，必须做 **V-REV**。
5. 禁止改 `git config`；未要求不 `commit`。

---

## 1. 本轨 vs Track A 分工

| 类别 | Track A（文本） | Track V（视觉） |
|------|-----------------|-----------------|
| cargo / clippy / OOCP | ✅ | ❌ |
| Vitest 单元 | ✅ | ❌ |
| Playwright **跑测试** | ✅ 只记退出码 | ✅ 失败时 **读截图定性** |
| 桌面 GUI 冒烟 | ❌ | ✅ 可选 V-OPT |
| 插件 RPC | ✅ 文本 | ❌ |

---

## 2. 汇报格式

```text
[PASS|FAIL|SKIP] V-<步骤ID> <名称>
命令: <命令>
终端: exit_code=<n>
视觉: ok | regression_suspected | inconclusive | infra | test_bug | n/a
依据: <截图路径 + 1–3 句所见>
```

**V-F1 PASS**：`exit_code=0`。  
**V-F1 FAIL**：`exit_code≠0` 且 `visual_assessment=regression_suspected`。  
**合法降级**：`exit_code≠0` 且 `visual_assessment=infra|test_bug` → 标 **FAIL(环境/测试)**，合并报告注明「非产品阻断」。

---

## 3. 执行步骤

### V-pre — 构建与交接

| 步骤ID | 操作 | 预期 |
|--------|------|------|
| V-pre1 | 若无 `dist/` 或 A 未跑：`npm run build:e2e` | 退出码 0 |
| V-pre2 | 读取最新 `test-reports/codex-track-a-*.md` | 记录 handoff 项 |

---

### V-F1 — Playwright preview E2E（核心）

**覆盖 spec**（`playwright.config.ts` 默认忽略 `tauri-native`）：

- `e2e/preview-shell.spec.ts` — `#app` 挂载、标题
- `e2e/send-message.spec.ts` — 发送消息、空消息禁用
- `e2e/switch-role.spec.ts` — 切换角色
- `e2e/install-plugin.spec.ts` — 插件安装 UI

#### V-F1-run

**默认**：

```powershell
npm run build:e2e
npm run test:e2e:preview
```

**Windows 外置 preview**（内置 webServer 超时）：

终端 1：

```powershell
npm run preview -- --host 127.0.0.1 --port 4180 --strictPort
```

终端 2：

```powershell
$env:PW_TEST_USE_EXTERNAL = "1"
npm run test:e2e:preview
```

记录 **exit_code** → 步骤 **V-F1-run**。

#### V-F1-vis（exit_code≠0 或 handoff A-T1 时 **必做**）

1. 打开 `playwright-report/index.html` 或 `npx playwright show-report`。
2. 在 `test-results/` 下读取失败用例 **`.png`**；若有 `trace.zip` 仅描述首屏，不必解压上传。
3. 判断分类：

| 分类 | 含义 | 合并建议 |
|------|------|----------|
| `product_bug` | 白屏、路由错、文案/布局明显错误 | 阻断 Web 发版 |
| `test_bug` | 选择器、断言过严、mock 未生效 | 修测试 |
| `infra` | preview 未起、端口占用、超时 | 重试 / 换 CI |
| `inconclusive` | 截图不足 | 人工复核 |

步骤 **V-F1-vis**。

---

### V-F2 — Tauri 原生 E2E

```powershell
$env:OCLIVE_TAURI_E2E = "1"
npm run test:e2e:tauri-native
```

| 步骤ID | 说明 |
|--------|------|
| V-F2-run | 记录 exit_code |
| V-F2-vis | 失败则读 `test-results/`（同 V-F1） |

**SKIP**：无 Tauri 构建、无 GUI 环境 — 注明原因。handoff **A-T2** 时优先执行。

---

### V-REV — 复审 Track A handoff

当 Track A 报告勾选 `handoff_to_track_v`：

1. 针对 A-T1/A-T2：执行 V-F1-vis / V-F2-vis（可重跑 1 次以 fresh 截图）。
2. 输出根因表（`product_bug` / `test_bug` / `infra` / `unknown`）。
3. **不要**重复已通过的全量 A 轨命令。

步骤 ID：**V-REV**。

---

### V-OPT — 桌面壳冒烟（默认 SKIP）

仅当 `CODEX_VISION_DESKTOP_SMOKE=1`：

| 步骤ID | 内容 |
|--------|------|
| V-OPT0 | 启动 `npm run tauri:dev`（Agent 无法开 GUI → SKIP） |
| V-OPT1 | 截图：主界面 |
| V-OPT2 | 截图：插件管理（Ctrl+Shift+F） |
| V-OPT3 | 截图：设置 → 存储管理（backend 标签与能力门控） |

检查：无白屏、无严重错位、存储面板 `supports_*` 与 backend 一致。

---

## 4. Track V 报告模板

`test-reports/codex-track-v-<时间>.md`：

```markdown
# Track V 测试报告（视觉轨）

- 模型: <GPT-4o|Claude|Gemini|…>
- 平台: ...
- Track A 引用: test-reports/codex-track-a-*.md

## 汇总
| 步骤 | exit_code | visual_assessment |
| V-F1-run | | |
| V-F1-vis | n/a | |
| V-F2 | | |
| V-REV | | |
| V-OPT | SKIP/… | |

## 截图路径
- test-results/...

## UI 发版建议
- [ ] Web preview 可发版
- [ ] 需修产品后重测
- [ ] 仅信 Ubuntu CI（本地 infra）

## 合并
见 CODEX_TEST_RUNBOOK.md §5
```

---

## 5. Track V 一键提示（多模态 Agent）

```text
执行 oclivenewnew/dev-notes/codex-testing/CODEX_TEST_RUNBOOK_VISION.md（Track V only）。
不要重跑 Track A 的 cargo/clippy/OOCP/Vitest。
先 V-pre（npm run build:e2e），再 V-F1-run；若 exit≠0 或 Track A handoff A-T1，必须 V-F1-vis 读 playwright-report 与 test-results PNG。
有 handoff 则 V-REV。V-OPT 仅当 CODEX_VISION_DESKTOP_SMOKE=1。
写入 test-reports/codex-track-v-<时间>.md。不要 git commit。
```

---

## 6. 合法 SKIP

| 条件 | 步骤 |
|------|------|
| 模型无 vision | 全文 — 换模型 |
| win32 preview 反复超时且 V-F1-vis=infra | V-F1 — 建议 Ubuntu CI 仲裁 |
| 无法启动 Tauri GUI | V-OPT |
| A-T 全 PASS 且 V-F1 exit 0 | V-REV — 「无需复审」 |

---

## 7. CI 对齐

| 环境 | 权威 |
|------|------|
| GitHub `frontend` job（Ubuntu） | Playwright 守门 |
| 本地 Track V | 辅助；与 A 轨合并判断 |

参考：`CONTRIBUTING.md`、`playwright.config.ts`、`e2e/preview-shell.spec.ts`
