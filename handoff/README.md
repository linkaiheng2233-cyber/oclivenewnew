# oclivenewnew 交接包（给 Claude / 4.6）

**史料归档索引**（根目录汇总表）：[`../ARCHIVE_PROJECT_HISTORY.md`](../ARCHIVE_PROJECT_HISTORY.md)。

本目录为 **与当前仓库一致** 的交接材料；若外部另有长文《开发标准》，请 **以本仓库源码 + `02_DEVELOPMENT_PLAN_v3.8.md` + `01_DEVELOPMENT_REPORT.md` 为准**，并阅读 `03_DEVELOPMENT_STANDARDS.md` 顶部的 **勘误**。

## 文件清单

| 文件 | 说明 |
|------|------|
| `00_HANDOFF_SUMMARY.md` | 一页摘要、必读事实、阅读顺序 |
| `01_DEVELOPMENT_REPORT.md` | Cursor 落地说明：已完成项、限制、路径索引 |
| `02_DEVELOPMENT_PLAN_v3.8.md` | 执行计划定稿（ADR、硬错误修正、禁止项） |
| `03_DEVELOPMENT_STANDARDS.md` | 开发流程与规范 **+ 对外部文档的勘误** |
| `04_4.6_PROJECT_TRUTH_CHECKLIST.md` | **4.6 认知对齐清单**（与 Cursor/仓库一致，接手前必读） |
| `05_4.6_对接报告.md` | **进度快照 + 代码锚点**（门禁命令；细节以 `00` / `01` / `04` 为准） |
| `06_WEEK3-004_DEVELOPMENT_PLAN.md` | **WEEK3-004**：5 个 API 命令的任务与验收（与仓库实现对齐） |
| `07_DEVELOPMENT_STANDARDS_WEEK3-004.md` | **WEEK3-004**：代码/DTO/测试/门禁标准 |
| `08_开发落实进度报告.md` | **对接 cloud4.6**：已落实项、门禁结果、剩余工作优先级 |
| `10_ERROR_CODE_DICTIONARY.md` | 错误码字典（前后端统一处理与告警基线） |
| `11_BACKEND_FREEZE_AND_REGRESSION.md` | 后端封板：接口冻结范围与回归门禁清单 |
| `12_BACKEND_PERF_RUNBOOK.md` | 后端压测运行手册（P50/P95/P99 输出与阈值） |
| `13_PERF_BASELINE_2026-04-01.md` | 首版压测基线记录（Mock LLM） |
| `PERFORMANCE_BASELINE_ACCEPTANCE.md` | 端到端性能验收与 Release 对比 |
| `FRONTEND_CHUNK_OPTIMIZATION.md` | 前端 chunk / 懒加载 / Sentry / Monaco 移除说明 |
| `PERF_PHASES.md` | **性能阶段定序与状态**（v0.2：**P1～P3 已收尾**；P4 reqwest-async 可选） |
| `RUST_RELEASE_AND_DEPENDENCIES.md` | Release profile；tokio/reqwest workspace |
| `BUNDLE_RESOURCES_SIZING.md` | `roles`/`plugins` 随包体积度量 |
| `14_POLICY_PLUGIN_PHASE1.md` | 策略插件化第 1 步对接（接口抽象、参数配置、职责收敛） |
| `15_POLICY_PLUGIN_GUIDE_AND_ROADMAP.md` | 策略插件化架构/流程图、验收标准、风险与中长期规划 |
| `16_POLICY_RELEASE_CHECKLIST.md` | 策略插件化发布前检查清单（门禁/快照/处置/交接） |
| `17_TIME_SCENE_EXPORT_HANDOFF.md` | 虚拟时间 / 场景 / 导出：交接确认、索引、待办（对齐 DeepSeek 审查） |
| `21_CREATOR_IDENTITY_BINDING.md` | **创作者**：`manifest.identity_binding`（全局身份 vs 按场景身份）说明与选型 |
| `../roles/README_MANIFEST.md` | **创作者**：`manifest.json` 全字段分类说明（与 `roles/manifest.template.json` 对照） |
| `18_DEVELOPMENT_REPORT_USER_ACTIONS.md` | **开发汇报终稿**：已完成项 + **需你方处理项**（联调/Ollama/打包网络等） |
| `WEEKLY_DEV_GUIDE.md` | **周节奏 + 任务 + 门禁**（WEEK3 起） |
| `KERNEL_MIGRATION_COMPLETE.md` | **内核迁入收尾**：模块归属与自检命令 |
| `../creator-docs/kernel/LIGHTWEIGHT_PROFILE.md` | **runtime 轻量 profile**：`Cargo` 特性组合、场景矩阵、CI / `scripts/check_kernel_runtime_minimal.*`、`invoke` 与依赖/`http_api` 拟定说明 |
| `LIGHTWEIGHT_FOLLOWUP_PLAN.md` | **轻量化后续阶段**：`http_api` 单源、Tauri 依赖去重、`tauri-invoke-capabilities` 防呆、P4 reqwest 与执行顺序 |
| `LIGHTWEIGHT_OOCP_WS_AXUM_FOLLOWUP.md` | **子计划**：壳层 OOCP WebSocket 与 `axum` 去直连评估（中长期，与阶段 2 批量去重解耦） |
| `ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md` | **工程质量 P0 / 内核 P1 / 生态 P2**：与 DeepSeek 指令对齐的分阶段清单、验收门槛与仓库事实快照 |

**Cursor 规则**：项目根 `.cursor/rules/oclivenewnew.mdc`（指向 `handoff` 文档）。

## 建议阅读顺序

`00` → **`08`**（当前落实）→ **`14`**（策略插件化）→ **`15`**（架构图与路线图）→ **`16`**（发布检查清单）→ **`17`**（虚拟时间/场景/导出）→ **`18`**（**汇报终稿 / 你方待办**）→ **`05`**（进度快照）→ **`04`**（防呆清单）→ **`WEEKLY_DEV_GUIDE`** → `01` → `02` → `03`（按需）

## 注意

- **不要**用 `curl http://localhost:5173/api/send_message` 作为默认联调方式；Tauri 命令应通过 **`invoke`**（或你们自建的 HTTP 层，需单独说明）。
- **DTO、枚举、表结构** 以 `crates/oclive_kernel_runtime/src/models/`、`crates/oclive_kernel_runtime/migrations/001_init.sql` 为准。
