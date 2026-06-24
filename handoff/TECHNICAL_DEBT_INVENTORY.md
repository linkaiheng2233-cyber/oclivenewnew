# Technical debt inventory

**Last updated:** 2026-06-25 (轮次 21 · Wave 0–5 全面优化)

**Product freeze (Theater v0):** **Lifted** — 朋友 cohort 产品门通过（7/10 卧槽）；模式 2 可开工；模式 3 仍冻结。见 [`theater/MODE2_UNFREEZE.md`](./theater/MODE2_UNFREEZE.md)。

**综合评分：** A− · 本地 dimension5 **十三检** PASS · workspace **doctest** 绿 · 审查汇报 SSOT：[`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md)

**下一动作：** **P1** — 模式 2 playtest 扩展；**K-SUPPLY-02** Release SHA256 asset（维护者）

**Verification (2026-06-25 轮次 21):** `cargo test --workspace --doc` PASS；`node scripts/dimension5-acceptance.mjs --ci` PASS；`cargo test -p oclive_kernel_host --lib` PASS。

---

## §1 活跃台账（OPEN · 开工清单）

| ID | 项 | 优先级 | 解冻/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **P0-STRANGER** | Theater 朋友 cohort 试玩（10 人） | **P0** | ≥60% 通过 · [`theater/PLAYTEST_MATRIX.md`](./theater/PLAYTEST_MATRIX.md) | **Done**（朋友 cohort 7/10 · 2026-06-25） |
| **K-DOC-17** | 注释英文化 batch 3 | P1 | `slot_runner.rs` · `kernel_strategy.rs` 等 | **Done**（轮次 16 复核：上述文件已为英文 `//!`/`///`） |
| **V-VSCODE-PERF-05** | VS Code F5 / `.vsix` 实机 | P1 | 姊妹仓 `oclive-vscode` 人工排期 | **OPEN**（cross-repo） |
| **K-CONTRACT-WIRING-01** | `extra_sections` 生产接线 | P2 | 首个外部插件作者 or Phase 5 通过后 | **OPEN** |
| **D-DOCDRIFT-01** | 重组后 normative 文档路径漂移（旧布局引用） | P0 | `check-stale-paths` 硬门禁绿 + `migrate-doc-paths` 路径存在性全过 | **Done**（轮次 17） |
| **D-SCRIPT-02** | `check-stale-paths.mjs` 误报/漏报（反例说明与行内路径） | P1 | 扩范围 + 修 pattern + 挂 dimension5 | **Done**（轮次 17） |
| **D-ORPHAN-04** | 残留空目录 `kernel/crates/models/` | P2 | 目录删除 + workspace 无引用 | **Done**（轮次 17） |
| **O-1** | `oclive_kernel_host` 编译期 `include_str!` 耦合 `distros/desktop-tauri/assets/plugin-bridge.iife.js` | P1 | 资产迁入 `kernel/crates/oclive_kernel_host/assets/` + copy 脚本改指向 | **Done**（轮次 18） |
| **O-2** | expert 孤儿前端（Vue/lib/test/i18n/API re-export，零 import） | P2 | 删除 + `role.ts`/locales 同步 + stale 文档措辞 | **Done**（轮次 18） |
| **D-DOC-RELOC-01** | 三份名实不符文档仍在 `creator-docs/`（VS Code 契约 / Studio 指南 / mumu 验收） | P2 | 物理迁至 `handoff/{vscode,studio,distros}/` + 原位 stub + 入链更新 | **Done**（轮次 18） |
| **K-SUPPLY-02** | Release 预编译内核 **SHA256SUMS**（防换包） | P1 | workflow + 脚本已入库；**维护者**首次 Release 挂 asset | **Partial** |
| **K-SUPPLY-03** | 插件安装后「请审本地源码」固定提示 | P2 | 市场/git/zip + CLI | **Done**（轮次 19） |
| **K-SUPPLY-04** | 前端 `npm-audit` 仅可见性（`continue-on-error`） | P2 | 连续 2 周期高危命中 → 升格硬门禁或文档豁免 | **OPEN** |
| **K-SUPPLY-05** | `deny.toml` `multiple-versions = warn` | P2 | 依赖树去重后改 `deny` | **OPEN** |
| **D-ORDER-01** | monorepo `roles` 路径 SSOT（27 集成测 + oclive-cli `join("roles")`） | P0 | `chat_pro_roles_dir()` / `tests/common` / `resolve_project_roles_dir()` | **Done**（条理优化 Wave A · 2026-06-24） |
| **D-ORDER-02** | `roles_dir.rs` debug 回退、`test_oocp.rs` 旧 `src-tauri` 路径 | P0 | 指向 `distros/chat-pro/roles` + `distros/desktop-tauri/Cargo.toml` | **Done**（Wave A） |
| **D-ORDER-03** | CI `cd fuzz`、Playwright `testDir`、`check:license` 插件路径、examples `../../roles` | P1 | 与 monorepo 布局一致 | **Done**（Wave B1/A5） |
| **D-ORDER-04** | `check-stale-paths` 仅扫 `.md` | P1 | 扩展 `.rs/.mjs/.sh/.yml` + dimension5 代码 ratchet | **Done**（Wave B2/B4） |
| **D-DOC-DRIFT-02** | AI 入口文档（rules/AGENTS/THREE_DISTRO/invoke 条数） | P1 | 与 BUS_FACTOR / INVOKE_HOTPATH_MATRIX 对齐 | **Done**（Wave C · 2026-06-24） |
| **D-DOC-DRIFT-03** | `KNOWN_VULNERABILITIES` quinn-proto 0.11.15 | P2 | 台账 + 扫描日期 | **Done**（Wave C4） |
| **D-AI-VERIFY-01** | AI 审查/汇报无核实纪律 → 误报入账 | P1 | [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md) + AGENTS/BOUNDARIES/Playbook 挂链 | **Done**（轮次 20） |
| **D-MAINT-01** | 远程 dependabot 陈旧分支（实测 **39**，**9** 含 `src-tauri`） | P2 | `gh api` 列表 + 批量 `git push origin --delete` | **OPEN** |
| **D-DOC-EN-01** | `creator-docs-en/security/KNOWN_VULNERABILITIES.md` 扫描日期滞后中文 | P2 | 对齐 `creator-docs/security/` 日期与命中条数 | **Done**（Wave 1 · 2026-06-25） |
| **D-ORDER-05** | `desktop-tauri/src/lib.rs` L203 仍写 `src-tauri/src/api/` | P2 | 改注释为 `distros/desktop-tauri/src/api/`；评估移出 stale-path 豁免 | **Done**（Wave 1） |
| **D-ORDER-06** | `distributions/vscode/out/` 与 `distros/` 命名并存 | P3 | gitignore 或删除构建产物 | **Done**（根 `.gitignore` 已含 `distributions/`） |
| **D-AI-VERIFY-02** | AGENTS 测试段链 `AI_VERIFICATION_PROTOCOL` + `check:rust` vs `check:release` doctest | P2 | AGENTS §测试体系 | **Done**（Wave 1） |
| **K-CI-01** | GitHub CI main 红：doctest 漂移 | **P0** | 修 doctest；`cargo test --workspace` 绿 | **Done**（Wave 0 · doctest 三处） |
| **D-READ-05** | `backend_registry.rs` 拆 `directory_slots` | P2 | 零语义变更 | **Done**（Wave 4 · `directory_slots_impl.rs`） |
| **D-PORT-02** | `PluginBackendRegistryPort` 拆窄 trait | P1 | `MemoryBackendPort` phase 1 | **Partial**（`memory_backend_port.rs`） |
| **D-SLOT-01** | BuiltinV1/V2 选择收到 resolver | P2 | 依赖 D-PORT-02 后续 | **Observe** |
| **D-TRAIT-01** | 单实现 trait 合并 | P3 | 仅明显 DI 噪音 | **Observe** |

---

## §1.5 供应链安全（Supply Chain · 2026-06-24）

**策略 SSOT**：[`creator-docs/security/SUPPLY_CHAIN.md`](../creator-docs/security/SUPPLY_CHAIN.md)

### 基线（已落地 · 非债）

| 护栏 | 说明 |
|------|------|
| `cargo audit` 0.22.1 | dimension5 + `ci.yml` + `cargo-audit-lockfile.yml` 三层硬门禁 |
| `Cargo.lock` ratchet | dimension5 禁止 `sqlx-mysql` / `rsa` |
| `KNOWN_VULNERABILITIES.md` | 漏洞级 SSOT；`Cargo.lock` PR 须滚动日期 |
| `deny.toml` + `oclive lint --deny` | 许可证允许表 · Apache-2.0 工作区 |
| 插件权限 A4 | manifest / runtime / 集成测三面一致 |
| SQL 迁移 checksum | 防迁移文件静默篡改 |

### 台账（OPEN / Observe / Deferred）

| ID | 项 | 优先级 | 状态 |
|----|-----|--------|------|
| **K-SUPPLY-01** | `cargo deny` 进 dimension5 / CI 硬门禁 | P1 | **Done**（轮次 19） |
| **K-SUPPLY-02** | Release SHA256SUMS | P1 | **Partial** — workflow 已入库；**维护者**首次 Release 挂 asset（见 [`.github/workflows/`](../../.github/workflows/) release 模板） |
| **K-SUPPLY-03** | 插件安装审源码提示 | P2 | **Done**（轮次 19） |
| **K-SUPPLY-04** | npm-audit 升格策略 | P2 | **OPEN** — 连续 2 周期高危 → 硬门禁或文档豁免（Observe 至 2026-07） |
| **K-SUPPLY-05** | deny 重复依赖 warn→deny | P2 | **OPEN** — `deny.toml` 已标注 K-SUPPLY-05；待 `cargo tree -d` 去重 |
| **K-SUPPLY-06** | 位级可重复构建（reproducible） | — | **Deferred** · 见 SECURITY_AUDIT_SCOPE 局限 |
| **K-SUPPLY-07** | SBOM（CycloneDX/SPDX） | — | **Deferred** · 政企/校企采购需求触发 |
| **K-SUPPLY-08** | crate 作者信誉 / 发布历史系统审计 | — | **Observe** · 无成熟自动化方案 |

**现在就能做（低成本）**：维持 dimension5 十三检绿 · `Cargo.lock` PR 更新 KNOWN_VULN · 发版前本地 `oclive lint --deny` · 校企仓要求组员 `npm ci && cargo build` 从源码跑通。

**下一工程动作（P1）**：K-SUPPLY-02 Release 哈希清单（与 `kernel_manifest` / bundled kernel 发版对齐）。

---

## §2 冻结 / registry（明确「不动」）

| ID | 项 | 解冻条件 |
|----|-----|----------|
| **dual_core** / **expert_routing** / **blueprint v3** | 实验管线 | **可选解冻 · 默认仍关**（蓝图 `dual_core.enabled` / 角色包 `expert_routing.json` 显式配置） |
| **D-READ-03** | `dual_pipeline` 表驱动 | 随 `dual_core` opt-in |
| **D-PORT-02** / **D-SLOT-01** | god-port collapse / 槽调度 | phase 1 memory 已拆；余组 Observe |
| **§3.1** | 纯 library API 对称化 | [`RFC_OCLIVE_KERNEL_LIBRARY.md`](./RFC_OCLIVE_KERNEL_LIBRARY.md) T0 |
| **模式 3** | 用户大纲演绎 / Mode 3 `send_message` 长对话 | 模式 2 playtest 扩展后另开计划 |
| ~~**模式 2**~~ | — | **已解冻** · [`MODE2_RFC.md`](./theater/MODE2_RFC.md) · `outline_rewrite` |

**Phase 5 结论（2026-06-25 更新）：** 朋友 cohort 产品门通过 → **模式 2 开工**；`dual_core` / `expert_routing` **机制可选、默认关**。详见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5.5。

---

## §3 观测台账（Observe · 无排期）

| ID | 项 | 说明 | 触发条件 | 下一动作 |
|----|-----|------|----------|----------|
| **D-PORT-03** | `BackendRegistry` UFCS 转发层 | D-PORT-02 已拆窄；collapse 等 remote policy RFC | 第二 remote 插件后端落地 or D-PORT-02 解冻 | 起草 remote policy RFC；评估 UFCS 层删除 |
| **D-READ-05** | `backend_registry` directory 子模块 | 机械拆文件；810 行可接受 | 文件 >1200 行 or 新 directory 后端类型 | 按子目录拆 `directory/` 模块 |
| **D-TRAIT-01** | 28 trait 单实现裁决 | 已裁决表保留；Repository 五件套合并等长期 | 外部贡献者要求合并 trait | 单 PR 合并一对 trait + 文档 |
| **D-POLICY-01** | Policy 三 trait 第二实现 | 等 remote policy RFC | remote policy RFC 合并 | 实现第二 `Policy*` 后端 |
| **D-ORPHAN-02** | `oclive_schema` 微型 crate | wasm 边界评估后再定 | wasm 宿主立项 | 评估合并进 `oclive_kernel_types` |
| **F4 / V2-remote** | remote 缺 env 静默回退 builtin | 已有 `startup_warnings`；矩阵诚实标 ⚠️ | 用户报告 silent fallback | 补 startup warning + 文档矩阵 |
| **K-PERF-10** | Chat chrome 懒加载 | **Partial** — overlay 已 lazy；chat chrome 仍 eager | 真人 playtest 归因首屏慢 **或** perf mark 超阈值 | 激活 chat chrome lazy PR |

### K-PERF-10 条件门（2026-06-18）

| 信号 | 结果 | 处置 |
|------|------|------|
| 工程代理 15s 通过率 | **100%** | 不激活 chat chrome lazy |
| 首屏 perf mark（[`PLAYTEST_MATRIX.md`](./theater/PLAYTEST_MATRIX.md) §性能） | 无真人失败数据 | 维持 **Partial / Deferred** |
| 真人 <60% 且归因首屏慢 | 未发生 | 待 P0-STRANGER 后复评 |

**结论：** K-PERF-10 **不启动**；待真人测试若首屏 perf 失败再激活。

---

## §4 长期 Deferred（战略 · 不阻塞当前）

| ID | 项 | 说明 |
|----|-----|------|
| **K-PERF-15** | 记忆候选池语义变更 | 产品确认召回语义 |
| **V-FUSED-01** | 多 `slot_registry` 实例融合 | Phase 3 |
| **§3.5–3.7** | 多模态 / 参考硬件 / Edge OTA | 路线图 |
| **§5.3** | 插件市场 UGC | 路线图 |
| **V-LORA-WORKSHOP-01** | 创作者微调工坊（T0–T3）+ `slot.lora.apply` 运行时 | 三发行版 smoke 后；愿景 [VISION_ROADMAP_MONTHLY.md](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)「微调工坊」；冻结期内仅 T0 契约 + T1 原型 |
| **D-OPUS-05 Phase 2** | re-export import 清零 | ratchet ≤76 只降不升 |
| **K-SUPPLY-06** | 位级可重复构建 | 内核 `kernel-v0.x` tag 稳定 + 专用 CI 镜像 |
| **K-SUPPLY-07** | SBOM 导出 | 校企/商业客户采购或合规要求 |

---

## §5 历史归档

Done 项（K-PERF-01~26、D-READ-01/02/04、K-ROBUST-01~03、Opus 4.8 Wave 0–4、Fable 5 M0–M4、K-DOC-15/16 等）见：

- [RECURRING_OPTIMIZATION_PLAYBOOK.md §8](./RECURRING_OPTIMIZATION_PLAYBOOK.md) 巡检日志
- [CHANGELOG.md](../CHANGELOG.md) `[0.4.0]` · `[Unreleased]`
- git log `handoff/` · `kernel/crates/oclive_kernel_host`

### 轮次 16 Done（2026-06-18）

| ID | 项 | 说明 |
|----|-----|------|
| **T-LAYER-16** | Theater 测迁出 domain | `theater_director_resolver` → `distros/desktop-tauri/tests/theater_director_resolver.rs` |
| **T-DOC-TD-01** | `theater_director` 文档扫尾 | DISTRO / ARCHITECTURE / NAMING / ROADMAP §7 / IA 头注 / domain README |
| **T-MINIMAL-TD-01** | minimal 插件自包含 | `examples/directory-plugin-theater-director-minimal/prompts/` 本地 `buildTheaterPrompt` |
| **T-CI-DRIFT-01** | prompt drift 门禁 | `dimension5-acceptance.mjs` + `test:theater:smoke` 双挂 |

### 轮次 17 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| **D-DOCDRIFT-01** | monorepo 后文档路径机械迁移 | `migrate-doc-paths.mjs` / `fix-remaining-doc-paths.mjs`；206 文件；`check-stale-paths` 硬门禁 |
| **D-SCRIPT-02** | `check-stale-paths.mjs` 扩范围 | dimension5 十一检 |
| **D-ORPHAN-04** | 删 `kernel/crates/models/` 空目录 | workspace 无引用 |

### 轮次 18 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| **O-1** | plugin-bridge 资产内核化 | `kernel/crates/oclive_kernel_host/assets/plugin-bridge.iife.js`；删 desktop-tauri 副本 |
| **O-2** | expert 孤儿前端清理 | 10 文件删；Tauri expert API / validation / dual_core 链保留 |
| **D-DOC-RELOC-01** | 文档名实归位 | `VSCODE_DISTRIBUTION` → `handoff/vscode/`；`USER_GUIDE` → `handoff/studio/`；`MUMU_UI_ACCEPTANCE` → `handoff/distros/` |

### 轮次 19 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| **K-SUPPLY-01** | `cargo deny` 硬门禁 | dimension5 第十二检 · `ci.yml` dimension5 job 安装 cargo-deny |
| **K-SUPPLY-02** | Release SHA256 | `generate-sha256sums.mjs` · `release-kernel-checksums.yml` · bundle 钩子 |
| **K-SUPPLY-03** | 插件审源码 toast | `installPath` DTO · 市场/git/zip · CLI · i18n |
| **K-SUPPLY-DOC-01** | 供应链策略 SSOT | `creator-docs/security/SUPPLY_CHAIN.md` + 本文件 §1.5 |

轮次 1–15 明细表已从本文件移除以降低噪音；需要历史格查 git `handoff/TECHNICAL_DEBT_INVENTORY.md` @ 2026-06-15。

---

## 速查坐标

| 用途 | 路径 |
|------|------|
| 编排 SSOT | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` |
| 槽态矩阵 | [SLOT_BACKEND_REALITY_MATRIX.md](./SLOT_BACKEND_REALITY_MATRIX.md) |
| Theater 验收 | [PLAYTEST_MATRIX.md](./theater/PLAYTEST_MATRIX.md) |
| Theater 模式 2 解冻 | [MODE2_UNFREEZE.md](./theater/MODE2_UNFREEZE.md) |
| 分层 ratchet | `handoff/LAYERING_BASELINE.json` |
| Theater director 集成测 | `distros/desktop-tauri/tests/theater_director_resolver.rs` |
| 供应链策略 | [SUPPLY_CHAIN.md](../creator-docs/security/SUPPLY_CHAIN.md) |
