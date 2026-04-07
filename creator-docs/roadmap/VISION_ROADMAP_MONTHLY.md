# oclive 愿景落实 · 按月计划

本文把「开放平台 + 双软件 + 角色包 + 可替换记忆/情感 + 可选多语言插件」拆成**按月可交付**的里程碑。顺序可随人力微调，但**契约先于实现、默认实现先于真插件**的原则不变。

---

## 愿景支柱（对照表）

| 支柱 | 含义 | 计划中对应项 |
|------|------|----------------|
| 开放 | 不追单点 SOTA，追**可替换、可文档化、可版本化** | 契约文档、trait 边界、开源准备 |
| 双软件 | **运行时（玩家）** 与 **创作者工具** 分离，**角色包**为唯一纽带 | 包规范强化、编写器、README 分工说明 |
| 角色即工作流 | 每个角色包是一套可声明的配置 + 可选后端 | manifest 扩展、`min_runtime`、后端枚举 |
| 记忆 / 情感可换 | 七维等只是**当前默认模块**，非平台上限 | Memory/Emotion 门面、第二套实现、远期侧车/WASM |

---

## 第 1 月：契约与代码边界（地基）

**目标**：不动产品行为的前提下，把「能换什么」说清楚、接稳。

| 交付物 | 说明 |
|--------|------|
| `creator-docs/plugin-and-architecture/PLUGIN_V1.md`（或并入本文附录） | 记忆子系统输入/输出、情感/演化子系统输入/输出（DTO 级）；与现有 `chat_engine` 流程对齐。 |
| `creator-docs/role-pack/PACK_VERSIONING.md` | 包版本、`schema_version`、`min_runtime_version`、未知字段策略（忽略/报错）。**已添加初版**（见仓库 [`PACK_VERSIONING.md`](../role-pack/PACK_VERSIONING.md)）。 |
| Rust 门面 | `MemoryBackend` / `AffectEmotionPipeline`（命名以仓库为准）：**当前实现**全部作为 `Default` 实现接入，主流程只做编排。 |
| manifest / `settings.json` | 增加 **`memory_backend` / `affect_backend`** 等枚举，**仅 `default` 生效**，其余值明确报错或日志提示「未实现」。 |

**验收**：全量 `cargo test`、`npm run build`；对话与好感等行为与本月前**无回归**（或仅有可说明的显式变更）。

---

## 第 2 月：角色包编写器 MVP

**目标**：创作者**不靠手写 JSON** 也能产出可被运行时加载的包。

| 交付物 | 说明 |
|--------|------|
| 编写器形态 | 独立应用或 oclive 内「创作者模式」二选一；优先**独立**，避免与玩家端耦合过重。 |
| 功能范围 | `manifest.json` 门面字段、`settings.json` 基础段、**与后端同一套校验**（或调用/复用校验逻辑）。 |
| 导出 | 生成 `roles/{id}/` 目录或 zip，结构与 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) 一致。 |
| 文档 | 创作者路径：`creator-docs/getting-started/` 等 |

**验收**：用编写器新建/编辑一个包，**零手写 JSON** 可被 oclive 加载并正常对话。

---

## 第 3 月：证明「可替换」——第二套内置实现

**目标**：用**最小第二实现**验证 trait/配置链，而非追求更强效果。

| 交付物 | 说明 |
|--------|------|
| 第二套 Memory 或 Affect | 例如：记忆检索改为「简化 FIFO / 标签过滤」或情感侧「直通占位」；**行为可简单，接口要真走枚举**。 |
| 编写器 | 可选到第二套 backend（若该实现面向创作者开放）。 |
| 回归 | 默认 backend 仍为线上默认；切换路径有测试覆盖。 |

**验收**：同一角色包仅改 `*_backend` 字段，可观察到**可测差异**（日志或固定用例）。

---

## 第 4 月：外接插件协议草案 + 工程化

**目标**：为「多语言插件」留**正式插口**，先实现**一种**宿主侧调用方式。

| 交付物 | 说明 |
|--------|------|
| 协议草案 | 推荐 **子进程 + JSON-RPC（stdin/stdout 或本地端口）** 或 **gRPC**；文档写清版本、超时、错误码。 |
| 试点 | **记忆侧车**优先（重 IO、适合进程隔离）；情感管线可仍内置。 |
| 安全 | 不默认任意执行；manifest 声明可执行路径或 URL，用户确认策略写进文档。 |
| CI / 开源准备 | `LICENSE`、根 `README` 项目化、`.gitignore` 与密钥扫描；可选 GitHub Actions：`cargo test` + `npm run build`。**本仓库已加** `LICENSE`（MIT）、重写 `README`、`CONTRIBUTING` / `SECURITY`、`.github/workflows/ci.yml`。 |

**验收**：一个**最小外部 demo 插件**（任意语言）可被 oclive 调通一轮「检索/写入」mock。

---

## 第 5 月：包内「知识载体」与检索钩子

**目标**：自媒体/创作者**预写答案**随包分发、可版本更新。

| 交付物 | 说明 |
|--------|------|
| 包结构 | 如 `knowledge/`（Markdown 分块或 JSON FAQ）+ manifest 引用。 |
| 运行时 | 对话前 **检索/注入**（关键词或向量二选一先做轻量）；与现有 prompt 管线衔接。 |
| 编写器 | 知识块编辑与版本展示；与包版本联动。 |

**验收**：换包版本后，同一问题能反映**新预写内容**（在「以包为准」策略下）。

---

## 第 6 月：双软件叙事落地 + 可选启动器雏形

**目标**：对外说法与仓库结构一致；降低非开发者上手成本。

| 交付物 | 说明 |
|--------|------|
| 根 README | **软件 A（运行时）** / **软件 B（编写器）** 分工、安装方式、角色包放置路径。 |
| 启动器（可选） | 检测 Ollama、设置 `OCLIVE_ROLES_DIR`、拉起运行时；**可与编写器分阶段**，不必同月完成。 |
| 扩展点索引 | `creator-docs/plugin-and-architecture/EXTENSION_POINTS.md`：列出稳定 trait、manifest 字段、外接协议版本。 |

**验收**：新用户仅读 README 能分清「玩」与「做包」两条路径。

---

## 第 7 月及以后（ backlog，按需排）

| 方向 | 说明 |
|------|------|
| WASM 插件 | 在进程插件稳定后，对计算型扩展做沙箱化。 |
| 动态 `.dll`/`.so` | 仅在有强需求与 ABI 规范时考虑；默认不推荐。 |
| 奖杯 / 关系仪式、多模式（纯聊 / 沉浸）细化 | 与产品节奏对齐，可插入各月小迭代。 |
| 生态 | 示例包、模板仓库、贡献指南 `CONTRIBUTING.md`。 |

**补充（体验向 backlog）**：编写器内试聊、启动器智能依赖、角色/插件市场与愿景对照的合并清单见 **[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)**（与本文并行维护，供排期引用）。

---

## 每月固定习惯（建议）

- **契约变更**走文档 + 版本号，避免静默改字段。  
- **默认路径永远可回退**：新后端挂了能切回 `default`。  
- **测试**：trait 切换与包加载至少有一层自动化覆盖。

---

## 文档索引

- 角色包契约：[roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)  
- 创作者向：[../role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md](../role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md) 等  
- 体验差异化与愿景对照 backlog：[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)  
- 本月计划若与实现不一致，**以仓库代码与校验为准**，并回写本文。

---

*本文档随愿景迭代更新；重大方向变更时请改日期与版本说明。*
