# AI 审查 / 汇报核实协议（Verification Protocol）

**用途**：约束自动化助手、外部审查模型、人类维护者在输出**质量审查 / 优化汇报 / 带数字的结论**前的核实纪律。与 [AI_CHANGE_BOUNDARIES.md](./AI_CHANGE_BOUNDARIES.md)（**改什么**）互补：本文管 **怎么说才算数**。

**何时必读**：

- 全仓质量审查、对标报告、A−/P0/P1 分级汇报
- 引用第三方审查（DeepSeek / 其他模型）并拟入账 `TECHNICAL_DEBT_INVENTORY.md`
- [RECURRING_OPTIMIZATION_PLAYBOOK.md](./RECURRING_OPTIMIZATION_PLAYBOOK.md) **全档 / 半档**收尾（§7 模板）

**元纪律**：与 Playbook §9 一致——**防回退，非追完美**；核实是为了避免**错误优先级**，不是为了追求覆盖率数字。

---

## 1. 交付物分级

| 级别 | 定义 | 必须附带的证据 |
|------|------|----------------|
| **L0 观察** | 印象、方向、待查假设 | 标注「未核实」 |
| **L1 单点事实** | 某一文件/命令可复现的结论 | 复现命令 + `HEAD` 短 SHA + 日期 |
| **L2 度量结论** | 含计数、比例、严重度排序 | 口径声明 + 命令输出摘要 + 排除规则 |
| **L3 入账建议** | 拟写入技术债 / 改 CI 门禁 | L2 证据 + 与 SSOT 对照 + 愿景影响 (V1–V4) |

**禁止**：将 L0 直接标为 P0/P1 或写入技术债 **OPEN** 行。

---

## 2. 强制核实规则（按主题）

### 2.1 测试与「覆盖率」

**禁止**用单一「测试行数 ÷ 生产行数」代表全项目质量，除非在报告中写明口径。

**本仓测试分层 SSOT**（汇报时必须引用此表，不得自造条数）：

| 层 | 位置 | 条数 / 规模 SSOT | 验证命令 |
|----|------|------------------|----------|
| 工程门禁 | dimension5 | 检查项总数以脚本结尾 `PASS (N checks)` 为准；`--ci` 的 SKIP 仍计入结果 | `node scripts/dimension5-acceptance.mjs --ci` |
| OOCP 黑盒 | `examples/oocp-test-suite/` | **S0–S12** + S15 SSE；可选 S13/S14 | `creator-docs/testing/OOCP_TEST_SUITE.md` |
| invoke 热路径 | `distros/desktop-tauri/tests/invoke_hotpath_matrix.rs` | **13** 条 `*_impl` + `process_message` | `INVOKE_HOTPATH_MATRIX.md` |
| 桌面集成测 | `distros/desktop-tauri/tests/` | 多文件；行数随版本变 | `cargo test -p oclivenewnew-tauri` |
| 内核 lib 单测 | `oclive_kernel_host` / `oclive_kernel_runtime` 等 `#[cfg(test)]` | 非固定；**不得**声称某文件「零单测」而未 `rg` | `rg '#\[test\]' <path>` |
| **Doctest（rustdoc 示例）** | 各 crate `///` 三反引号块 | 非固定；**`--lib` 与 `npm run check:rust` 不跑** | `cargo test --workspace --doc`（= `npm run check:rust:test:all` 的一部分） |
| 前端烟测 | Vitest + Playwright preview | `npm run test:unit`；`test:e2e:preview`（Ubuntu CI） | `creator-docs/testing/OVERVIEW.md` |
| Fuzz | `kernel/fuzz/fuzz_targets/` | **7** 目标 | `creator-docs/testing/FUZZING.md` |

> **⚠️ 本地绿 ≠ 远程绿（doctest 盲区）**：日常门禁 `npm run check:rust`（= `cargo test --workspace --lib`）与 Playbook 基线 `cargo test -p oclive_kernel_host --lib` **均跳过 doctest**；CI 的 `rust` job 跑 `cargo test --workspace`（**含 doctest**）。**改动公开 DTO 字段 / trait 签名 / crate 重命名 / 公开 re-export 路径后，必须 `cargo test --workspace --doc`**，否则 rustdoc 示例漂移会在本地全绿却让 CI 硬门禁红（2026-06-25 实例：`AgentInput` 加字段、crate 改名后三处 doctest 在 `--lib` 下不可见）。

**声称「某模块无单测」前必须**：

```powershell
rg '#\[test\]' kernel/crates/oclive_kernel_host/src/domain/<module>.rs
rg '<ModuleName>' distros/desktop-tauri/tests
```

**声称「覆盖率低」时须声明口径**，例如：`kernel/**/*.rs` 中路径含 `tests` 的行数比、或 `cargo llvm-cov`（若未跑则不得写百分比）。

---

### 2.2 `.unwrap()` / `.expect()` / panic 风险

**禁止**对 `oclive_kernel_host/src` 做全文件 `rg '\.unwrap\(\)'` 后直接称为「生产热路径 N 处」。

**生产路径统计须**：

1. 排除 `tests/`、`tests.rs`
2. 排除 `#[cfg(test)]` **之后**的代码块（文件内联测试模块）
3. 区分 `.unwrap()`、`.unwrap_or()`、`.expect()`（三者语义不同）

**参考命令（Node，仓库根）**：

```powershell
node -e "const fs=require('fs'),path=require('path');function walk(d,a=[]){for(const e of fs.readdirSync(d,{withFileTypes:true})){const p=path.join(d,e.name);if(e.isDirectory()){if(e.name==='target')continue;walk(p,a);}else if(e.name.endsWith('.rs'))a.push(p);}return a;}let prod=0,test=0;for(const p of walk('kernel/crates/oclive_kernel_host/src')){let t=fs.readFileSync(p,'utf8');const n=(t.match(/\.unwrap\(\)/g)||[]).length;if(!n)continue;if(p.endsWith('tests.rs')){test+=n;continue;}const parts=t.split(/#\[cfg\(test\)\]/);prod+=(parts[0].match(/\.unwrap\(\)/g)||[]).length;test+=n-prod;}console.log({prod,test});"
```

（2026-06-25 快照：`oclive_kernel_host` 生产路径 `.unwrap()` **0**，测试块内 **~135**。）

---

### 2.3 供应链（npm / cargo）

| 来源 | SSOT | 禁止 |
|------|------|------|
| Rust 漏洞级 | [KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md) + `cargo audit` | 用模型记忆代替本地 audit |
| GTK3 等 warning 忽略 | [.cargo/audit.toml](../.cargo/audit.toml)（**11** 条） | 称为「未文档化」 |
| npm | `npm audit --omit=dev`；CI `npm-audit` 为 **continue-on-error** | 将可见性 job 红 X 当作 main 阻塞 |

**英文安全文档**：`creator-docs-en/security/KNOWN_VULNERABILITIES.md` **存在**；汇报「无英文版」前须 `glob` 核实。可报告「英文扫描日期滞后于中文」。

---

### 2.4 远程分支 / CI / 门禁

| 声称 | 核实方式 |
|------|----------|
| dependabot 分支数 | `gh api repos/<owner>/<repo>/branches --paginate` 过滤 `dependabot`（**禁止**沿用旧帖数字） |
| main 是否可合 | `gh run list --limit 5` + 失败 job 逐步日志 |
| 硬门禁 vs 可见性 | 读 `.github/workflows/ci.yml` 的 `continue-on-error` |

**硬门禁（红 = 不能合）**：`rust`、`oocp-test-suite`、`frontend`（ubuntu Playwright）、`cross-host-e2e`、`dimension5-acceptance`、`cargo-audit`、`stale-paths`、`layering-ratchet` 等 **未**标 `continue-on-error` 的 job。

**可见性（红 X 可存在）**：`npm-audit`、`loom`、`fuzz`、`e2e-tauri`、`cli-bench`、`visual-presentation-smoke` 等。

---

### 2.5 文档与中英 parity

- 文件数：`creator-docs/` vs `creator-docs-en/` 须 `glob` 计数，不用约数。
- 「关键文档缺英文」：逐路径 `Test-Path` / `glob`，不可类推。
- CHANGELOG parity：以 `node scripts/check-changelog-parity.mjs` 为准（dimension5 项）。

---

### 2.6 第三方审查报告（DeepSeek 等）

1. 默认状态：**待核实（L0）**
2. 每条 P0/P1 **至少一条**本协议 §2 中的复现命令验证后，才可升为 L2/L3
3. 常见误报模式（本仓已发生）：
   - 大文件末尾 `#[cfg(test)]` 未读 →「零单测」
   - unwrap 全文件计数 →「编排热路径 panic」
   - dependabot / 文档数量未 `gh`/`glob` → 数字偏差

---

### 2.7 代码冗余 / 过度工程 / 「不简洁」声称

「代码冗余 / 不够简洁 / 过度工程 / 认知负担高」属 **L2 度量结论**，须附：

1. 具体 **`文件:行`** + 重复块数量或重复字段数（如「6 个构造函数各手写 33 字段、其中 ~18 字段恒为 `None`」）
2. 一个 **行为等价** 的收敛方案（`#[derive(Default)]` + `..Default::default()` / 共享 helper / 删死代码），而非仅「代码混乱」印象
3. 收敛验证命令（相关测试 + `cargo test --workspace --doc`）

**禁止**：凭印象写「这块代码很乱 / 应该重构」却无 `文件:行` 与等价方案；或把 §9 之外的大重构当作「优化」入账（见 [AI_CHANGE_BOUNDARIES.md](./AI_CHANGE_BOUNDARIES.md) G9）。

---

## 3. 汇报模板（审查 / 优化轮次必填）

```markdown
## 审查轮次（YYYY-MM-DD · HEAD <sha> · 档位 快/半/全）

### 基线
- dimension5: PASS/FAIL（命令 + 日期）
- GitHub CI main 最近 run: <url/结论>
- 本地额外: `cargo test -p oclive_kernel_host --lib` 等

### 发现清单（L2+ 才可标优先级）
| ID | 级别 | 声称 | 核实命令 / 证据 | 愿景 | 处置 |
|----|------|------|-----------------|------|------|

### 本轮 Done / Deferred
### 误报剔除（第三方审查若适用）
```

---

## 4. 与 Playbook / 技术债的衔接

- **快档**：§2 基线 + 路径 ratchet + 本协议 §2.4 CI 一眼
- **半档**：+ 维度一/四/七 + 测试分层表对照
- **全档**：+ §7 综合评分；新债入 [TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md)，编号 `D-AI-*`（流程）、`D-MAINT-*`（维护）、`D-DOC-EN-*`（英文滞后）等

---

## 5. 文档引用核实

**禁止**在审查报告 / 技术债建议中引用下列来源作为 **现行行为** SSOT：

| 禁止作 truth | 改用 |
|--------------|------|
| `handoff/archive/*` · `04_4.6_PROJECT_TRUTH_CHECKLIST.md` | BUS_FACTOR + 源码 |
| 已完成 Phase closure（如 USER_IDENTITY Phase2 设计报告） | 源码 + MODULE_MAP §11 |
| AGENTS 内核长节（未与 MODULE_MAP 对齐的段落） | [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) |
| 复制 MODULE_MAP 表格到其他 handoff 新文 | **链接** MODULE_MAP |

**模块/槽位相关结论**须对照 [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md)；**文档分责**见 [`handoff/README.md`](./README.md) §文档分责 · G10–G16。

**文档类 L2/L3 结论额外要求**：

- 声称「文档缺失 / 应新建 XX.md」前：必须证明 [`handoff/README.md`](./README.md) §文档分责 **无** 覆盖 SSOT（G11）。
- 声称「文档与源码不一致」：须给出 **SSOT 路径 + 源码路径** 各一，禁止只引用 AGENTS 长节。
- 文档改动汇报须列出：**只改了哪一份 SSOT**；若 >1 份，须说明为何非 G12 违规或 maintainer 明示。

---

## 6. 相关

- [AI_CHANGE_BOUNDARIES.md](./AI_CHANGE_BOUNDARIES.md) — 改动边界
- [RECURRING_OPTIMIZATION_PLAYBOOK.md](./RECURRING_OPTIMIZATION_PLAYBOOK.md) — 巡检流程
- [TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md) — 台账
- [INVOKE_HOTPATH_MATRIX.md](./INVOKE_HOTPATH_MATRIX.md) — invoke 条数 SSOT
- [MODULE_MAP_AND_HANDOFF.md](./MODULE_MAP_AND_HANDOFF.md) — 模块注册表
- [handoff/README.md](./README.md) §文档分责
