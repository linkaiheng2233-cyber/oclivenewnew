# Codex 自动化测试运行手册（双轨）

**版本**：v2.1 · 分模型执行  
**仓库根目录**：`oclivenewnew/`

---

## 双轨一览

| 轨道 | 文档 | 推荐模型 | 能力 |
|------|------|----------|------|
| **Track A · 文本轨** | 本文 **第 I 部分** | DeepSeek V3/R1、Codex、Claude（纯文本）、Composer Agent | 终端、日志、JSON、退出码 |
| **Track V · 视觉轨** | [`CODEX_TEST_RUNBOOK_VISION.md`](CODEX_TEST_RUNBOOK_VISION.md) | GPT-4o、Claude（vision）、Gemini（vision） | Playwright 截图 / HTML 报告判读 |

**执行顺序**：**先 A 后 V**。A 负责内核与协议；V 负责 E2E 失败时的 UI 定性（及可选桌面冒烟）。

**发版判定（合并后）**：

| 级别 | 条件 |
|------|------|
| 内核可合并 | Track A：P0 全 PASS，且 `E-OOCP`、`E-RESTART` PASS |
| 含 Web 壳 | 上式 + Track V：`V-F1` 终端 PASS，或 FAIL 但 `visual_assessment=infra/test_bug` 且产品无回归 |
| 阻断 | Track A `P0_BLOCKED=是` |

---

## 0. 共同规则

1. 工作目录：仓库根；无 `node_modules/` → `npm install`。
2. 每项立即汇报（**§2**）；禁止改 `git config`；未要求不 `commit`。
3. HTTP/OOCP：`OCLIVE_HTTP_API_MOCK_LLM=1`，不依赖 Ollama。
4. Track A：P0 任一 FAIL 仍跑完 A 内全部 P0，最终标 `P0_BLOCKED=是`。
5. 报告输出（无目录则创建）：
   - A → `test-reports/codex-track-a-<YYYYMMDD-HHmm>.md`
   - V → `test-reports/codex-track-v-<YYYYMMDD-HHmm>.md`
   - 合并 → `test-reports/codex-merged-<YYYYMMDD-HHmm>.md`（**§5** 模板）

---

## 1. 环境检查 — **E0**（两轨先做）

| 检查 | 命令 | 预期 |
|------|------|------|
| 仓库根 | `git rev-parse --show-toplevel` | 路径含 `oclivenewnew` |
| Node / Rust | `node -v` / `cargo -V` | 有输出 |
| 依赖 | 无 `node_modules` → `npm install` | 退出码 0 |

---

## 2. 单项汇报格式

```text
[PASS|FAIL|SKIP] <轨道>-<步骤ID> <名称>
命令: <完整命令>
摘要: passed=… | failed=… | SKIP 原因
失败: <stderr/stdout 最多 30 行>
```

Track A 步骤前缀：`A-G*`、`B-M*`、`C-C*`、`D-S*`、`E-*`、`F-L*`、`G-D*`、`H-X*`、`A-T*`、`J-P*`  
Track V 步骤前缀：`V-*`（见视觉轨文档）

---

# 第 I 部分 — Track A（文本轨 · 无需多模态）

**禁止**：解析 PNG/截图、设计稿、Playwright HTML 报告图像。  
**E2E（A-T）**：只记录 **退出码 + 文本日志**；非 0 时写 `handoff_to_track_v: A-T1` 或 `A-T2`。

---

### 阶段 A — P0 全局门禁

| 步骤ID | 命令 | 预期 |
|--------|------|------|
| A-G1 | `npm run build` | 退出码 0（Tauri / `dist/`） |
| A-G2 | `npm run check:rust:fmt` | 退出码 0 |
| A-G3 | `npm run check:rust:clippy` | 退出码 0 |
| A-G4 | `npm run check:rust:test` | 全 pass |
| A-G5 | `npm run check:rust:test:all` | 全 pass |
| A-G6 | `npm run test:unit` | Vitest 全绿 |
| A-G7 | `cargo test --workspace` | 全 pass |

可选：`npm run check:release`（仍须分项汇报 A-G1–G6 可映射项）。

---

### 阶段 B — P0 插件与多槽

```powershell
cargo test -p oclivenewnew-tauri --test plugin_backends_v2_resolve
cargo test -p oclivenewnew-tauri --test slot_runner_p4
cargo test -p oclivenewnew-tauri --test blueprint_v2_role_load
cargo test -p oclivenewnew-tauri --test blueprint_v2_mumu_load
cargo test -p oclivenewnew-tauri --test save_role_slot_registry
cargo test -p oclivenewnew-tauri --test slot_resolver_v3
cargo test -p oclivenewnew-tauri --test permission_three_way_consistency
```

| 步骤ID | 测试 |
|--------|------|
| B-M1 … B-M7 | 与上表顺序一致 |

---

### 阶段 C — P0 对话编排与 API

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

步骤 ID：`C-C1` … `C-C9`。

---

### 阶段 D — P1 存储与记忆引擎

| 步骤ID | 命令 |
|--------|------|
| D-S1 | `cargo test -p oclivenewnew-tauri store_trait` |
| D-S2 | `cargo test -p oclive_kernel_runtime memory_engine` |

---

### 阶段 E — P1 HTTP / OOCP / 重启

| 步骤ID | 操作 |
|--------|------|
| **E-BUILD** | `cargo build -p oclivenewnew-tauri` → 退出码 0 |

**E-SRV**（PowerShell，仓库根，后台）：

```powershell
$env:OCLIVE_ROLES_DIR = (Resolve-Path .\roles).Path
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$target = (cargo metadata --format-version=1 --no-deps | ConvertFrom-Json).target_directory
$bin = Join-Path $target "debug\oclivenewnew-tauri.exe"
Start-Process -FilePath $bin -ArgumentList "--api","--port","8420" -NoNewWindow
```

**E-SRV**（Bash）：

```bash
export OCLIVE_ROLES_DIR="$PWD/roles"
export OCLIVE_HTTP_API_MOCK_LLM=1
target=$(cargo metadata --format-version=1 --no-deps | jq -r .target_directory)
"$target/debug/oclivenewnew-tauri" --api --port 8420 &
```

**E-HEALTH**（最多 60s，每 2s）：`curl -sf http://127.0.0.1:8420/health`

| 步骤ID | 命令 |
|--------|------|
| **E-OOCP** | `cd examples/oocp-test-suite; npm test; cd ../..` |
| **E-RESTART** | 仓库根 `npm run test:e2e:core-api-restart` |
| **E-SIDECAR** | `cargo test -p oclivenewnew-tauri --test protocol_boundary_sidecar` |
| **E-STOP** | 终止占用 8420 的 `oclivenewnew-tauri` |

---

### 阶段 F — P2 CLI / 校验

| 步骤ID | 命令 |
|--------|------|
| F-L1 | `cargo test -p oclive-cli` |
| F-L2 | `cargo test -p oclive_validation` |
| F-L3 | `cargo run -p oclive-cli -- --experimental init --monolith --non-interactive --preset headless-api -o $env:TEMP\oclive-codex-monolith-smoke`（失败可 SKIP） |

---

### 阶段 G — P2 双核（默认 SKIP）

仅当 `CODEX_RUN_DUAL_CORE=1`：

- G-D1：`cargo test -p oclivenewnew-tauri --lib --features dual_core`
- G-D2：`cargo test -p oclivenewnew-tauri --test dual_core_happy_path`
- G-D3：`$env:OCLIVE_OOCP_INCLUDE_DUAL_CORE="1"` + OOCP（需阶段 E API）

---

### 阶段 H — P2 杂项（时间紧 SKIP）

| 步骤ID | 命令 |
|--------|------|
| H-X1 | `cargo test -p oclivenewnew-tauri --test loom_concurrency` |
| H-X2 | `cargo test -p oclivenewnew-tauri --test perf_chat_turns` |
| H-X3 | `cargo test -p oclivenewnew-tauri --test knowledge_pack` |
| H-X4 | `cargo test -p oclivenewnew-tauri --test role_cache_knowledge_reload` |
| H-X5 | `cargo test -p oclivenewnew-tauri --test tauri_api_smoke` |
| H-X6 | `npm run check:license` |

---

### 阶段 A-T — E2E **仅退出码**（交给 V 判图）

Playwright 需要 **e2e 构建**（含 Tauri invoke 桩），不是普通 `A-G1` build：

| 步骤ID | 命令 | 预期 |
|--------|------|------|
| **A-T0** | `npm run build:e2e` | 退出码 0 |
| **A-T1** | `npm run test:e2e:preview` | 退出码 0 → PASS；非 0 → FAIL + `handoff_to_track_v: A-T1` |
| **A-T2** | `npm run test:e2e:tauri-native` | 0 → PASS；否则 SKIP 或 FAIL + handoff |

Windows A-T1 超时：记 FAIL，备注 handoff（外置 preview 见视觉轨 V-F1）。

**本阶段覆盖用例（preview）**：`e2e/preview-shell.spec.ts`、`send-message.spec.ts`、`switch-role.spec.ts`、`install-plugin.spec.ts`（不含 `tauri-native.spec.ts`）。

---

### 阶段 J — 手工插件（默认 SKIP）

仅 `CODEX_RUN_MANUAL_PLUGIN=1`：J-P1 Remote / J-P2 directory-minimal / J-P3 llamacpp（RPC 文本，无需 vision）。

---

## Track A 报告模板

`test-reports/codex-track-a-<时间>.md`：

```markdown
# Track A 测试报告（文本轨）

- 模型: <DeepSeek|Codex|…>
- 平台: win32|linux|darwin
- P0_BLOCKED: 是|否

## 汇总
| 阶段 | PASS | FAIL | SKIP |

## FAIL 清单
1. A-… — …

## 结论
- 内核可合并: 是|否

## handoff_to_track_v
- [ ] A-T1 Playwright preview 需视觉轨
- [ ] A-T2 Tauri native 需视觉轨
- [ ] 无
```

---

## 4. Track A 一键提示（DeepSeek / 纯文本）

```text
执行 oclivenewnew/dev-notes/codex-testing/CODEX_TEST_RUNBOOK.md 第 I 部分（Track A only）。
不要读 CODEX_TEST_RUNBOOK_VISION.md；不要分析任何图片。
顺序: E0 → A → B → C → D → E → F →（G/H 按 SKIP）→ A-T0 → A-T1 → A-T2 → J。
A-T 前必须 npm run build:e2e。OOCP 必须 OCLIVE_HTTP_API_MOCK_LLM=1。
写入 test-reports/codex-track-a-<时间>.md。不要 git commit。
```

---

## 5. 合并报告模板（A + V 完成后）

`test-reports/codex-merged-<时间>.md`：

```markdown
# oclivenewnew 合并测试结论

| 轨道 | 报告 | 模型 |
|------|------|------|
| Track A | test-reports/codex-track-a-*.md | |
| Track V | test-reports/codex-track-v-*.md | |

## 判定
- [ ] 内核可合并（A：P0 + E-OOCP + E-RESTART）
- [ ] Web 可合并（V-F1 终端 PASS，或 V 判定 infra/test_bug）

## Track V 专责项
- Playwright 截图根因: …
```

---

## 6. Track A 故障排查

| 现象 | 处理 |
|------|------|
| 缺 `dist/`（Tauri build） | A-G1 |
| Playwright 找不到 mock | 先 **A-T0** `build:e2e` |
| OOCP 拒绝连接 | E-HEALTH / E-STOP 清端口 |
| A-T1 FAIL | **不要猜 UI** → Track V |

**视觉轨** → [`CODEX_TEST_RUNBOOK_VISION.md`](CODEX_TEST_RUNBOOK_VISION.md)
