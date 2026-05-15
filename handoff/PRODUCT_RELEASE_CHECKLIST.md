# 产品发版勾选表（P0 子集）

**用途**：发版会议或维护者自检时**只过本表**；权威缺口仍以 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A** 为准。详细说明与「硬骨头」排期见 [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md)。

**下一阶段（工程）**：**A1.1a**（HTTP 进程重启）与 **A1.2**（`invoke` 宿主热路径 9 条 `*_impl`）已入库可回归；**未勾主项**仍以 **A1.1b（GUI / 安装器 / 真 IPC）**、**A2.2 / A2.3 / A4.2** 等为主，按 [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) **§四** 拆独立 issue。

**与 CI**：本表**不替代** CI。本地建议顺序：`npm run test:unit` → `npm run check:release` →（可选）与 CI 相同的 OOCP / 姊妹仓检查；详见 [CONTRIBUTING.md](../CONTRIBUTING.md)「测试要求」「CI 对齐」。

---

## 闸门与记录

- [ ] **`npm run check:release`** 已通过（含全量 `cargo test`）
- [ ] **`npm run test:unit`** 已通过（**未**包含在 `check:release` 内；CI `frontend` job 会跑）
- [ ] **`CHANGELOG.md` / `CHANGELOG.en.md`** 已写入本版本用户可见条目（双语同步）
- [ ] **版本号**已对齐：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`（若本次发版 bump）

---

## 回归与手工（链到既有清单，不重复维护用例）

- [ ] [mumu 模块发版前验收清单](../creator-docs/guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md)
- [ ] [Plugin Manager V2 + 复杂情感回归](../creator-docs/guides/REGRESSION_COMPLEX_EMOTION_QA.md)
- [ ] [角色包导入 — 手工测试清单](../roles/TESTING_ROLE_PACK_IMPORT.md)（若本版 touched 导入 / manifest）
- [ ] [高风险能力验收（演示向）](./PLUGIN_HIGH_RISK_ACCEPTANCE.md)（目录插件 / MCP / 网络授权路径能演示）

---

## 对外说明（轻量 P0）

- [ ] [对外兼容一页表](../creator-docs/COMPATIBILITY.md)（主程序 / 编写器 / 启动器 / `min_runtime_version`）已核对是否需要更新
- [ ] 根 [README.md](../README.md)「早期采用者 / 已知限制」段落仍与当前行为一致

---

## §A 映射（主清单 P0 — 发版前能勾则勾）

下列与 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A1–A5** 对应；未实现项保持 **不勾选**，但应在 CHANGELOG 或 README「已知限制」中诚实写出。

### A1 测试与质量

- [x] **A1.1a（子项）** **HTTP `--api` 进程重启烟测**（[`scripts/e2e-core-api-restart.mjs`](../scripts/e2e-core-api-restart.mjs) + CI `oocp-test-suite`）— 见 [`OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md)  
- [ ] **A1.1b（主项）** 安装包 / **桌面 GUI** / 切角等完整自动化 — Playwright 等另立项  
- [x] **A1.2** **`invoke` 宿主热路径（9 条 `*_impl`）**：[`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md) + [`invoke_hotpath_matrix.rs`](../src-tauri/tests/invoke_hotpath_matrix.rs)；**全命令 golden / Playwright 真 IPC** 仍属 A1.1b 或后续增强  
- [x] **A1.3** 本地与 CI 闸门习惯已文档化（见 CONTRIBUTING + 本表「闸门」）
- [x] **A1.4** 回归清单通过上节链接聚合（本表）

### A2 首装与环境

- [ ] **A2.1** 首装失败路径文案 + i18n（子集见 [ERROR_CODES §1.5](../creator-docs/getting-started/ERROR_CODES.md) 增补说明）
- [ ] **A2.2** 可选环境自检
- [ ] **A2.3** 离线/弱网

### A3 崩溃与诊断

- [ ] **A3.1** Sentry 等（若启用：默认、隐私、可关闭与 README 一致）
- [ ] **A3.2** 用户可见错误走 `[CODE]` + 前端映射（持续扫尾）

### A4 插件与安全

- [ ] **A4.1** 高风险能力可演示（见 [PLUGIN_HIGH_RISK_ACCEPTANCE.md](./PLUGIN_HIGH_RISK_ACCEPTANCE.md)）
- [ ] **A4.2** manifest / 校验 / 运行时三面一致盘点

### A5 版本与兼容

- [ ] **A5.1** 对外兼容表已随版本审阅（[COMPATIBILITY.md](../creator-docs/COMPATIBILITY.md)）
- [x] **A5.2** CHANGELOG 双语纪律（CONTRIBUTING 已要求；发版勾选本表「闸门与记录」）

---

## 相关链接

- [PROJECT_OVERVIEW.md §8 发版极简清单](../creator-docs/getting-started/PROJECT_OVERVIEW.md)
- [DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
