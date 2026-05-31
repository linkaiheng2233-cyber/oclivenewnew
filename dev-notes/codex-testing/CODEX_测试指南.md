# oclivenewnew · Codex 测试指南（转发用）

**仓库**：`oclivenewnew` 根目录  
**详细手册**：`CODEX_TEST_RUNBOOK.md`（文本轨）、`CODEX_TEST_RUNBOOK_VISION.md`（视觉轨）

---

## 一、你要做什么

在仓库根目录**自动执行回归测试**，按步骤跑命令，**不要改业务代码**，**不要 git commit**。

测试分 **两条轨**，按模型能力选一条或两条都跑：

| 轨道 | 适用模型 | 做什么 |
|------|----------|--------|
| **Track A · 文本轨** | DeepSeek、Codex、无 vision 的 Agent | 终端命令、cargo/npm 测试、OOCP HTTP、E2E **只看退出码** |
| **Track V · 视觉轨** | GPT-4o、Claude vision、Gemini vision | Playwright **失败时读截图** 判断 UI/环境/产品问题 |

**推荐顺序**：先 **Track A** → 再 **Track V**（V 可读 A 的报告）。

---

## 二、共同规则

1. 工作目录：仓库根；无 `node_modules` 先 `npm install`。
2. HTTP/OOCP 测试必须：`OCLIVE_HTTP_API_MOCK_LLM=1`（不依赖 Ollama）。
3. 禁止：修改 `git config`、force push、未要求时 `commit`。
4. Track A 的 P0 任一项失败：继续跑完 A 内全部 P0，最终标记 `P0_BLOCKED=是`。
5. 报告写入 `test-reports/`（无则创建）：
   - A → `test-reports/codex-track-a-<YYYYMMDD-HHmm>.md`
   - V → `test-reports/codex-track-v-<YYYYMMDD-HHmm>.md`
   - 两轨都完成后 → `test-reports/codex-merged-<YYYYMMDD-HHmm>.md`

---

## 三、每项汇报格式

```text
[PASS|FAIL|SKIP] <轨道>-<步骤ID> <名称>
命令: <完整命令>
摘要: ...
失败: <最多 30 行日志>
```

Track V 额外字段：

```text
终端: exit_code=<n>
视觉: ok | regression_suspected | inconclusive | infra | test_bug | n/a
依据: <截图路径 + 简要描述>
```

---

## 四、Track A（文本轨）— 完整步骤

> **不要**打开或分析任何图片/截图。

### E0 环境

```powershell
git rev-parse --show-toplevel
node -v
cargo -V
npm install   # 若无 node_modules
```

### 阶段 A — P0 全局门禁（逐项执行并汇报）

```powershell
npm run build
npm run check:rust:fmt
npm run check:rust:clippy
npm run check:rust:test
npm run check:rust:test:all
npm run test:unit
cargo test --workspace
```

步骤 ID：`A-G1` … `A-G7`

### 阶段 B — P0 插件多槽

```powershell
cargo test -p oclivenewnew-tauri --test plugin_backends_v2_resolve
cargo test -p oclivenewnew-tauri --test slot_runner_p4
cargo test -p oclivenewnew-tauri --test blueprint_v2_role_load
cargo test -p oclivenewnew-tauri --test blueprint_v2_mumu_load
cargo test -p oclivenewnew-tauri --test save_role_slot_registry
cargo test -p oclivenewnew-tauri --test slot_resolver_v3
cargo test -p oclivenewnew-tauri --test permission_three_way_consistency
```

步骤 ID：`B-M1` … `B-M7`

### 阶段 C — P0 对话编排

```powershell
cargo test -p oclivenewnew-tauri --test process_message_golden_path
cargo test -p oclivenewnew-tauri --test narrative_hint_prompt_roundtrip
cargo test -p oclivenewnew-tauri --test narrative_hint_contract_audit
cargo test -p oclivenewnew-tauri --test complex_emotion_hint_persistence
cargo test -p oclivenewnew-tauri --test chat_integration
cargo test -p oclivenewnew-tauri --test http_api_chat
cargo test -p oclivenewnew-tauri --test week3_004_api
cargo test -p oclivenewnew-tauri --test invoke_hotpath_matrix
cargo test -p oclivenewnew-tauri --test policy_e2e_matrix
```

步骤 ID：`C-C1` … `C-C9`

### 阶段 D — P1 存储

```powershell
cargo test -p oclivenewnew-tauri store_trait
cargo test -p oclive_kernel_runtime memory_engine
```

步骤 ID：`D-S1`、`D-S2`

### 阶段 E — P1 HTTP / OOCP / 重启

```powershell
cargo build -p oclivenewnew-tauri
```

后台启动 API（PowerShell）：

```powershell
$env:OCLIVE_ROLES_DIR = (Resolve-Path .\roles).Path
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$target = (cargo metadata --format-version=1 --no-deps | ConvertFrom-Json).target_directory
$bin = Join-Path $target "debug\oclivenewnew-tauri.exe"
Start-Process -FilePath $bin -ArgumentList "--api","--port","8420" -NoNewWindow
```

健康检查（最多 60s）：`curl -sf http://127.0.0.1:8420/health`

```powershell
cd examples/oocp-test-suite
npm test
cd ../..
npm run test:e2e:core-api-restart
cargo test -p oclivenewnew-tauri --test protocol_boundary_sidecar
```

步骤 ID：`E-BUILD`、`E-HEALTH`、`E-OOCP`、`E-RESTART`、`E-SIDECAR`  
最后 **E-STOP**：终止占用 8420 端口的进程。

### 阶段 F — P2 CLI（时间紧可 SKIP F-L3）

```powershell
cargo test -p oclive-cli
cargo test -p oclive_validation
```

### 阶段 H — P2 杂项（时间紧整段 SKIP）

```powershell
cargo test -p oclivenewnew-tauri --test loom_concurrency
cargo test -p oclivenewnew-tauri --test perf_chat_turns
cargo test -p oclivenewnew-tauri --test knowledge_pack
cargo test -p oclivenewnew-tauri --test role_cache_knowledge_reload
cargo test -p oclivenewnew-tauri --test tauri_api_smoke
npm run check:license
```

### 阶段 A-T — E2E（仅退出码，失败交 Track V）

```powershell
npm run build:e2e
npm run test:e2e:preview
npm run test:e2e:tauri-native
```

- `A-T0`：`build:e2e`  
- `A-T1`：preview — 非 0 → FAIL，报告写 `handoff_to_track_v: A-T1`  
- `A-T2`：tauri-native — 无法跑则 SKIP  

**不要**在 A 轨分析 Playwright 截图。

### Track A 报告

写入 `test-reports/codex-track-a-<时间>.md`，包含：

- `P0_BLOCKED: 是|否`
- 各阶段 PASS/FAIL/SKIP 汇总
- FAIL 清单
- `handoff_to_track_v` 是否勾选
- 结论：**内核可合并** 是|否（P0 全 PASS 且 E-OOCP、E-RESTART PASS）

---

## 五、Track V（视觉轨）— 仅多模态模型

> 无 vision 能力的模型 **不要执行本节**。

1. 读 `test-reports/codex-track-a-*.md`（若有 handoff）。
2. 若未 build：`npm run build:e2e`
3. 跑 Playwright：

```powershell
npm run test:e2e:preview
```

Windows 超时则用外置 preview：

```powershell
# 终端1
npm run preview -- --host 127.0.0.1 --port 4180 --strictPort
# 终端2
$env:PW_TEST_USE_EXTERNAL = "1"
npm run test:e2e:preview
```

4. **若 exit_code ≠ 0 或 A 轨 handoff A-T1**：打开 `playwright-report/`、`test-results/*.png`，填写 `visual_assessment`（`product_bug` / `test_bug` / `infra` / `inconclusive`）。
5. 可选：`npm run test:e2e:tauri-native`（步骤 V-F2）。
6. 写入 `test-reports/codex-track-v-<时间>.md`。

**不要**重跑 Track A 的 cargo/clippy/OOCP。

---

## 六、合并结论（两轨完成后）

写入 `test-reports/codex-merged-<时间>.md`：

```markdown
# oclivenewnew 合并测试结论

| 轨道 | 报告 | 结果 |
|------|------|------|
| Track A | codex-track-a-*.md | P0_BLOCKED? |
| Track V | codex-track-v-*.md | UI 结论 |

## 发版判定
- [ ] 内核可合并（A：P0 + E-OOCP + E-RESTART 全 PASS）
- [ ] Web 可合并（V-F1 exit 0，或 V 判定 infra/test_bug 非产品回归）
```

---

## 七、一键任务（复制整段给 Codex）

### 任务 A — 文本模型（DeepSeek / Codex）

```text
你是 oclivenewnew 仓库的自动化测试 Agent。严格按 oclivenewnew/dev-notes/codex-testing/CODEX_测试指南.md 第四节 Track A 执行。

规则：
- 仓库根目录工作；先 npm install（若无 node_modules）
- 顺序 E0 → A-G1..G7 → B → C → D → E → F → H（H 可 SKIP）→ A-T0/A-T1/A-T2
- OOCP 必须 OCLIVE_HTTP_API_MOCK_LLM=1
- 不要读图片；不要执行 Track V
- 每项用第三节格式汇报；全部完成后写 test-reports/codex-track-a-<时间>.md
- 不要 git commit；不要改 git config；不要修改源码

开始执行。
```

### 任务 V — 多模态模型（GPT-4o / Claude vision）

```text
你是 oclivenewnew 仓库的视觉轨测试 Agent。严格按 oclivenewnew/dev-notes/codex-testing/CODEX_测试指南.md 第五节 Track V 执行。

规则：
- 不要重跑 cargo/clippy/OOCP
- 先读 test-reports/codex-track-a-*.md 的 handoff
- npm run build:e2e 后跑 test:e2e:preview；失败必须读 test-results PNG 并写 visual_assessment
- 写 test-reports/codex-track-v-<时间>.md
- 不要 git commit

开始执行。
```

---

## 八、发版判定速查

| 级别 | 条件 |
|------|------|
| **内核可合并** | Track A：P0 全 PASS + `E-OOCP` + `E-RESTART` PASS |
| **含 Web 壳** | 上式 + Track V：`V-F1` 终端 PASS，或 FAIL 但视觉判定为 infra/test_bug |
| **阻断** | Track A `P0_BLOCKED=是` |

---

详细步骤与故障排查见 `CODEX_TEST_RUNBOOK.md`、`CODEX_TEST_RUNBOOK_VISION.md`。
