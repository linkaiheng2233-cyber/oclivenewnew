# 产品线任务分桶（按复杂度 · 执行视图）

**用途**：把 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A / §C** 拆成「可先打包的细碎活」与「需单独攻坚的硬骨头」，便于排期。**不替代**主清单里的 `[ ]` / `[x]`；发版仍以主清单与 [PROJECT_OVERVIEW.md](../creator-docs/getting-started/PROJECT_OVERVIEW.md) §8 为准。

**排序原则**：先做**低风险、短平快、文档/流程类**（可合并为「文档与闸门日」）；再做**有边界的中小工程**；**硬骨头**各自独立里程碑，避免与细碎混在同一 PR。

**当前阶段（2026-05）**：**批次一**（§一 细碎 + §二 中小块中的文档/闸门/兼容表/ERROR_CODES §1.5/高风险演示表/README·SECURITY·CONTRIBUTING 等）**已入库**（见仓库 `CHANGELOG` `[Unreleased]` 与提交 **`33be1c4`** 一带）。**A5.1** 兼容表基线已于 **A5 结项** 再次充实（`handoff/A5_CLOSURE_SUMMARY.md`）。**默认下一焦点**：本分桶 **§四 硬骨头** — 每条单独 issue / 里程碑，勿与文档批次混 PR。

---

## 一、细碎易做（建议打包，1～2 个工作日内可清）

> 多为 **handoff / README / Issue 模板 / 互链**，几乎不改运行时；可开一个 PR 标题如 `docs(product): release hygiene batch`。
>
> **状态**：**批次一已落实**（C1 发版表、路线图互链、A9/A10 轻量、Issue 模板、C2 Breaking 半页、A5.2/A1.4 组织级等）；若日后新增细碎项，可再开 **批次二** 表格行。

| 主清单锚点 | 内容 | 备注 |
|------------|------|------|
| **C1** | 新建 **`handoff/PRODUCT_RELEASE_CHECKLIST.md`** 骨架：只列 **§A 的 P0 行**（引用主清单 §A1–A5 文案或编号），发版会议只过此表 | **已建**：[`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md)；高风险演示表见 [`PLUGIN_HIGH_RISK_ACCEPTANCE.md`](./PLUGIN_HIGH_RISK_ACCEPTANCE.md) |
| **C1（路线图互链）** | **路线图互链**：在 `VISION_ROADMAP_MONTHLY.md`、`BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md` 各加一节或一段，指向主清单 §A / 本分桶 | 对应主清单 §C1 第二项 |
| **A9.1** | **单一支持入口**：根 `README.md` 固定一句「提问先去…」+ Issue 模板指向 [FAQ](../creator-docs/FAQ.md) / [DOCUMENTATION_INDEX](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) | 机械活 |
| **A9.2** | **首发预期**：`README` 或 `README.en.md` 增加「早期采用者 / 已知限制」短段落（5～10 行级） | 与 A9.1 同 PR 亦可 |
| **A10.1（轻量版）** | **许可证三问**：README 或 `SECURITY.md` 顶部/底部 3 条 bullet（模型/插件/落盘），链到 [LICENSE_POLICY](../creator-docs/LICENSE_POLICY.md) | 完整法律审阅仍属 P1，此处先做「扫一眼能答」 |
| **C2（轻量版）** | **Breaking 流程**：`CONTRIBUTING.md` 或 `handoff/` 半页：谁审、谁写迁移、谁改校验与文档 | 与 bus factor 笔记可分两条 commit |
| **A5.2** | **CHANGELOG 纪律**：确认 CONTRIBUTING 已要求双语；可选在 **PRODUCT_RELEASE_CHECKLIST** 中加「已写 CHANGELOG 双语文案」勾选行 | 主仓已有双语 CHANGELOG 习惯则偏 **核对** |
| **A1.4（组织级）** | **回归清单版本化**：不在此重复写用例——在 **PRODUCT_RELEASE_CHECKLIST** 中 **链接** 已有 [MUMU_UI_ACCEPTANCE_CHECKLIST](./distros/MUMU_UI_ACCEPTANCE_CHECKLIST.md)、[REGRESSION_COMPLEX_EMOTION_QA](../creator-docs/guides/REGRESSION_COMPLEX_EMOTION_QA.md) 等，并注明「发版前勾选」 | 避免与现成 guides 重复维护 |

---

## 二、中小块（可并行，每条约 0.5～3 人日；适合按周迭代）

> **状态（批次三）**：**A6.1** 全界面 Han 守卫（Vitest）+ **A8.2** 长任务进度（导入文件名/市场同步/zip 安装）+ **A1** Playwright 关键路径（`build:e2e` + `e2e-mock/`）已落；**A6.1 / A8.2** 切片项可勾选主清单。

| 主清单锚点 | 内容 | 复杂度说明 |
|------------|------|----------------|
| **A5.1** | **对外兼容一页表**：主程序 semver ↔ 编写器/启动器 ↔ `min_runtime_version` ↔ 包 schema（表格 + 破坏性迁移指针） | **基线已入库**（[`COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md)、[`A5_CLOSURE_SUMMARY.md`](./A5_CLOSURE_SUMMARY.md)）；姊妹仓版本号仍须发版时人工对拍 |
| **A1.3（强化）** | **CI 与本地一致**：核对 `.github/workflows` 与 `npm run check:release` 覆盖差；缺则在 CI 或文档补一句「发版前本地补跑」 | CONTRIBUTING 已有 `check:release`，多为 **对齐与文档** |
| **A6.1（切片）** | **界面无残留中文**：按 **一个垂直域** 扫（如仅设置页 / 仅插件管理），`rg Han` + i18n 键 | 全应用一次扫完工作量大，**按切片收口** |
| **A3.2（切片）** | **`KernelErrorBody` JSON `code` + `apiErrors`**：目录插件 `ApiError` 已 JSON 化；未知码 **`UNKNOWN_WITH_CODE`** 兜底；`[CODE]` 仅 legacy | 见 `handoff/A3_CLOSURE_SUMMARY.md` |
| **A8.1（切片）** | **高频键盘路径**：例如「插件管理打开 / 关闭 / 发送」一条链 | 全 a11y 属硬骨头，先做 **单链** |
| **A8.2（单点）** | **长任务进度**：例如仅「大包导入」进度与失败态 | 与 A8.1 类似，**单场景**先闭环 |
| **A4.1（文档先行）** | **高风险能力验收表**：Markdown 表（权限 → 期望弹窗 → 拒绝后行为）+ 手工演示脚本 | 自动化可后移；先 **可演示** |
| **A2.1（子集）** | **首装失败文案**：先只做 **1～2 类路径**（如 Ollama 不可达、`roles` 不可写），错误码 + i18n +「下一步」 | 全路径做完即升入硬骨头 |

---

## 三、大块（多依赖或跨模块，按「里程碑」排，勿与细碎混 PR）

| 主清单锚点 | 内容 | 依赖 / 说明 |
|------------|------|----------------|
| **A4.2** | **权限与 manifest 一致性**：`oclive_validation`、运行时、`DIRECTORY_PLUGINS` / manifest 文档 **三面盘点** | 依赖 inventory，可能牵迁移与校验 |
| **A3.1** | **崩溃与遥测**：Sentry 构建期 DSN、设置页退出、脱敏、README 一致 | 见 `handoff/A3_CLOSURE_SUMMARY.md` |
| **A2.2** | **可选环境自检**：首次启动或设置页探测（模型、目录可写等）+ 降级说明 | 与 A2.1 文案衔接；有产品形态设计 |
| **A7** | **性能与资源**：与 perf handoff 对照，公开冷启动/长会话 **数字或已知限制** | 需测量与文案 |

---

## 四、硬骨头（单独立项，挨个敲碎）

> **下一阶段默认从这里拆 issue。** 每条建议 **独立 issue / 里程碑**；不要与「一、细碎」捆在同一迭代承诺里。

| 顺序建议 | 主清单锚点 | 为何硬 | 建议的第一锤 |
|----------|------------|--------|----------------|
| 1 | **A1.1** | **核心路径自动化**（装→启→切角→发消息→重启）稳定、防 flake、进 CI | **可 CI 子集已入库**：HTTP 重启（[`e2e-core-api-restart.mjs`](../../scripts/e2e-core-api-restart.mjs)）+ **`vite preview` Playwright**（[`distros/chat-pro/e2e/preview-shell.spec.ts`](../../distros/chat-pro/e2e/preview-shell.spec.ts)）；**A1.1c 原生壳 / 安装器** 仍单独立项 |
| 2 | **A1.2** | **`invoke` 全矩阵**或契约对照数据集，覆盖面大 | **宿主热路径已收口**：[`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md) + `invoke_hotpath_matrix.rs`（**9** 条 `*_impl`）；**golden / 余下命令** 按需加行，不挡本条 |
| 3 | **A2.3** | **离线/弱网** 全产品面（索引、Remote、市场相关） | 先画 **状态机 + 用户可见文案** 表，再按模块实现 |
| 4 | **A2.1（全集）** | 首装失败 **全路径** i18n + 引导 | A2.1 子集在「二」做完后再开「全集」里程碑 |
| 5 | **A6.1（全集）** | 全应用 Han 清零（承诺范围内） | 依赖切片策略完成后再做总扫 |
| 6 | **A8（全集）** | 无障碍与焦点体系化 | 在 A8.1/A8.2 多条切片稳定后做规范与审计 |

**说明**：硬骨头之间 **A1.1 → A1.2** 较自然；**A2.3** 可与 A2.2 并行设计但实现常更拖。**C2 Bus factor** 若要做成「可读代码 + 交接笔记」，工作量在「写作与评审」，复杂度在组织协调，也可单独列为硬骨头。

---

## 五、推荐的第一周组合（示例）

1. **Day 1**：完成 **「一、细碎」** 整包 PR + **PRODUCT_RELEASE_CHECKLIST** 骨架。  
2. **Day 2–3**：**A5.1** 兼容表 + **A4.1** 验收表（文档）。  
3. **Day 4–5**：**A2.1 子集** 或 **A3.2 一屏**（二选一，看当前用户投诉热点）。  
4. **并行立项**：**A1** 可 CI 子集（**A1.1a** HTTP、**A1.1b** Web 预览 Playwright、**A1.2** 九条 `invoke`）已落库；下一步：**A1.1c**（安装包 / Tauri 原生窗）与 **A2.2 / A2.3 / A4.2** 等，按 **§四** 单独立项。

---

## 相关链接

- 主清单：[PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)  
- 发版习惯：[PROJECT_OVERVIEW.md §8](../creator-docs/getting-started/PROJECT_OVERVIEW.md)  
- 贡献与闸门：[CONTRIBUTING.md](../CONTRIBUTING.md)
