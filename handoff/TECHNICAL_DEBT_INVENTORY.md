# Technical debt inventory

**Last updated:** 2026-06-24 (轮次 18 · 巡检优化收尾)

**Product freeze (Theater v0):** Active until **5 人真人陌生人** ≥60% 通过。工程代理 100% **不替代**产品门槛。见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §4.8 · 解冻 checklist [`theater/MODE2_UNFREEZE.md`](./theater/MODE2_UNFREEZE.md)。

**综合评分：** A− · 基线 dimension5 **十一检** PASS · `oclive_kernel_host` 编译期不再依赖 `distros/desktop-tauri/` · expert 孤儿前端已清 · 三份名实不符文档已归位 `handoff/`

**下一动作：** **P0-STRANGER** — 维护者带 5 名零文档测试者；验收标准见 [`theater/PLAYTEST_MATRIX.md`](./theater/PLAYTEST_MATRIX.md)

**Verification (2026-06-24 轮次 18):** `node scripts/dimension5-acceptance.mjs --ci`；`cargo build -p oclive_kernel_host`；`npm run test:unit`；`npm run build -w @oclive/chat-pro`；`node scripts/check-stale-paths.mjs`。

---

## §1 活跃台账（OPEN · 开工清单）

| ID | 项 | 优先级 | 解冻/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **P0-STRANGER** | Theater 5 人真人陌生人测试 | **P0** | ≥60% 通过 · [`theater/PLAYTEST_MATRIX.md`](./theater/PLAYTEST_MATRIX.md) | **OPEN** — 工程代理 100%；真人表待填 |
| **K-DOC-17** | 注释英文化 batch 3 | P1 | `slot_runner.rs` · `kernel_strategy.rs` 等 | **Done**（轮次 16 复核：上述文件已为英文 `//!`/`///`） |
| **V-VSCODE-PERF-05** | VS Code F5 / `.vsix` 实机 | P1 | 姊妹仓 `oclive-vscode` 人工排期 | **OPEN**（cross-repo） |
| **K-CONTRACT-WIRING-01** | `extra_sections` 生产接线 | P2 | 首个外部插件作者 or Phase 5 通过后 | **OPEN** |
| **D-DOCDRIFT-01** | 重组后 normative 文档路径漂移（旧布局引用） | P0 | `check-stale-paths` 硬门禁绿 + `migrate-doc-paths` 路径存在性全过 | **Done**（轮次 17） |
| **D-SCRIPT-02** | `check-stale-paths.mjs` 误报/漏报（反例说明与行内路径） | P1 | 扩范围 + 修 pattern + 挂 dimension5 | **Done**（轮次 17） |
| **D-ORPHAN-04** | 残留空目录 `kernel/crates/models/` | P2 | 目录删除 + workspace 无引用 | **Done**（轮次 17） |
| **O-1** | `oclive_kernel_host` 编译期 `include_str!` 耦合 `distros/desktop-tauri/assets/plugin-bridge.iife.js` | P1 | 资产迁入 `kernel/crates/oclive_kernel_host/assets/` + copy 脚本改指向 | **Done**（轮次 18） |
| **O-2** | expert 孤儿前端（Vue/lib/test/i18n/API re-export，零 import） | P2 | 删除 + `role.ts`/locales 同步 + stale 文档措辞 | **Done**（轮次 18） |
| **D-DOC-RELOC-01** | 三份名实不符文档仍在 `creator-docs/`（VS Code 契约 / Studio 指南 / mumu 验收） | P2 | 物理迁至 `handoff/{vscode,studio,distros}/` + 原位 stub + 入链更新 | **Done**（轮次 18） |

---

## §2 冻结 / registry（明确「不动」）

| ID | 项 | 解冻条件 |
|----|-----|----------|
| **dual_core** / **expert_routing** / **blueprint v3** | 实验管线 | 重大发版决策；默认关 |
| **D-READ-03** | `dual_pipeline` 表驱动 | 随 `dual_core` |
| **D-PORT-02** / **D-SLOT-01** | god-port collapse / 槽调度 | 外部贡献者 or 第二成熟插件后端 |
| **§3.1** | 纯 library API 对称化 | 第二宿主强需求 + RFC |
| **模式 2 / 3** | 用户大纲演绎 / Mode 3 | Track B ≥60% + [`MODE2_UNFREEZE.md`](./theater/MODE2_UNFREEZE.md) 签字 |

**Phase 5 结论（2026-06-15）：** 真人门槛未过 → **维持冻结**；`dual_core` / `expert_routing` / 编排扩展 **不开工**。轮次 18 仅清 dead code / 编译耦合 / 文档名实，**未动**上述冻结项。详见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5.5。

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
