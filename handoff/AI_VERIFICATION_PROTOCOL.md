# AI 审查 / 汇报核实协议（Verification Protocol）

**SSOT 范围**：本文定义审查结论的证据等级、数字核实、文档声明的代码支撑调查限度与汇报口径；巡检档位和维度范围以[多轮巡检手册](./RECURRING_OPTIMIZATION_PLAYBOOK.md#1-触发条件与档位)为准。
**最后更新**：2026-10-03（补充技术债实施的证据停止线与完成范围约束）。

**用途**：约束自动化助手、外部审查模型、人类维护者在输出**质量审查 / 优化汇报 / 带数字的结论**前的核实纪律。与 [AI_CHANGE_BOUNDARIES.md](./AI_CHANGE_BOUNDARIES.md)（**改什么**）互补：本文管 **怎么说才算数**。

**何时必读**：

- 全仓质量审查、对标报告、A−/P0/P1 分级汇报
- 活跃文档声明 → 实现/测试/运行证据的支撑调查
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
| OOCP 黑盒 | `examples/oocp-test-suite/` | 默认 **16** 场景（S0、S0b、S1–S12、S15、S16）；可选 S13/S14 | `creator-docs/testing/OOCP_TEST_SUITE.md` |
| invoke 热路径 | `distros/desktop-tauri/tests/invoke_hotpath_matrix.rs` | **13** 条 `*_impl` + `process_message` | `INVOKE_HOTPATH_MATRIX.md` |
| 桌面集成测 | `distros/desktop-tauri/tests/` | 多文件；行数随版本变 | `cargo test -p oclivenewnew-tauri` |
| 内核 lib 单测 | `oclive_kernel_host` / `oclive_kernel_runtime` 等 `#[cfg(test)]` | 非固定；**不得**声称某文件「零单测」而未 `rg` | `rg '#\[test\]' <path>` |
| **Doctest（rustdoc 示例）** | 各 crate `///` 三反引号块 | 非固定；**`--lib` 与 `npm run check:rust` 不跑** | `cargo test --workspace --doc`（= `npm run check:rust:test:all` 的一部分） |
| 前端烟测 | Vitest + Playwright preview | `npm run test:unit`；`test:e2e:preview`（Ubuntu CI） | `creator-docs/testing/OVERVIEW.md` |
| Fuzz | `kernel/fuzz/fuzz_targets/` | **7** 目标 | `creator-docs/testing/FUZZING.md` |

> **⚠️ 本地绿 ≠ 远程绿（doctest 覆盖）**：日常 lib 测试和 Playbook 的 `--lib` 基线跳过 doctest；CI 普通 workspace 测试只覆盖默认启用的库文档测试。runtime、Host 与桌面 manifest 当前写 `doctest=false`，普通测试默认省略这些 crate 的 doctest；显式 `--doc` 仍可选择库文档测试，须按实际命令和退出码核覆盖。**改动公开 DTO 字段 / trait 签名 / crate 重命名 / 公开 re-export 路径后，仍须 `cargo test --workspace --doc`**；列清单不等于执行通过。2026-06-25 的历史实例（`AgentInput` 加字段、crate 改名后三处 doctest 在 `--lib` 下不可见）保留其当时范围，不当作当前全部 crate 默认有覆盖的依据。选择规则见 [Cargo 测试文档](https://doc.rust-lang.org/cargo/commands/cargo-test.html)。

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
| npm 生产依赖 | `npm audit --omit=dev --audit-level=high`；CI `npm-audit` 中为**硬门禁** | 把完整 dev graph 命中误报为生产暴露，或把生产 0 扩写成全依赖 0 |
| npm 开发工具链 | `npm audit --audit-level=high` + `npm ls`；CI `npm-audit` 中同为**硬门禁**，当前基线见 **K-SUPPLY-12** | 用 `--force` / 无证据 override 掩盖 peer 或审计冲突 |

**英文安全文档**：`creator-docs-en/security/KNOWN_VULNERABILITIES.md` **存在**；汇报「无英文版」前须 `glob` 核实。可报告「英文扫描日期滞后于中文」。

---

### 2.4 远程分支 / CI / 门禁

| 声称 | 核实方式 |
|------|----------|
| dependabot 分支数 | `gh api repos/<owner>/<repo>/branches --paginate` 过滤 `dependabot`（**禁止**沿用旧帖数字） |
| main 是否可合 | `gh run list --limit 5` + 失败 job 逐步日志 |
| main 是否强制 CI | 分支保护 / ruleset API 中 required context 必须包含稳定 `ci-gate`；仅看到 Actions 在跑不等于门禁生效 |
| 硬门禁 vs Nightly | 同时读 `.github/workflows/ci.yml` 与 `.github/workflows/nightly-advisory.yml`；不要仅凭 job 名称判断 |

**硬门禁（红 = 不能合）**：GitHub 分支保护绑定稳定 `ci-gate`。该汇总 job 检查 `ci-impact-plan` 成功，并核对全量模式的全部责任组成功，或选择模式的 selected job 成功且 unselected job 确为 skipped。`rust`、`oocp-test-suite`、`frontend`（Ubuntu Playwright）、`cross-host-e2e`、`dimension5-acceptance`（唯一持有主工作流 `cargo audit`）、`npm-audit`、`stale-paths`、`layering-ratchet` 等仍是受信责任组，但不单独配置成可能永远等待的 required context。

**Nightly/手动证据（不挡 main，但失败不可吞）**：`loom`、`fuzz`、`e2e-tauri`、`cli-bench`、`visual-presentation-smoke` 位于 `nightly-advisory.yml`；它们失败会让该工作流变红并按项保留日志/artifact，不能汇报成通过。主工作流不再允许 `continue-on-error: true`；规划器失败会触发全量执行并让 `ci-gate` 失败。

#### CI 推送节奏与证据绑定

| 阶段 | 默认动作 | 远端全量 CI |
|------|----------|-------------|
| 开发切片 | 受影响窄测 + 本地提交；未知范围先按 M，触及契约/编排/权限再升档 | 不作为日常调试循环 |
| 协作同步 | 推送逻辑完整提交；未冻结时优先保持无 ready PR 的远端分支 | 不等待、不声称 Done |
| 里程碑 / L 结案 | applicable 本地全量门禁 → 冻结 HEAD → 一次推送 | 必须绑定目标 SHA 并等待终态 |
| 远端失败 | 读取失败 job 日志 → 根因窄测 → 修复 | 修复完成后再推；禁止无改动反复重跑掩盖确定性失败 |
| 远端已绿 | 保持 HEAD 不变；run URL 先记 PR 评论/交付报告 | 禁止为回写证据单独追加提交而再次触发全矩阵 |

- **批次冻结线**：同一主题、相互依赖的低/中风险切片先以定向测试和适用的模块检查推进；不要把每个子提交都命名为独立里程碑并各跑一次本地全量和远端全量。待这一批预定范围完成、差量稳定，再对最终 HEAD 跑一次适用的本地完整门禁并取得该 SHA 的正式远端证据。安全、持久化、Kernel/Host 公共契约、CI 控制面或发布边界的独立验收点，可单独冻结和验证，并记录为何不能合批。
- **现行触发事实**：Push 与非纯文档 ready PR 仍按当前策略运行全量；节省无谓全量运行主要靠控制推送与验收节奏，不能把未推送的本地通过、草稿 PR 的 `ci-draft-gate` 或旧 SHA 的绿色结果说成新 SHA 的正式验收。中途协作确需推送时，如触发全量，应如实记录该次运行，但不必为了继续同批开发而等待它并把它另算一次结案。
- `concurrency.cancel-in-progress` 只会取消旧 run，不能收回已经消耗的 runner 时间；因此“频繁推送后靠自动取消”不算合理节奏。
- 只有与报告中目标 **完整 SHA** 一致的成功 run 才是当前远端证据。后续实质提交必须重新验证；纯证据回写不应制造新的 HEAD。
- 技术债需要仓库内证据时，优先随下一次实质提交一并回写；在此之前保持原状态并链接 PR 评论，不得用旧 SHA 冒充新 HEAD 已验证。

#### 领域感知 CI 选择性执行（Stage 3.1 · draft development）

- `ci-impact-plan` 发布 `plan.json`、`execution.json`、Job Summary 与 artifact；`domain-aware-pr-v2` 只有在 PR plan 非 shadow、policy 为 `pull_request`、无 warning/full fallback、直接/受影响模块和 selected job 均非空时才允许选择性执行。
- PR 选择结果必须由 comparison base 中的受信规划器、影响契约与执行策略生成，gate 也使用该基线校验；若基线尚无策略脚本（首次上线 bootstrap），必须全量。
- 满足安全条件的草稿 PR 可按 `selected_validators[].workflow_jobs` 执行，但结果名是 `ci-draft-gate`，不得把它配置为 main required context，也不得汇报成最终可合并证据。
- `ready_for_review` 必须在当前提交上重新触发 CI。ready 时只有直接模块为 `oclive.docs` 的既有 Canary 可继续选择性执行；其他 ready PR、Push、未知/高风险路径和规划异常全部全量，并由正式 `ci-gate` 验收。
- `oclive ci plan` 的 `selected_validators` 仍不是“已执行”证据；验收必须引用对应 gate 及实际 job 终态。手动 `--shadow` 计划禁止用于跳 job。
- `npm run ci:shadow-samples` 的 JSON/Markdown 是**规划模拟**：当前基线为 21/21（18 targeted / 3 fail-safe），只能证明固定样本仍按当前规则路由；不得把它汇报成 21 次远端 CI、零漏选或 validator 已执行。
- 全量 run 自动上传 90 天 `oclive-ci-compare-*`：只有 plan/execution、同一 workflow SHA、完整终态 job 快照和完整结果同时成立时，`authoritative_ci_comparison` 才可为 true。选择性 run、快照缺失或未终态只算 observational；`false_negative_candidates` 还需维护者裁决，不能自动删改影响边。
- 未映射路径、损坏模块描述、未知 required 扩展及中央高风险规则会使当前 policy `full_fallback`；这代表必须执行全量，不代表全量已经通过。
- 规划器异常时所有责任组通过 fail-safe 条件运行，但 gate 仍须失败；修复规划器后重新验证，不能把降级运行粉饰成绿。
- 新 ready 类别仍处 Stage 2 Compare；扩大 ready 选择面前必须更新 [`SOMEDAY_TOOLCHAIN_CI.md`](../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md) 和本协议。

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

### 2.8 关联改动与模块兼容性结论

声称「已完成」「已兼容」「不会影响其它模块」前，须按 [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G17 给出能力闭环证据：

1. **影响链**：生产者 → wire/DTO/event/RPC → 适配/权限 → 消费者 → 状态/回退 → 测试。
2. **核对范围**：内核、Tauri、`distros/shared`、Chat Pro/Theater、官方插件、角色包/编写器；不适用项明确写「无需改 + 原因」。
3. **兼容口径**：区分 schema 兼容、结构兼容、行为兼容、跨版本兼容；只跑构建不得声称四者全部成立。
4. **证据**：边界两侧至少各一条测试；Chat Pro / 目录插件另附 `npm run check:module-compat`。Breaking 变更附迁移/回退与 [`BREAKING_CHANGE_PROCESS.md`](./BREAKING_CHANGE_PROCESS.md) 清单。

**禁止**：以「改动文件少」证明风险低；以插件 `version` 相同证明宿主兼容；只验证 Vue 入口而不验证 iframe 回退（或反之）。

---

<a id="doc-code-support-audit"></a>

### 2.9 Document → Code 支撑调查：预算与停止线

**适用范围**：只选活跃 SSOT 中对**当前行为或验收**作出的重要声明；目标、草案、历史结论先标明性质，不当作已交付承诺。调查只读，不在同一批次顺手修代码、补测试、改债务状态或重写原声明。先记受检完整 SHA、工作树状态、入选声明及排除理由；本节不另造巡检档位，也不代替开发任务的适用门禁。

下表的 **D1–D3 是调查深度**，与本文 §1 的 **L0–L3 证据等级**彼此独立；达到 D3 不等于具备 L3 入账证据。

| 层级 | 最多查到哪里 | 进入与停止条件 |
|------|--------------|----------------|
| **D1 主路径** | 精确声明及 owner → 一条生产入口/主调用链 → 一项直接测试或对应 CI/实机证据；每条最多 **15 分钟** | 默认层。能回答“说了什么、谁负责、代码在哪里、证据是否足够”即停止；不为低概率假设继续追分支 |
| **D2 重要边界** | 在 D1 之上，最多再核 **两个**与结论有关的异常、回退、跨层或生命周期边界；每条额外最多 **20 分钟** | 仅当 D1 有具体矛盾、责任不清，或声明涉及安全、持久化、Kernel/Host 权责及现实高频路径。足以分类就停止 |
| **D3 反例压力测试** | 一次只验证一个能推翻现有判断的具名假设；首个专项最多 **60 分钟** | **不在首轮自动开启**。须先记录触发理由、隔离环境、身份/副作用和止点；只有架构边界、真实安全/数据损坏风险、已发生的关键失败或可能推翻冻结结论才启动。超时后记 unknown 并另定切片，不递增预算追到“绝对无疑” |

**每批硬止点**：最多 **8 条声明或 120 分钟，先到即停**；宽声明先拆成可定位的一句话，按用户可见影响和架构风险排序。首轮以广度建图，报告“入选 / 已查 / unknown / 未入选”四个数与筛选口径；覆盖比例只描述选样，不作为通过线。达到预算的项保持 `Unknown` 或限定的证据缺口，不为填满表格臆造结论。Code → Document 反查另列批次，不沿一条声明反向扩成全仓测绘；已发现的真实安全/数据损坏事故转独立处理，不在本批静默扩查。

**逐项记录**：`声明原句与锚点 | 当前/目标/历史 | 责任层 | 实现入口 | 直接测试及其是否在目标 SHA 通过 | CI/实机证据 | 已查层级/耗时 | 分类 | 未查边界/下一动作`。分类只用 `Supported`、`Evidence gap`、`Implementation gap`、`Semantic drift`、`Boundary leak`、`Unknown`；必要时另记风险标签。`Supported` 必须由**该声明所需层级**的实现和实际通过证据支撑：单测不替代真实设备，存在测试文件不等于测试已运行，通用 CI 绿灯也不自动证明某条未映射声明。仅 `rg` 未命中不能判 `Implementation gap`；源码与文档冲突须按 §5 各给一个现行锚点。分类是调查结果，不自动生成新债、优先级或 Done；入账仍须满足 L3 入账证据和 owner/去重审查。

**升级口径**：四个 D1 问题已答且证据足够 → `Supported — STOP`；有实现但缺所需测试或实机证据 → `Evidence gap — STOP`，另排验证；预算内不能确定 → `Unknown — STOP`，给出最短下一步。只有文档与代码均有合理依据且会改变产品/架构语义时，才向维护者提出具体取舍；普通缺口继续登记，不反复请求确认。

---

### 2.10 技术债实施：足以修复与验收即停止补证

**目标是消除已定义的问题**。调查只服务于确认原因、选择改法和判断修复是否有效；不得把“继续找到未测边界”当成默认下一步。进入实施时用一句话写清待修问题、影响和有限完成条件，沿用原债范围，不因测试推进自动扩大验收范围。

- 原因、责任位置和预期行为已明确，便实施最小修复；验证以能识别原问题的定向回归和受影响边界为主。
- 已有实现和证据足以支持修复结论时，停止新增测试或调查，进入适用门禁、结案或下一债。仅待目标 SHA 正式 CI 的项记为待验收，不因此再发明业务验证任务。
- **未覆盖不等于缺陷，也不自动成为阻塞项**。补测前必须说明：会影响哪项修复决策或现行承诺；为什么现有证据不足。只为增加信心、填满组合或证明所有路径而补测，不予排期。
- **边际收益下降即停，是否深入由维护者确认**：若新增验证已不改变修复方案或验收判断，停止该项；简要列出已知结论、剩余疑点、深入的预期收益与成本，等待维护者决定。不得自行续预算、换切片名称或增加组合绕过停止线；可继续其它已明确的债务。正常修复所需的定向回归仍按既定范围完成。
- 只有真实反例、已发生的重要失败、安全/数据损坏风险或已明确承诺的能力缺口，才触发新的有界调查；高风险仍须完成必要验证，不能以“少调查”为由忽略具体问题。
- 文档承诺与代码确有差距时，作为实现或语义债处理；不得仅缩减文档、删测试或降低门禁来制造完成。存在产品/架构取舍则登记决策点，按维护者要求跳过并汇报。
- 汇报优先说明修复了什么、用户会看到什么变化、还有什么具体阻塞；测试数量、证据文件数量和新增调查切片不作为偿债进度。

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
### 关联影响闭环（已改 / 已核对无需改 / 兼容与回退 / 跨边界测试）
### 误报剔除（第三方审查若适用）
```

---

## 4. 与 Playbook / 技术债的衔接

- **快 / 半 / 全档范围**只以 [巡检手册 §1](./RECURRING_OPTIMIZATION_PLAYBOOK.md#1-触发条件与档位) 为准；本文不另列维度，避免两套档位口径分叉。每档报告均按本协议 §2.4 核对远端 CI 的目标 SHA 与结论，并列明未测范围。
- **全档评分**按巡检手册 §7；快档与半档不沿用历史评分。新债先满足本协议 L3 再对照 [TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md) 去重、确定归属和编号；不能将观察直接升为 OPEN 或把本地绿灯写成 Done。

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
