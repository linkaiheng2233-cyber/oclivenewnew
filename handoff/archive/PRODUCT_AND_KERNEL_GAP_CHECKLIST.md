# oclive：产品级 + 纯净内核/平台目标 — 可优化清单

本文合并两类差距：

1. **产品级**：陌生人能装、能懂、出事少、你能扛住反馈（桌面宿主首发）。
2. **纯净内核与平台目标**：机器人侧「灵魂/陪伴」、嵌入式覆盖、AI 软硬件基座（与 [KERNEL_AND_MODULES_ARCHITECTURE.md](../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)、[OCLIVE_CLI_GUIDE.md](../creator-docs/cli/OCLIVE_CLI_GUIDE.md) 对齐）。

**优先级约定**

| 标记 | 含义 |
|------|------|
| **P0** | 建议作为「公开发布」硬门槛；未做则首发风险高 |
| **P1** | 不挡首发，但强烈影响口碑与维护成本 |
| **P2** | 中长期平台/内核叙事；可分阶段排期 |

---

## A. 产品级（桌面宿主与首发）

### A1. 测试与质量闸门（P0）

- [x] **核心路径自动化（可 CI 子集）**：**HTTP `--api` 进程重启**（[`scripts/e2e-core-api-restart.mjs`](../scripts/e2e-core-api-restart.mjs) + CI `oocp-test-suite`）；**`vite build:e2e` + `vite preview` + Playwright** 首屏与关键路径（[`e2e/preview-shell.spec.ts`](../e2e/preview-shell.spec.ts)、[`e2e/send-message.spec.ts`](../e2e/send-message.spec.ts)、[`e2e/switch-role.spec.ts`](../e2e/switch-role.spec.ts)、[`e2e/install-plugin.spec.ts`](../e2e/install-plugin.spec.ts)；`e2e-mock/` invoke 桩，CI **`frontend`** job）。**安装包 / Tauri 原生窗 / WebDriver 全屋** 另立项，不记入本条。
- [ ] **A1.1c（基础建设已启动）**：[`e2e/tauri-native.spec.ts`](../e2e/tauri-native.spec.ts) + [`scripts/e2e-tauri-native-ci.sh`](../scripts/e2e-tauri-native-ci.sh) + CI **`e2e-tauri`** job（`continue-on-error: true`，Ubuntu + `tauri-driver` 最小烟测：窗口标题 + `.left-pane`）。**全屋 GUI / 安装包签名** 仍延后。
- [ ] **核心路径自动化（原生安装包与 Tauri 窗 · 全屋）**：签名分发、多 OS 安装器、真 `invoke` GUI 全链；依赖发行流水线，**不作为当前 A1 必勾项**。
- [x] **`invoke` 宿主热路径集成烟测**：高流量命令经 [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md) 对齐；[`invoke_hotpath_matrix.rs`](../src-tauri/tests/invoke_hotpath_matrix.rs) 串联 **9** 条 `*_impl`。**全 handler golden / 真 IPC** 仍后续增强。
- [x] **发版前闸门与 CI 对齐**：[`CONTRIBUTING.md`](../CONTRIBUTING.md) 已列 **`npm run check:release`**、**`npm run test:unit`**、**`npm run test:e2e:preview`**（Playwright）及 **OOCP** / **`rust`** / **`frontend`** job 差异；CI 见 [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)。
- [x] **回归清单版本化**：关键手工场景由 [PRODUCT_RELEASE_CHECKLIST.md](./PRODUCT_RELEASE_CHECKLIST.md) **「回归与手工」** 节链到既有 guides（mumu / 复杂情感 / 角色包导入 / 高风险能力）；发版按版本勾选，不在此重复维护用例树。

### A2. 首装、环境与可恢复性（P0）

- [x] **首装失败路径文案（invoke 全码表）**：`apiErrors` 与 `toFriendlyErrorMessage` 覆盖内核 `AppError`、扩展事务码、**`ROLE_RUNTIME_NOT_READY` / `STARTUP_HEALTH_FAILED`**、插件槽 **`PLUGIN_BACKENDS_DIRECTORY_SLOT`**；首装自助仍见 [`ERROR_CODES` §1.5–1.6](../creator-docs/getting-started/ERROR_CODES.md)。**HTTP `--api`** 与 Tauri 失败载荷均为 **`KernelErrorBody` JSON**（`SCREAMING_SNAKE_CASE` `code`），见 [`KERNEL_ERROR_CODE_CONVENTION.md`](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)。
- [x] **可选环境自检**：设置 → 常规 →「环境自检」+ `run_environment_diagnostics`（Ollama、roles 根、app_data 可写）；失败见返回 `*Detail` 与 §1.5。
- [x] **离线/弱网（主路径 + 全局提示）**：社区索引失败 → 缓存 + 工作台 i18n + **`App.vue` 顶栏下全局条**（`uiStore.connectivityBanner`）；文档 [`ERROR_CODES` §1.6](../creator-docs/getting-started/ERROR_CODES.md)。**后续可增强**：Remote 插件/MCP 失败聚合、与 oclive-plugin-market 站统一「网络状态」组件。

### A3. 崩溃、遥测与隐私（P0 / P1）

- [x] **崩溃与诊断**：Sentry — 构建期 DSN、**设置页可退出**（`localStorage` **`oclive.telemetry.sentryOptOut`**）、`sendDefaultPii: false`、URL query 脱敏；与根 **README / README.en** 一致。结项说明见 [`A3_CLOSURE_SUMMARY.md`](./A3_CLOSURE_SUMMARY.md)。
- [x] **日志与用户可见错误**：`invoke` / 目录插件 **`ApiError`** 与内核同源 **JSON `code`**；前端 **`apiErrors`** + **`UNKNOWN_WITH_CODE`** 兜底；避免 UI 主路径依赖 `[CODE]`。

### A4. 插件与安全边界（P0）

- [x] **高风险能力（MCP + 目录 process + Remote network）**：显式授权文件、`HIGH_RISK_CAPABILITY_NOT_GRANTED`、Agent 调试面板 grant/revoke、CI 可 `OCLIVE_SKIP_HIGH_RISK_GRANTS`；Remote `network:*` 与 manifest `permissions` 已对齐（见 [`A4_CLOSURE_SUMMARY.md`](./A4_CLOSURE_SUMMARY.md)）。
- [x] **权限与 manifest 完全一致性**：`permissions` 字段 + `oclive_validation::plugin_permissions` + `high_risk_grants.json` 规范键；集成测 [`permission_three_way_consistency.rs`](../src-tauri/tests/permission_three_way_consistency.rs)；文档 [PLUGIN_V1 §权限规范](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)。

### A5. 版本、兼容与升级（P0）

- [x] **对外兼容一页表**：主程序版本 ↔ 编写器/启动器 ↔ `min_runtime_version` ↔ 角色包 schema；破坏性变更的迁移提示。（基线见 [`creator-docs/COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md)、[`handoff/A5_CLOSURE_SUMMARY.md`](./A5_CLOSURE_SUMMARY.md)；**每次发版**仍须按 [`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md) 核对是否更新表内快照。）
- [x] **CHANGELOG 纪律**：用户可感知的变更必记；插件作者可据此适配。（`CONTRIBUTING` 已要求双语；发版见 [`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md) 闸门。）

### A6. 国际化与文案（P1）

- [x] **界面语言切换后无残留中文**（在承诺范围内；长尾可声明例外）。Vitest 守卫 [`i18n_hardcoded_ui.spec.ts`](../src/__tests__/i18n_hardcoded_ui.spec.ts) 扫描 `src/**/*.vue|ts`（排除 `i18n/locales` 与注释）；角色扮演解析正则见 allowlist。
- [x] **creator-docs-en（创作者文档英文镜像）**：主干（总索引、插件契约、`guides/`、`LICENSE_POLICY`、FAQ/兼容等）已与 [DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) 及 [creator-docs-en/README.md](../creator-docs-en/README.md) 对拍收尾；**`creator-docs/roadmap/`** 等愿景与生态长文仍以中文为准；**更新纪律**（契约变更时同步英文或 CHANGELOG 声明）见英文 README 小节 [Documentation bilingual closure baseline](../creator-docs-en/README.md#documentation-bilingual-closure-baseline)。

### A7. 性能与资源（P1）

- [x] **低端机 / 冷启动 / 长会话**：与 [13_PERF_BASELINE_2026-04-01.md](./13_PERF_BASELINE_2026-04-01.md) 等基线对照，公开「已知限制」或数字底线。（**对外一页**：[`PERFORMANCE.md`](../creator-docs/getting-started/PERFORMANCE.md) / [`creator-docs-en/.../PERFORMANCE.md`](../creator-docs-en/getting-started/PERFORMANCE.md)，数值锚定 [`LIGHTWEIGHT_PROFILE.md`](../creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.7 与 `oclive bench` Schema。）

### A8. 无障碍与基础 UX（P1）

- [x] **高频路径键盘与焦点（插件管理切片）**：V1 目录列表 ↑/↓ 与 `tabindex`；添加/删除槽位弹窗打开聚焦首控件/取消钮；架构图节点 `select` 可 Tab 聚焦。（聊天发送等待办）
- [x] **长任务进度**：大包导入（`import_progress` + 文件名/序号）、市场索引同步（按钮 + `role=status`）、本地插件 zip 安装（按钮态）；环境自检保留 `envDiagLoading` 文案。

### A9. 支持与预期（P1）

- [x] **单一主入口**：Issue 模板、FAQ、社区/邮件任选其一写死；降低「不知道去哪问」。（**GitHub Issues** + Bug / Feature / Support 模板；README「支持」小节；首次响应预期见 README。）
- [x] **首发预期管理**：README 明确「早期采用者」范围与已知限制。

### A10. 法律与分发（P1）

- [x] **许可证与免责声明**：第三方模型、插件市场、用户数据落盘 — README/隐私摘要可扫一眼即答三问。（**[`DISCLAIMER.md`](../creator-docs/legal/DISCLAIMER.md)** / EN 镜像，链自 README 与 `SECURITY*`。）

---

## B. 纯净内核 / 机器人灵魂 / 嵌入式 / 平台

### B1. 叙事与边界（P0 / P2）

- [x] **「纯净内核」定义成文**：见 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)。
- [x] **「灵魂」交付单元**：角色包 + 有效 `plugin_backends` + 会话策略 — K3 **RobotSoulPack** profile（[ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) + `oclive pack validate --profile robot-soul` + `examples/robot-soul-minimal/`）。

### B2. 机器人 / 多模态 / 低延迟（P1 / P2）

- [ ] **多模态抽象**：语音流、视觉、触觉等若进产品 — 与现有文本回合编排的关系图与 MVP 边界（目录插件 vs 内核扩展）。
- [ ] **打断与半双工**：边听边说、唤醒打断与当前 `send_message` 模型的差距评估与 PoC。（**拓展基础已就位**：蓝图 `slot_registry` + 实验编排；预留说明见 [TECHNICAL_DEBT_INVENTORY.md §3.5](./TECHNICAL_DEBT_INVENTORY.md)）
- [ ] **多机器人 / 多用户隔离**：身份、密钥、记忆命名空间 — 若做 B 端或云边协同则升格为 P0。（**拓展基础已就位**：同上 §3.5）

### B3. 嵌入式：`kernel_server` vs `library`（P0 / P2）

- [x] **能力对称策略**：Monolith 仅 `kernel_server`；`library` 路径见 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) §5、[KERNEL_IMPLEMENTATION_PLAN.md](../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md) K4。（**拓展基础已就位**：`oclive_kernel_types` / `oclive_kernel_contracts` + `oclive init --project-type library`；预留说明 [TECHNICAL_DEBT_INVENTORY.md §3.1](./TECHNICAL_DEBT_INVENTORY.md)）
- [x] **脚手架 → 真内核接榫**：K1 [headless-kernel-minimal](../examples/headless-kernel-minimal/README.md)（`--api`）；K2 `oclive_kernel_runtime` / `oclive_kernel_server` + `oclive-cli --kernel-source`（见内核计划 **验收留痕**）。
- [x] **诚实范围表**：已写入 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) §6；随实机验证更新措辞。

### B4. 平台基座：工具链与开发者路径（P1 / P2）

- [x] **「平台开发者一条路径」**：[KERNEL_PLATFORM_DEVELOPER_PATH.md](../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)（中英）链到 PLUGIN_V1、SETTINGS_REFERENCE、OOCP、目录插件。
- [ ] **参考硬件或仿真靶子**：至少一类参考板或 docker-compose 侧车，降低硬件团队试错成本。（**拓展基础已就位**：CI ARM64 交叉编译 + `headless-api` 模板；预留说明 [TECHNICAL_DEBT_INVENTORY.md §3.6](./TECHNICAL_DEBT_INVENTORY.md)）
- [ ] **无头/边缘运维**：OTA、回滚、远程日志/健康检查 — 若承诺「平台」则需里程碑（**P2**，不阻塞 K1–K5）。（**拓展基础已就位**：`--api` HTTP + 侧车插件协议；预留说明 [TECHNICAL_DEBT_INVENTORY.md §3.7](./TECHNICAL_DEBT_INVENTORY.md)）

### B5. 姊妹仓与整机交付（P1）

- [x] **主仓 ↔ doll core / 交付包**：契约与交付说明见 [KERNEL_PLATFORM_DEVELOPER_PATH.md](../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md) 与 doll core `README.md`。

---

## C. 横切（两类目标共用）

### C1. 文档与索引（P1）

- [x] **PRODUCT_RELEASE_CHECKLIST**：仅 P0 子集可勾版（**骨架已建**：[`handoff/PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md)）；发版会议只过一张表。
- [x] **本清单与路线图互链**：`VISION_ROADMAP_MONTHLY.md`、`BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md` 已链至 §A / 分桶 / 发版勾选表。

### C2. 工程纪律（P0）

- [x] **Breaking 变更流程**：[`handoff/BREAKING_CHANGE_PROCESS.md`](./BREAKING_CHANGE_PROCESS.md)（识别 → 声明 → 审阅 → 迁移指南 → `oclive_validation` → 文档；PR/迁移模板；CONTRIBUTING 已链入）。
- [x] **Bus factor**：[`handoff/BUS_FACTOR_NOTES.md`](./BUS_FACTOR_NOTES.md)（编排、`PluginHost`、错误码、DB、复杂情感、Monolith、角色包、测试/CI 入口索引；文档索引与 AGENTS 已链入）。

---

## D. 建议执行顺序（2026-05-15：内核 K0–K5 已收口）

1. **内核里程碑** — [KERNEL_IMPLEMENTATION_PLAN.md](../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md)：**K0–K5（除 P2 OTA）** 已在计划与工程中收口；本地验收见该文档 **「验收留痕」**；持续 CI 见 `oocp-test-suite`。
2. **产品级 A 区（P0）** — 见上文 §A：安装/崩溃/插件安全/兼容等，可在内核里程碑确认后集中排期。
3. **B2 / B4 未勾项** — 多模态、参考硬件靶子、边缘运维等仍为中长期（P1/P2），与内核里程碑解耦。
4. **产品线 · 硬骨头（§A 中高成本项）** — 见 [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) **§四**：核心路径 e2e（**A1.1**）、`invoke` 矩阵（**A1.2**）、离线弱网（**A2.3**）等；**逐项单独立项**，验收标准写在 issue 正文，发版仍过 [PRODUCT_RELEASE_CHECKLIST.md](./PRODUCT_RELEASE_CHECKLIST.md)。

**按复杂度分桶的执行排序（细碎打包 vs 硬骨头分击）**：[PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md)（不改变本节 §A 勾选权威，仅作排期视图）。

---

## 相关链接

- 架构与模块编号：[OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · 内核与六宿主槽总览图：[KERNEL_AND_MODULES_ARCHITECTURE.md](../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)
- CLI / 无头 / 库 / Monolith：[OCLIVE_CLI_GUIDE.md](../creator-docs/cli/OCLIVE_CLI_GUIDE.md) · [RFC_OCLIVE_MONOLITH_MODE.md](../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)
- 第 1–6 模块与 `send_message`：[PLUGIN_V1.md](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)
- 项目全貌：[PROJECT_OVERVIEW.md](../creator-docs/getting-started/PROJECT_OVERVIEW.md)
