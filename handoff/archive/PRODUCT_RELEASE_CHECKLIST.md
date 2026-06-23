# 产品发版勾选表（P0 子集）

**用途**：发版会议或维护者自检时**只过本表**；权威缺口仍以 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A** 为准。详细说明与「硬骨头」排期见 [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md)。

**下一阶段（工程）**：**A1** 可 CI 子集已收口；**A3**（崩溃与诊断 / 用户可见错误 JSON 与 i18n）已按 `handoff/A3_CLOSURE_SUMMARY.md` 落实；**A5** 兼容表与 CHANGELOG 纪律已按 `handoff/A5_CLOSURE_SUMMARY.md` 收口；未勾主项以 **A1.1c**、**A4.2**、**原生安装包 GUI E2E** 等为主，按 [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) **§四** 拆独立 issue。

**与 CI**：本表**不替代** CI。本地建议顺序：`npm run test:unit` → `npm run build && npm run test:e2e:preview`（与 CI `frontend` 对齐）→ `npm run check:release` →（可选）与 CI 相同的 OOCP / 姊妹仓检查；详见 [CONTRIBUTING.md](../CONTRIBUTING.md)「测试要求」「CI 对齐」。

---

## 闸门与记录

**核对日期**：2026-05-20 · **目标版本**：**v0.2.0**（workspace 版本号已与 [RELEASE_VERSIONING.md](../creator-docs/development/RELEASE_VERSIONING.md) 对齐）

- [x] **`npm run test:unit`** 已通过（25 tests，2026-05-20 复验）
- [x] **`npm run build`** 已通过（2026-05-20 复验）
- [x] **`cargo clippy --workspace -D warnings`** 已通过（2026-05-20 工程扫尾复验）
- [x] **`cargo test --workspace --lib`** 已通过（128 tests，2026-05-20 复验）
- [ ] **`npm run check:release`** — ⚠️ 发版当日在维护机执行（含全量 `cargo test`；Windows 集成测以 CI Ubuntu 为准）
- [x] **`CHANGELOG.md` / `CHANGELOG.en.md`** — **`[0.2.0] - 2026-05-22`** 条目已整理；发版日仅做最终条目追加与 tag
- [x] **版本号** — 桌面 **0.2.0**、CLI **0.1.0**、**`oclive_kernel_runtime` 0.2.0** 与 `package.json` / 各 `Cargo.toml` 一致（发版日 bump 时再核对 `tauri.conf.json`）
- [x] **姊妹仓编写器** — `HOST_RUNTIME_VERSION` **0.2.0** 与主仓 `src-tauri/Cargo.toml` 一致；保存时保留蓝图 `includes` / `groups` / `expert_overlay` / `runtime_config`（2026-05-20）
- [x] **对外文档** — README / CREATOR_WORKFLOW 以 **编写器 + 运行时** 为准；启动器标注已退役（2026-05-20）
- [x] **`npm run contract:json-keys`**（oclive-pack-editor）— 与主仓 `json_keys.rs` 一致（2026-05-20 复验）

---

## 回归与手工（链到既有清单，不重复维护用例）

**状态说明**：下列为发版会议手工项；未勾选表示**待发版日执行**，不阻塞工程扫尾合入。

- [ ] [mumu 模块发版前验收清单](../distros/MUMU_UI_ACCEPTANCE_CHECKLIST.md) — ⚠️ 发版前手工
- [ ] [Plugin Manager V2 + 复杂情感回归](../creator-docs/guides/REGRESSION_COMPLEX_EMOTION_QA.md) — ⚠️ 发版前手工
- [ ] [角色包导入 — 手工测试清单](../roles/TESTING_ROLE_PACK_IMPORT.md)（若本版 touched 导入 / manifest）— ⚠️ 按需
- [ ] [高风险能力验收（演示向）](./PLUGIN_HIGH_RISK_ACCEPTANCE.md) — ⚠️ 演示向；A4.1 已 CI/文档收口，发版日再勾

---

## 对外说明（轻量 P0）

- [ ] [对外兼容一页表](../creator-docs/COMPATIBILITY.md) — ⚠️ 发版日核对是否需要更新（A5.1 基线已入库）
- [ ] 根 [README.md](../README.md)「早期采用者 / 已知限制」— ⚠️ 发版日快速复读；当前与 A3/A5 收口一致

---

## §A 映射（主清单 P0 — 发版前能勾则勾）

下列与 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A1–A5** 对应；未实现项保持 **不勾选**，但应在 CHANGELOG 或 README「已知限制」中诚实写出。

### A1 测试与质量

- [x] **A1.1a（子项）** **HTTP `--api` 进程重启烟测**（[`scripts/e2e-core-api-restart.mjs`](../scripts/e2e-core-api-restart.mjs) + CI `oocp-test-suite`）— 见 [`OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md)  
- [x] **A1.1b** **`vite preview` + Playwright 首屏**：[`e2e/preview-shell.spec.ts`](../e2e/preview-shell.spec.ts)，`npm run test:e2e:preview`；**CI：Ubuntu `frontend`**（`PW_TEST_USE_EXTERNAL` + 后台 preview；Windows `frontend` 不跑本项）  
- [ ] **A1.1c（延伸）** **安装包 / Tauri 原生窗 / 全屋 GUI E2E**：WebDriver 或发行流水线；**不挡 A1 可 CI 子集收口**  
- [x] **A1.2** **`invoke` 宿主热路径（11 条 `*_impl` 烟测，含蓝图槽 API）**：[`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md) + [`invoke_hotpath_matrix.rs`](../src-tauri/tests/invoke_hotpath_matrix.rs)；**golden / 全 handler** 仍属后续增强  
- [x] **A1.3** 本地与 CI 闸门习惯已文档化（见 CONTRIBUTING + 本表「闸门」）
- [x] **A1.4** 回归清单通过上节链接聚合（本表）

### A2 首装与环境

- [x] **A2.1** 首装与 invoke 可见路径：`[CODE]` + `apiErrors` 覆盖（含事务扩展码、`ROLE_RUNTIME_NOT_READY`、`STARTUP_HEALTH_FAILED`、`PLUGIN_BACKENDS_DIRECTORY_SLOT`、Remote LLM 文案）；**补丁**：`invoke` 失败载荷已统一为 **内核 `KernelErrorBody` JSON 单行**（与 HTTP `error` 同形），见 [`A2_KERNEL_JSON_ERROR_PATCH.md`](./A2_KERNEL_JSON_ERROR_PATCH.md)
- [x] **A2.2** 可选环境自检（设置 → 常规 →「环境自检」+ `run_environment_diagnostics`：Ollama `/api/tags`、roles 根可读、app_data 写探针）
- [x] **A2.3** 离线/弱网（插件索引失败 → 缓存 + 工作台 i18n + **顶栏下全局提示条**；[ERROR_CODES §1.6](../creator-docs/getting-started/ERROR_CODES.md)；全产品 Remote 统一状态机仍可迭代）

### A3 崩溃与诊断

- [x] **A3.1** Sentry 等（若启用：默认、隐私、可关闭与 README 一致）— 见 [`A3_CLOSURE_SUMMARY.md`](./A3_CLOSURE_SUMMARY.md)
- [x] **A3.2** 用户可见错误走 **`KernelErrorBody` JSON `code`** + 前端 **`apiErrors`** 映射（含 `ApiError` 与 `UNKNOWN_WITH_CODE` 兜底；`[CODE]` 仅 legacy）— 同上

### A4 插件与安全

- [x] **A4.1** 高风险能力可演示（MCP http/stdio、目录插件 `process` spawn；见 [PLUGIN_HIGH_RISK_ACCEPTANCE.md](./PLUGIN_HIGH_RISK_ACCEPTANCE.md)、[A4_CLOSURE_SUMMARY.md](./A4_CLOSURE_SUMMARY.md)）
- [x] **A4.2** manifest / 校验 / 运行时三面一致（`plugin_permissions` + `high_risk_grants` 规范键 + 集成测；见 [A4_CLOSURE_SUMMARY.md](./A4_CLOSURE_SUMMARY.md)）

### A5 版本与兼容

- [x] **A5.1** 对外兼容表基线已入库（[COMPATIBILITY.md](../creator-docs/COMPATIBILITY.md)、[`A5_CLOSURE_SUMMARY.md`](./A5_CLOSURE_SUMMARY.md)）；**每次发版**仍须核对表中快照与姊妹仓是否需要更新
- [x] **A5.2** CHANGELOG 双语纪律（CONTRIBUTING 已要求；发版勾选本表「闸门与记录」）

---

## 相关链接

- [PROJECT_OVERVIEW.md §8 发版极简清单](../creator-docs/getting-started/PROJECT_OVERVIEW.md)
- [DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
