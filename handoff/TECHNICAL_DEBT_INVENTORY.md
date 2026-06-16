# Technical debt inventory

**Last updated:** 2026-06-15 (轮次 15 · M5 收束 + Theater P0 工程验收)

**Product freeze (Theater v0):** Active until **5 人真人陌生人** ≥60% 通过。工程代理 100% **不替代**产品门槛。见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §4.8。

**综合评分：** A− · 基线 dimension5 九检 PASS · `oclive_kernel_host --lib` **193** 绿 · layering ratchet 3/1 未涨

**下一动作：** **P0-STRANGER** — 维护者带 5 名零文档测试者；验收标准见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §4.8

**Verification (2026-06-15 轮次 15):** `node scripts/dimension5-acceptance.mjs --ci`；`npm run test:theater:smoke`；`npm run tauri:build:theater`（MSI + NSIS 绿）；`cargo test -p oclive_kernel_host --lib`。

---

## §1 活跃台账（OPEN · 开工清单）

| ID | 项 | 优先级 | 解冻/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **P0-STRANGER** | Theater 5 人真人陌生人测试 | **P0** | ≥60% 通过 · [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §4.8 | **OPEN** — 工程代理 100%；真人表待填 |
| **K-DOC-15** | 契约层 `//` 注释英文化 | P1 | `role.rs` / `dto.rs` / `role_pack_config.rs` / `contracts` | **Done**（轮次 15 复核无中文 `//`） |
| **K-DOC-16** | `prompt_builder` 中文范围声明 | P1 | `mod.rs` + `sections.rs` 文件头 | **Done** |
| **K-DOC-17** | 注释英文化 batch 3 | P1 | `slot_runner.rs` · `kernel_strategy.rs` 等 | **OPEN** — good-first-issue 候选 |
| **V4-ONBOARD-03** | good-second-issue 策展 + GitHub issues | P1 | [GOOD_FIRST_ISSUES.md](./GOOD_FIRST_ISSUES.md) #11–13 | **Done** |
| **V-VSCODE-PERF-05** | VS Code F5 / `.vsix` 实机 | P1 | 姊妹仓人工排期 | **OPEN** |
| **K-CONTRACT-WIRING-01** | `extra_sections` 生产接线 | P2 | 首个外部插件作者 or Phase 5 通过后 | **OPEN** |

---

## §2 冻结 / registry（明确「不动」）

| ID | 项 | 解冻条件 |
|----|-----|----------|
| **dual_core** / **expert_routing** / **blueprint v3** | 实验管线 | 重大发版决策；默认关 |
| **D-READ-03** | `dual_pipeline` 表驱动 | 随 `dual_core` |
| **D-PORT-02** / **D-SLOT-01** | god-port collapse / 槽调度 | 外部贡献者 or 第二成熟插件后端 |
| **§3.1** | 纯 library API 对称化 | 第二宿主强需求 + RFC |

**Phase 5 结论（2026-06-15）：** 真人门槛未过 → **维持冻结**；`dual_core` / `expert_routing` / 编排扩展 **不开工**。详见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5.5。

---

## §3 观测台账（Observe · 无排期）

| ID | 项 | 说明 |
|----|-----|------|
| **D-PORT-03** | `BackendRegistry` UFCS 转发层 | D-PORT-02 已拆窄；collapse 等 remote policy RFC |
| **D-READ-05** | `backend_registry` directory 子模块 | 机械拆文件；810 行可接受 |
| **D-TRAIT-01** | 28 trait 单实现裁决 | 已裁决表保留；Repository 五件套合并等长期 |
| **D-POLICY-01** | Policy 三 trait 第二实现 | 等 remote policy RFC |
| **D-ORPHAN-02** | `oclive_schema` 微型 crate | wasm 边界评估后再定 |
| **F4 / V2-remote** | remote 缺 env 静默回退 builtin | 已有 `startup_warnings`；矩阵诚实标 ⚠️ |
| **K-PERF-10** | Chat chrome 懒加载 | **Partial** — overlay 已 lazy；chat chrome 仍 eager；**条件未触发**（见下） |

### K-PERF-10 条件门（2026-06-15）

| 信号 | 结果 | 处置 |
|------|------|------|
| 工程代理 15s 通过率 | **100%** | 不激活 chat chrome lazy |
| 首屏 perf mark（THEATER_15S_ACCEPTANCE §性能） | 无真人失败数据 | 维持 **Partial / Deferred** |
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

Done 项（K-PERF-01~26、D-READ-01/02/04、K-ROBUST-01~03、Opus 4.8 Wave 0–4、Fable 5 M0–M4 等）见：

- [RECURRING_OPTIMIZATION_PLAYBOOK.md §8](./RECURRING_OPTIMIZATION_PLAYBOOK.md) 巡检日志
- [CHANGELOG.md](../CHANGELOG.md) `[0.4.0]`
- git log `handoff/` · `crates/oclive_kernel_host`

轮次 1–14 明细表已从本文件移除以降低噪音；需要历史格查 git `handoff/TECHNICAL_DEBT_INVENTORY.md` @ 2026-06-14。

---

## 速查坐标

| 用途 | 路径 |
|------|------|
| 编排 SSOT | `crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` |
| 槽态矩阵 | [SLOT_BACKEND_REALITY_MATRIX.md](./SLOT_BACKEND_REALITY_MATRIX.md) |
| Theater 验收 | [THEATER_15S_ACCEPTANCE.md](./theater/THEATER_15S_ACCEPTANCE.md) |
| 分层 ratchet | `handoff/LAYERING_BASELINE.json` |
