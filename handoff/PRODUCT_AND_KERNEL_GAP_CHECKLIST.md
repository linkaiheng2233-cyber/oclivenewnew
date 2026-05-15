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

- [ ] **核心路径自动化（GUI / 安装器）**：安装 → 启动 → 切角色 → 发消息 → 关开恢复；**HTTP `--api` 宿主进程重启** 子集已 CI（[`scripts/e2e-core-api-restart.mjs`](../scripts/e2e-core-api-restart.mjs)，见 [`OOCP_TEST_SUITE.md`](../creator-docs/testing/OOCP_TEST_SUITE.md)）。
- [x] **`invoke` 宿主热路径集成烟测**：高流量命令经 [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md) 对齐；[`invoke_hotpath_matrix.rs`](../src-tauri/tests/invoke_hotpath_matrix.rs) 串联 **9** 条 `*_impl`。**全 handler golden / 真 IPC** 仍后续增强。
- [ ] **发版前闸门与 CI 对齐**：本地 `npm run check:release` / 全量 `cargo test` 与 CI 一致执行习惯写进贡献说明并坚持。
- [ ] **回归清单版本化**：关键手工场景（导入包、目录插件、权限拒绝路径）有勾选表，随版本更新。

### A2. 首装、环境与可恢复性（P0）

- [ ] **首装失败路径文案**：Ollama/模型、roles 路径、权限、杀毒误拦 — 错误码 + i18n 覆盖「下一步怎么做」。
- [ ] **可选环境自检**：首次启动或设置页可触发轻量探测（模型可达性、目录可写等），失败时降级说明清楚。
- [ ] **离线/弱网**：索引同步、远程插件等失败时的可理解状态与重试，避免「静默坏」。

### A3. 崩溃、遥测与隐私（P0 / P1）

- [ ] **崩溃与诊断**：Sentry 或等价方案若启用 — 默认开关、隐私说明、脱敏规则、用户可关闭（与根 README 叙述一致）。
- [ ] **日志与 `[CODE]`**：用户可见错误已走 code + 前端映射的继续扫尾；避免 Rust 直出整句中文给 UI。

### A4. 插件与安全边界（P0）

- [ ] **高风险能力验收表**：`process:spawn`、`network:*`、stdio MCP 等 — 弹窗、拒绝后降级、审计记录可演示。
- [ ] **权限与 manifest 一致性**：文档、校验 crate、运行时行为三者对齐，避免「文档说有、实际没有」。

### A5. 版本、兼容与升级（P0）

- [ ] **对外兼容一页表**：主程序版本 ↔ 编写器/启动器 ↔ `min_runtime_version` ↔ 角色包 schema；破坏性变更的迁移提示。
- [ ] **CHANGELOG 纪律**：用户可感知的变更必记；插件作者可据此适配。

### A6. 国际化与文案（P1）

- [ ] **界面语言切换后无残留中文**（在承诺范围内；长尾可声明例外）。
- [x] **creator-docs-en（创作者文档英文镜像）**：主干（总索引、插件契约、`guides/`、`LICENSE_POLICY`、FAQ/兼容等）已与 [DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) 及 [creator-docs-en/README.md](../creator-docs-en/README.md) 对拍收尾；**`creator-docs/roadmap/`** 等愿景与生态长文仍以中文为准；**更新纪律**（契约变更时同步英文或 CHANGELOG 声明）见英文 README 小节 [Documentation bilingual closure baseline](../creator-docs-en/README.md#documentation-bilingual-closure-baseline)。

### A7. 性能与资源（P1）

- [ ] **低端机 / 冷启动 / 长会话**：与 [13_PERF_BASELINE_2026-04-01.md](./13_PERF_BASELINE_2026-04-01.md) 等基线对照，公开「已知限制」或数字底线。

### A8. 无障碍与基础 UX（P1）

- [ ] **高频路径键盘与焦点**：设置、插件管理、聊天发送、关闭对话框。
- [ ] **长任务进度**：导入、拉取、大索引等不可无限转圈无反馈。

### A9. 支持与预期（P1）

- [ ] **单一主入口**：Issue 模板、FAQ、社区/邮件任选其一写死；降低「不知道去哪问」。
- [ ] **首发预期管理**：README 明确「早期采用者」范围与已知限制。

### A10. 法律与分发（P1）

- [ ] **许可证与免责声明**：第三方模型、插件市场、用户数据落盘 — README/隐私摘要可扫一眼即答三问。

---

## B. 纯净内核 / 机器人灵魂 / 嵌入式 / 平台

### B1. 叙事与边界（P0 / P2）

- [x] **「纯净内核」定义成文**：见 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)。
- [x] **「灵魂」交付单元**：角色包 + 有效 `plugin_backends` + 会话策略 — K3 **RobotSoulPack** profile（[ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) + `oclive pack validate --profile robot-soul` + `examples/robot-soul-minimal/`）。

### B2. 机器人 / 多模态 / 低延迟（P1 / P2）

- [ ] **多模态抽象**：语音流、视觉、触觉等若进产品 — 与现有文本回合编排的关系图与 MVP 边界（目录插件 vs 内核扩展）。
- [ ] **打断与半双工**：边听边说、唤醒打断与当前 `send_message` 模型的差距评估与 PoC。
- [ ] **多机器人 / 多用户隔离**：身份、密钥、记忆命名空间 — 若做 B 端或云边协同则升格为 P0。

### B3. 嵌入式：`kernel_server` vs `library`（P0 / P2）

- [x] **能力对称策略**：Monolith 仅 `kernel_server`；`library` 路径见 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) §5、[KERNEL_IMPLEMENTATION_PLAN.md](../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md) K4。
- [x] **脚手架 → 真内核接榫**：K1 [headless-kernel-minimal](../examples/headless-kernel-minimal/README.md)（`--api`）；K2 `oclive_kernel_runtime` / `oclive_kernel_server` + `oclive-cli --kernel-source`（见内核计划 **验收留痕**）。
- [x] **诚实范围表**：已写入 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) §6；随实机验证更新措辞。

### B4. 平台基座：工具链与开发者路径（P1 / P2）

- [x] **「平台开发者一条路径」**：[KERNEL_PLATFORM_DEVELOPER_PATH.md](../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)（中英）链到 PLUGIN_V1、SETTINGS_REFERENCE、OOCP、目录插件。
- [ ] **参考硬件或仿真靶子**：至少一类参考板或 docker-compose 侧车，降低硬件团队试错成本。
- [ ] **无头/边缘运维**：OTA、回滚、远程日志/健康检查 — 若承诺「平台」则需里程碑（**P2**，不阻塞 K1–K5）。

### B5. 姊妹仓与整机交付（P1）

- [x] **主仓 ↔ doll core / 交付包**：契约与交付说明见 [KERNEL_PLATFORM_DEVELOPER_PATH.md](../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md) 与 doll core `README.md`。

---

## C. 横切（两类目标共用）

### C1. 文档与索引（P1）

- [x] **PRODUCT_RELEASE_CHECKLIST**：仅 P0 子集可勾版（**骨架已建**：[`handoff/PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md)）；发版会议只过一张表。
- [x] **本清单与路线图互链**：`VISION_ROADMAP_MONTHLY.md`、`BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md` 已链至 §A / 分桶 / 发版勾选表。

### C2. 工程纪律（P0）

- [ ] **Breaking 变更流程**：谁审、谁写迁移、谁更新文档与校验。
- [ ] **Bus factor**：关键路径（编排、迁移、DTO）至少两人可读或有一份「若我不在」的交接笔记。

---

## D. 建议执行顺序（2026-05-15：内核 K0–K5 已收口）

1. **内核里程碑** — [KERNEL_IMPLEMENTATION_PLAN.md](../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md)：**K0–K5（除 P2 OTA）** 已在计划与工程中收口；本地验收见该文档 **「验收留痕」**；持续 CI 见 `oocp-test-suite`。
2. **产品级 A 区（P0）** — 见上文 §A：安装/崩溃/插件安全/兼容等，可在内核里程碑确认后集中排期。
3. **B2 / B4 未勾项** — 多模态、参考硬件靶子、边缘运维等仍为中长期（P1/P2），与内核里程碑解耦。
4. **产品线 · 硬骨头（§A 中高成本项）** — 见 [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) **§四**：核心路径 e2e（**A1.1**）、`invoke` 矩阵（**A1.2**）、离线弱网（**A2.3**）等；**逐项单独立项**，验收标准写在 issue 正文，发版仍过 [PRODUCT_RELEASE_CHECKLIST.md](./PRODUCT_RELEASE_CHECKLIST.md)。

**按复杂度分桶的执行排序（细碎打包 vs 硬骨头分击）**：[PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md)（不改变本节 §A 勾选权威，仅作排期视图）。

---

## 相关链接

- 内核与六槽总览：[KERNEL_AND_MODULES_ARCHITECTURE.md](../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)
- CLI / 无头 / 库 / Monolith：[OCLIVE_CLI_GUIDE.md](../creator-docs/cli/OCLIVE_CLI_GUIDE.md) · [RFC_OCLIVE_MONOLITH_MODE.md](../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)
- 六槽与 `send_message`：[PLUGIN_V1.md](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)
- 项目全貌：[PROJECT_OVERVIEW.md](../creator-docs/getting-started/PROJECT_OVERVIEW.md)
