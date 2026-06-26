# handoff/ — 活跃工程交接文档

本目录仅保留**当前仍被 AGENTS.md、CI 或贡献流程直接引用**的短文；属 **AI 接手包**（维护者深读）。**新人请先** [human-docs/06_KERNEL_LEARNING_PATH.md](../human-docs/06_KERNEL_LEARNING_PATH.md) 与 [human-docs/08_REFERENCE_MAP.md](../human-docs/08_REFERENCE_MAP.md)，再按需打开本目录。

**模块定义与槽位关系 SSOT**：[`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md)（模块注册表 · **非**进度文档）。

**物理分层**：**内核**项在**本目录根级**与 `creator-docs/kernel/`；**发行版**项在 [`distros/`](../distros/)、[`theater/`](theater/)、[`vscode/`](vscode/) 等子目录。

历史批次报告、closure summary、旧周报与编号开发计划已迁入 [`archive/`](archive/)。

---

## 文档分责（SSOT · 防耦合）

**原则**：一事一文；**链接代替复制**；无 RFC / 关键决策 **不新建** handoff 或 creator-docs 顶层文档。改文档时 **只改该文档 SSOT 范围**，勿顺手同步十处（见 [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G10–G16 · §文档编写纪律）。

### 创建 / 大改文档前（AI 与人类）

1. 查下表 — **已有 SSOT 则禁止新建**  
2. 读完该 SSOT + [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md)（若涉模块）  
3. 对源码或迁移 — **可以慢，减少错误**  
4. 写完后更新本表一行（G16）并跑 `check-stale-paths`（若动路径）

**效率源于限制**：文档越少、边界越清，接手越快。

| 主题 | 唯一 SSOT | 只链接、勿重复粘贴 |
|------|-----------|-------------------|
| **模块定义 · 六槽 · 设施 · 独立通道 · 槽间关系** | [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) | OCLIVE_ARCHITECTURE（对外叙述）、AGENTS 内核小节 |
| 六槽 DTO · `send_message` **顺序** · wire 枚举 | [`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) | MODULE_MAP（只写关系与约束） |
| 六槽 backend 24 格真值 | [`SLOT_BACKEND_REALITY_MATRIX.md`](./SLOT_BACKEND_REALITY_MATRIX.md) | MODULE_MAP · PLUGIN_V1 |
| 关键路径文件 · DB 表 | [`BUS_FACTOR_NOTES.md`](./BUS_FACTOR_NOTES.md) | MODULE_MAP |
| domain/infrastructure 分层 | [`ARCHITECTURE_LAYERING.md`](./ARCHITECTURE_LAYERING.md) | — |
| 角色包 vs 蓝图 | [`ROLE_PACK_BOUNDARY.md`](./ROLE_PACK_BOUNDARY.md) | ROLE_PACK_SPEC §0 摘要 |
| 聊天 vs 记忆三套存储 | [`CHAT_STORAGE_ARCHITECTURE.md`](./CHAT_STORAGE_ARCHITECTURE.md) | MODULE_MAP §4 摘要 |
| **Turn Thinking 持久化分流**（Fast strong_only） | [`RFC_TURN_THINKING_PERSISTENCE.md`](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) | MODULE_MAP §12 · DISTRO_CAPABILITY_PROFILE §3.2.1 |
| 发行版 HostProfile 字段 | [`DISTRO_CAPABILITY_PROFILE.md`](../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) | — |
| 活跃债 / 冻结 / OPEN | [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) | MODULE_MAP · VISION 路线图 |
| 版本与文档地图 | [`PROJECT_STATUS_AND_ALIGNMENT.md`](../creator-docs/getting-started/PROJECT_STATUS_AND_ALIGNMENT.md) | — |
| AI 改代码 / **文档**边界 | [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G1–G16 · §文档编写纪律 | `.cursor/rules` 摘要 |
| AI 审查数字核实 | [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md) | — |
| 文档总索引 | [`DOCUMENTATION_INDEX.md`](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) | 勿在 AGENTS 复制全表 |
| **人类文档包进度** | [`human-docs/README.md`](../human-docs/README.md) §文档包进度 | TECHNICAL_DEBT（代码债） |

---

## 耦合 / 过期审计（2026-06-25）

下列项 **仍留盘** 但边界已变；**勿当 truth**，维护时 **只改标注 SSOT**，勿批量同步旧文。

| 文档 | 问题 | 处理 |
|------|------|------|
| [`04_4.6_PROJECT_TRUTH_CHECKLIST.md`](./04_4.6_PROJECT_TRUTH_CHECKLIST.md) | **已归档**；路径/迁移号过时 | 仅史料；truth → BUS_FACTOR + 源码 |
| [`CHAT_STORAGE_MIRROR_COLLAPSE.md`](./CHAT_STORAGE_MIRROR_COLLAPSE.md) | 2026-06-05 **已完成** 破坏性摘要 | 历史；现行 → CHAT_STORAGE_ARCHITECTURE |
| [`USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md`](./USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md) | Phase2 **已交付** | _closure 性质；新改动走 RFC + 源码 |
| [`REPLY_POST_PROCESSOR_DESIGN_REPORT.md`](./REPLY_POST_PROCESSOR_DESIGN_REPORT.md) | 与 Phase2 重叠 | 设计史料；行为以源码 + ROLE_PACK 为准 |
| [`REPLY_POST_PROCESS_POLISH_SCOPE.md`](./REPLY_POST_PROCESS_POLISH_SCOPE.md) | 与 Phase2 重叠 | polish 范围快照；现行 → 源码 |
| [`DIMENSION5_CLOSURE_SIGNOFF.md`](./DIMENSION5_CLOSURE_SIGNOFF.md) | Dimension 5 结项签收 | 历史签收；现行门禁 → `dimension5-acceptance.mjs` |
| [`QUALITY_REVIEW_2026-06-26.md`](./QUALITY_REVIEW_2026-06-26.md) | 时点质量审查快照 | 不更新；现行 → 门禁 + TECHNICAL_DEBT |
| [`RFC_OCLIVE_KERNEL_LIBRARY.md`](./RFC_OCLIVE_KERNEL_LIBRARY.md) | 内核库 RFC 草案 | 草案；实现以源码 + MODULE_MAP 为准 |
| [`SISTER_REPO_DOC_SWEEP.md`](./SISTER_REPO_DOC_SWEEP.md) | 姊妹仓文档清扫记录 | 维护批次；现行 → 各仓 AGENTS |
| [`SPRINT_E_VISUAL_FUTURE.md`](./SPRINT_E_VISUAL_FUTURE.md) · [`LIVE2D_CUBISM_DEFER.md`](./LIVE2D_CUBISM_DEFER.md) · [`PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN.md`](./PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN.md) | 视觉 sprint / 立绘计划 | 阶段笔记；现行 → theater/DEVELOPMENT_ROADMAP · RFC_PORTRAIT |
| [`PATENT_SUBMISSION_PRIORITY.md`](./PATENT_SUBMISSION_PRIORITY.md) | 专利提交优先级 | 法务批次；非工程 truth |
| [`OPUS_48_PERF_BASELINE.md`](./OPUS_48_PERF_BASELINE.md) · [`P4_*`](./P4_CRATE_AUDIT.md) · [`P4_DUAL_CORE_AUDIT.md`](./P4_DUAL_CORE_AUDIT.md) | 阶段审查快照 | 不更新；现行 perf → PERF_PHASES · [`TTFT_BENCHMARK.md`](./TTFT_BENCHMARK.md) |
| [`PHASE4_ECOSYSTEM_NOTES.md`](./PHASE4_ECOSYSTEM_NOTES.md) · [`OPTIMIZATION_PHASE5_DECISIONS.md`](./OPTIMIZATION_PHASE5_DECISIONS.md) | 阶段笔记 |  Theater/产品决策 → theater/DEVELOPMENT_ROADMAP |
| [`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) vs MODULE_MAP | 叙述 vs 注册表 **分工** | 改模块定义 **只**改 MODULE_MAP |
| [`DEEP_PROMPT_DISTILLATION.md`](./DEEP_PROMPT_DISTILLATION.md) §2 | 曾重复归类表 | 已收敛为链接 MODULE_MAP |
| [`AGENTS.md`](../AGENTS.md) 内核长节 | 易与 MODULE_MAP 双写 | 保持摘要 + 链接；细节不进 AGENTS |
| [`handoff/archive/`](../handoff/archive/) | 故意保留 | G3：禁止当 truth |

**建议后续（非本 PR 必须）**：将 `04_4.6` · `CHAT_STORAGE_MIRROR_COLLAPSE` 物理迁入 `archive/`（需批量改链，走单独维护 PR）。

---

## 活跃文件（根目录 · 跨发行版）

| 文件 | 用途 |
|------|------|
| [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) | 破坏性变更流程 |
| [PRODUCT_LINE_TASK_BUCKETS.md](PRODUCT_LINE_TASK_BUCKETS.md) | 产品线任务分桶 |
| [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) | 技术债清单 |
| [RECURRING_OPTIMIZATION_PLAYBOOK.md](RECURRING_OPTIMIZATION_PLAYBOOK.md) | 巡检手册（§8 日志） |
| [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md) | 新人 issue 策展 |
| [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) | 双核实验运行时交接 |
| [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) | 角色包 vs 蓝图边界 |
| [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) | 分层架构说明 |
| [BUS_FACTOR_NOTES.md](BUS_FACTOR_NOTES.md) | 关键路径 bus factor |
| [MODULE_MAP_AND_HANDOFF.md](MODULE_MAP_AND_HANDOFF.md) | **模块注册表**（定义 · 划分 · 六槽/设施/独立通道关系 · 改动约束） |
| [INVOKE_HOTPATH_MATRIX.md](INVOKE_HOTPATH_MATRIX.md) | Tauri invoke 热路径矩阵 |
| [04_4.6_PROJECT_TRUTH_CHECKLIST.md](04_4.6_PROJECT_TRUTH_CHECKLIST.md) | **已归档**认知快照（以 BUS_FACTOR + 源码为准） |
| [AI_CHANGE_BOUNDARIES.md](AI_CHANGE_BOUNDARIES.md) | AI / Agent 改动边界与禁止区 |
| [PERF_PHASES.md](PERF_PHASES.md) | 性能/包体与协议验证快照 |
| [CHAT_STORAGE_ARCHITECTURE.md](CHAT_STORAGE_ARCHITECTURE.md) | 聊天混合存储架构 |
| [GITHUB_PLUGIN_INDEX_LINE.md](GITHUB_PLUGIN_INDEX_LINE.md) | GitHub 插件索引线 |
| [BLUEPRINT_FOLDER_LAYOUT.md](BLUEPRINT_FOLDER_LAYOUT.md) | 蓝图目录布局 |
| [COMMENT_ENGLISH_MIGRATION_PLAN.md](COMMENT_ENGLISH_MIGRATION_PLAN.md) | 注释英文化计划 |
| [OCLIVE_POSITIONING_DIFFERENTIATION.md](OCLIVE_POSITIONING_DIFFERENTIATION.md) | 定位与差异化 |
| [THREE_DISTRO_KERNEL_CLOSURE.md](THREE_DISTRO_KERNEL_CLOSURE.md) | 三发行版内核结项 |
| [KERNEL_SCHEDULER_RESCOPE.md](KERNEL_SCHEDULER_RESCOPE.md) | 内核调度范围重划 |

**Chat Pro（`desktop`）** = 主应用默认发行版；契约与工程文档在 `creator-docs/` 与本目录根级文件，**不单建 `handoff/desktop/`**。

## 发行版附带文档（工作文档 · 按 distro 归位）

契约 SSOT 仍在 [`creator-docs/`](../creator-docs/)；下表为各发行版**协调与工作文档**入口。

| 目录 | 发行版 | 入口 |
|------|--------|------|
| [theater/](theater/) | **AI 剧场** | [README](theater/README.md) · [DEVELOPMENT_ROADMAP](theater/DEVELOPMENT_ROADMAP.md) |
| [vscode/](vscode/) | **VS Code Flash** | [README](vscode/README.md) |
| [launcher/](launcher/) | 启动器（姊妹仓） | [README](launcher/README.md) |
| [pack-editor/](pack-editor/) | 角色包编写器（姊妹仓） | [README](pack-editor/README.md) |
| [studio/](studio/) | 工作室（合并叙事） | [README](studio/README.md) |

## 归档规则

- **迁入 `archive/`**：`*_CLOSURE_SUMMARY*`、旧 `0x_` / `1x_` 开发报告、已完成破坏性摘要（如 mirror collapse）、Phase closure 设计报告。
- **留在根目录**：上表「跨发行版」+ **模块注册表**；新增 handoff 前须满足 **RFC 或关键决策**（见 AI_CHANGE_BOUNDARIES G11）。
- **新增文档自检**：是否已有 SSOT？能否只扩展现有文一节？能否只链 MODULE_MAP / TECHNICAL_DEBT？
- **勿删** `archive/` 内文件；链接失效时改链到现行 SSOT，勿复制 archive 段落到活跃文。

性能阶段总表见本目录 [PERF_PHASES.md](PERF_PHASES.md) 与 [`creator-docs/development/`](../creator-docs/development/)。
