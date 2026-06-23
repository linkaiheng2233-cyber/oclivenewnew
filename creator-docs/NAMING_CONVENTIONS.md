# OCLive 命名规范与概念对齐（SSOT）

**用途**：统一项目内核心概念的**权威名称**、crate 职责边界、canonical import 路径，以及禁止使用的别名。  
**读者**：Rust / 前端贡献者、Cursor / Agent、姊妹仓集成方。  
**状态**：2026-06-06 首版；与 [AGENTS.md](../AGENTS.md)、[OCLIVE_ARCHITECTURE_OVERVIEW.md](getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) 对齐。  
**范围**：文档与术语；**不**触发 crate 重命名或运行时代码变更。

[English summary in §0](#0-english-summary)

---

## 0. English summary

This page is the **naming SSOT** for OCLive. Key rules:

1. **Six host slots** = `memory` / `emotion` / `event` / `prompt` / `llm` / `agent` (v2: `slot_registry`; legacy: `plugin_backends`).
2. **Facility modules** = in-orchestration kernel extensions **not** in the six slots (e.g. complex emotion, expert routing).
3. **Kernel crates** are **not** renamed in v0.2.x; use the [crate quick-reference](#3-crate-层级速查表) instead.
4. **`pipeline.ocblueprint`** is a **frozen filename**; conceptually call it **blueprint file** — it is **not** a step-scheduling DSL.
5. **`dual_core`** = feature/config gate; **`dual_pipeline`** = Rust orchestrator + blueprint `pipeline.{stable,experimental}` section.
6. **Canonical imports**: DTOs → `oclive_kernel_types`; traits → `oclive_kernel_contracts`; orchestration → `oclive_kernel_host::domain::…`.
7. **Normative creator docs**: Chinese under `creator-docs/`; English mirrors under `creator-docs-en/`. Crate / module READMEs: **English body**, Chinese notes optional.

---

## 1. 核心概念权威命名

同一概念在全项目（文档、代码注释、Issue、Agent 指令）**只使用下表「权威名」**。

### 1.1 架构层级

| 权威名（中文） | Authoritative English | 定义 | 典型实例 / 路径 |
|----------------|----------------------|------|-----------------|
| **内核** | **kernel** | 负责回合编排、会话状态、DB、HTTP API 的无 UI 逻辑 | `oclive_kernel_host`、`oclive-kernel-server --api` |
| **内核宿主 crate** | **kernel host crate** | 编排 + 持久化 + HTTP 的 Rust 库 crate 名 | `oclive_kernel_host`（**不是**「发行版宿主」） |
| **发行版** | **distro** | 面向用户的前端壳 + 集成逻辑 | 桌面 `oclivenewnew-tauri`、VS Code 扩展、未来游戏壳 |
| **宿主进程** | **host process** | 运行某发行版的 OS 进程；可 attach 或 spawn 内核 | Tauri 桌面进程、VS Code extension host |
| **单写者内核** | **single-writer kernel** | 同一时刻一个 `:8420` 进程写 `app.db` | [DISTRO_KERNEL_LIFECYCLE.md](kernel/DISTRO_KERNEL_LIFECYCLE.md) |
| **角色包** | **role pack** | 身份、人格、关系、`prompts/` 等内容 | `roles/{id}/` |
| **蓝图** | **blueprint** | 槽位实例、后端路由、模型、交互/记忆策略、双核开关等系统配置 | `pipeline.ocblueprint` 内 `slot_registry`、`runtime_config` 等 |
| **契约型薄核** | **contract-type thin kernel** | 内核只做编排 + 跨宿主错误语义；能力经槽位接入 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |

> **消歧**：中文「宿主」在口语中可能指「发行版进程」或 `oclive_kernel_host` crate。文档中应写全：**发行版宿主进程** vs **内核宿主 crate（`oclive_kernel_host`）**。

### 1.2 模块分类（与六槽正交）

| 权威名（中文） | Authoritative English | 是否写入 `plugin_backends` / `slot_registry` | 说明 |
|----------------|----------------------|---------------------------------------------|------|
| **后端模块（第 1–6 模块）** | **backend module (slots 1–6)** | **是**（v2：`slot_registry.type`） | 六宿主语义槽 |
| **六槽 / 六宿主槽** | **six host slots** | 同上 | 与「第 1–6 模块」同义；**优先在架构文档用「第 N 模块」** |
| **设施模块** | **facility module** | **否** | 编排行内、不占六键的统称 |
| **第 N 设施子模块** | **facility submodule N** | **否** | 已登记编号；全名 = `{专名}` + `设施子模块` |
| **第 1 设施子模块（复杂情感）** | **complex emotion facility submodule** | 否 | 代码：`complex_emotion`、`narrative_hint` |
| **第 2 设施子模块（专家模型）** | **expert model facility submodule** | 否 | 默认实现：**专家路由** `expert_routing.json` |
| **第 3 设施子模块（立绘）** | **portrait facility submodule** | 否 | 代码：`portrait_catalog`、`visual_state_id`；实现口语：**表现导演** |
| **第 4 设施子模块（视觉表现）** | **visual presentation facility submodule** | 否 | 代码：`visual_presentation`、`performance_directive`；产品口语：**角色舞台** |
| **后端模块插件模块** | **backend module plugin** | 插件 manifest，非模块号 | 例：「第 5 模块的 directory 插件实现」 |
| **无编号设施模块** | **unnumbered facility module** | 否 | `PluginHost`、`PersonalityEngine`、好感、`Repository` 等 |
| **独立通道能力增强模块** | **side-channel capability enhancement module** | **否** | 统称；注册表 SSOT：[RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) |
| **用户身份 Prompt 模板** | **User Identity Prompt Template** | 否 | 独立通道 **`user_identity`**；角色包 `user_identities/`；pre-LLM 注入 |
| **回复后处理插件** | **Reply Post-Processor Plugin** | 否 | 独立通道 **`reply_post_process`**；`config.json` → `reply_post_processor`；trait `ReplyPostProcessor` |
| **剧场场景导演** | **Theater Scene Director** | 否 | 独立通道 **`theater_director`**；`generate_theater_scene` / `POST /theater/scene`；`provides: theater_director`（**已交付 2026-06**） |
| **后处理链** | **post-process chain** | 否（策略枚举；**尚未全链落地**） | 发行版 `distro.oclive.toml` `[post_process].chain`；RFC 见 [RFC_OCLIVE_POST_PROCESS_CHAIN.md](rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md) |

### 1.3 六槽权威键名

| 编号 | `slot_registry.type` / legacy `plugin_backends` 键 | 权威中文 | 禁止别名 |
|------|---------------------------------------------------|----------|----------|
| 第 1 模块 | `memory` | 记忆模块 | `mem_backend`、`memory_backend` |
| 第 2 模块 | `emotion` | 情感模块 | `affect`、`affect_backend`、`emotion_backend` |
| 第 3 模块 | `event` | 事件模块 | — |
| 第 4 模块 | `prompt` | Prompt 模块 | — |
| 第 5 模块 | `llm` | LLM 模块 | `model_backend`（作槽位名时） |
| 第 6 模块 | `agent` | Agent 模块 | `tool`、`skills`（作槽位名时） |

### 1.4 构建态 vs 运行时态（易混）

| 权威名 | 层次 | 含义 |
|--------|------|------|
| **单核双态构建** | 编译期 | 外核态（`PluginHost`） vs 宏核态（Monolith）；见 RFC Monolith |
| **双核双态 / 运行时双核** | 运行时 | Stable 核 vs Experimental 核；`dual_core` feature + 蓝图开关 |
| **Stable 核（稳定核）** | 运行时 | 固定六槽顺序；默认 `co_present` / `process_message` 主路径 |
| **Experimental 核（实验核）** | 运行时 | `pipeline.experimental` DAG；`DualPipelineRunner` |

**禁止**把 Monolith 称为「双核」；**禁止**把 `dual_pipeline` 模块称为「蓝图 pipeline 文件」。

---

## 2. 命名原则（执行）

1. **命名即文档**：crate / 文件名应能回答「改什么去哪」；做不到时必须在本文 + [crates/README.md](../crates/README.md) 补速查表。
2. **概念统一**：权威名唯一；别名列入 §6 禁止列表。
3. **层级自解释**：依赖方向 `types → contracts → runtime → host → {server, tauri}` 不变。
4. **职责匹配**：磁盘名 / JSON 键 / Rust 模块名不一致时，以**职责**为准并在本文标注，不强行重命名已冻结产物。
5. **理清而非推翻**：v0.2.x 不 rename crate；通过文档与 canonical import 收口 re-export。

---

## 3. Crate 层级速查表

### 3.1 完整 workspace（不仅六个）

| 层级 | Crate | 一句话职责 | 我要改… | 典型路径 |
|------|-------|-----------|---------|----------|
| L0 契约 | `oclive_kernel_types` | DTO、`AppError`、`SendMessageRequest/Response` | API 字段、错误码体 | `src/models/dto.rs` |
| L0 契约 | `oclive_kernel_contracts` | 可替换后端 trait 端口 | 新增 trait 方法 | `src/llm.rs`, `plugin_host.rs` … |
| L0 校验 | `oclive_validation` | 角色包 / 蓝图校验规则 | manifest / blueprint 规则 | `src/blueprint_v2.rs` |
| L0 校验 | `oclive_schema` | blueprint serde schema | 磁盘形状增量 | — |
| L1 引擎 | `oclive_kernel_runtime` | 纯业务公式、路径/发现常量；**过渡期 re-export L0** | Prompt 段落、`*_engine` 公式 | `domain/prompt_builder/mod.rs`（`sections.rs`） |
| L2 编排 | `oclive_kernel_host` | **`process_message`**、DB、HTTP、基础设施 | 回合流程、持久化、插件 wiring | `domain/chat_engine/` |
| L3 二进制 | `oclive_kernel_server` | 无头 `oclive-kernel-server --api` 入口 | CLI 参数 only | `src/main.rs` |
| L3 二进制 | `oclivenewnew-tauri` | 桌面 IPC 薄壳、kernel attach | Tauri 命令、深链 | `src-tauri/src/api/` |
| 工具 | `oclive-cli` | init / pack / bench / doctor | 脚手架模板 | `crates/oclive-cli/` |
| L0 基础设施 | `oclive_sqlx` | workspace 统一 sqlx 依赖与特性 | sqlx 版本/特性 bump | `crates/oclive_sqlx/` |
| L0 校验 | `oclive_validation_wasm` | pack-editor wasm 校验边界 | wasm32 构建与 re-export | `crates/oclive_validation_wasm/` |
| 测试 | `fuzz` | cargo-fuzz 目标（非 default-members） | fuzz  harness | `fuzz/` |
| **实验（已删）** | `oclive_runtimed` | HTTP 队列 + 健康代理原型 | **已于 2026-06-10 删除（D-ORPHAN-01）** | 恢复见 git 历史 |

记忆口诀：**Types = 形状，Contracts = 接口，Runtime = 公式，Host = 流程，Server/Tauri = 入口。**

### 3.2 任务 → Crate 决策树

```
改 HTTP/IPC 载荷字段？          → oclive_kernel_types
改可插拔后端接口？              → oclive_kernel_contracts
改 Prompt 怎么拼、好感怎么算？   → oclive_kernel_runtime（公式）
改一条消息的执行顺序/分支？      → oclive_kernel_host::process_message
改 Tauri 命令名/前端 invoke？    → src-tauri/src/api/*.rs
改角色包 JSON 能否通过校验？     → oclive_validation
改磁盘 blueprint 文件名/顶层键？ → 冻结（v2/v3）；仅 RFC 可动
```

### 3.1 Schema 类型例外（`oclive_validation` vs `oclive_kernel_types`）

§0 写「DTOs → `oclive_kernel_types`」指 **HTTP / IPC / 编排载荷**（`SendMessageRequest`、`KernelErrorBody` 等）。**磁盘 schema 与蓝图校验结构**以 **`oclive_validation`** 为 SSOT（如 `SlotRegistryEntry`、蓝图 `groups` 规则、`PIPELINE_BLUEPRINT_FILENAME`）。

| 类别 | SSOT crate | 说明 |
|------|------------|------|
| API / 回合 DTO | `oclive_kernel_types` | 前后端契约；回复字段为 **`reply`** |
| 蓝图 / manifest 校验 | `oclive_validation` | 加载路径、`pack validate`、wasm 边界 |
| Ergonomic re-export | `oclive_kernel_types` 或 `oclive_kernel_runtime::validation` | 仅为减少 import 路径；**禁止**在业务 crate 直接 `use oclive_validation::` 除非处于校验/加载/打包路径 |

物理迁移 `SlotRegistryEntry` → `kernel_types` 属 breaking 范围，见 [TECHNICAL_DEBT_INVENTORY.md](../handoff/TECHNICAL_DEBT_INVENTORY.md) **D-SSOT-01** 后续项。

### 3.3 六个 kernel crate 是否 rename？

**结论（v0.2.x）：不 rename crate。** 理由与影响：

| 方案 | 为什么不做 | 影响范围（若强行 rename） |
|------|-----------|-------------------------|
| 改为 `oclive_kernel_dto` 等语义化包名 | 2026-05 crate 拆分刚完成；姊妹仓 path 依赖、CI、文档、`.cursor/rules` 大量引用 | 全 workspace `Cargo.toml`、import、handoff、pack-editor validation path |
| 合并 `runtime` + `host` | 违背「纯引擎 vs I/O 编排」分层纪律 | 测试隔离性下降、headless 嵌入场景回退 |

**文档补偿（已采用）**：

- 本文 §3 + [crates/README.md](../crates/README.md)
- crate `lib.rs` 顶部英文模块说明（`oclive_kernel_runtime` 已标注 transitional re-export）
- 层级代号 **L0/L1/L2/L3** 可在 PR / Issue 中使用

**后续可选（非 v0.2）**：在 **不 rename crate** 前提下，于 `Cargo.toml` `[package.metadata.docs.rs]` 或 docs.rs 增加 `display-name` 描述；或 v0.3 评估 **crate alias**（Rust 1.77+）仅用于新代码。

### 3.4 `oclive_runtimed` 说明（已删除）

- 实验性 scheduler daemon 原型，**已于 2026-06-10 删除**（技术债 D-ORPHAN-01，从未接入产品路径）
- 恢复方式：`git log --diff-filter=D -- crates/oclive_runtimed`
- 命名中的 `runtimed` = 实验性 daemon；**不要**与 `oclive_kernel_runtime` 混淆

---

## 3.5 运行时缩写（人类文档对齐）

代码与日志中常见缩写；全名见 [human-docs/03_GLOSSARY.md](../human-docs/03_GLOSSARY.md)。

| 缩写 | 含义 | 代码锚点 |
|------|------|----------|
| **`mrid`** | manifest role id（角色包 ID） | `SendMessageRequest.role_id` |
| **`srid`** | session-scoped role id（DB / 缓存命名空间） | `conversation_state_role_id(mrid, session_id)` |
| **`pl`** | 本回合 `ResolvedRolePlugins`（六槽句柄集） | `process_message` 内局部名 |

**规则**：文档与 Issue 首次出现写「`srid`（session-scoped role id）」；禁止把 `srid` 与 `mrid` 混称为「角色 id」而不加限定。

---

## 4. Re-export 多路径与 canonical import

### 4.1 现状（已核实）

| 类型 | 真定义位置 | 过渡 re-export 路径 | 备注 |
|------|-----------|---------------------|------|
| DTO / `AppError` | `oclive_kernel_types` | `oclive_kernel_runtime::*`、`host::error::*`、`tauri::error::*` | runtime `lib.rs` 标明 **transitional** |
| Trait 端口 | `oclive_kernel_contracts` | `oclive_kernel_runtime::…`、`host::domain::ports::*` | ports **无 trait 定义** |
| 引擎模块 | `oclive_kernel_runtime::domain::*` | `oclive_kernel_host::domain::*`（`pub use`） | host 内可用 `crate::domain::` |
| 编排入口 | `oclive_kernel_host::domain::chat_engine::process_message` | ~~`oclivenewnew_tauri::domain::process_message`~~（**P1 Done**：tauri 不再 re-export `domain`） |
| HTTP API | `oclive_kernel_host::http_api` | `oclivenewnew_tauri::http_api` | attach 模式共用 |
| 校验 | `oclive_validation` | `oclive_kernel_types`（`SlotRegistryEntry` 等）、`oclive_validation_wasm` | 类型级 re-export 少量 |

### 4.2 Canonical import 路径（新代码必须遵守）

**唯一过渡规则（与 [CONTRIBUTING.md §Rust import 纪律](../CONTRIBUTING.md#rust-import-纪律) 对齐）**：

1. **新 Rust 代码**只从 canonical crate import（下表「Canonical import」列）。
2. **禁止**为取 DTO / trait 而新增 `use oclive_kernel_runtime::SendMessageRequest` 等绕路（runtime 仅保留路径、内核发现、引擎 `domain/*` 合法用途）。
3. **`src-tauri`** 经 `oclive_kernel_host` / `oclive_kernel_types` 消费内核；**勿**假设编排仍在 `src-tauri/src/domain`（**P1 Done，已迁出**）。
4. 存量 re-export 可读，但 PR 触及时优先改为 canonical 路径。

| 你在写… | Canonical import | 禁止作为新代码首选 |
|---------|------------------|-------------------|
| 任何 crate 的 DTO / 错误 | `oclive_kernel_types::…` | `oclive_kernel_runtime::SendMessageRequest` |
| Trait 端口 | `oclive_kernel_contracts::…` | 仅为了 trait 而 `use oclive_kernel_runtime::LlmClient` |
| Host 内编排 | `crate::domain::…` / `crate::domain::ports::…` | 跨 crate 直接 `use oclive_kernel_runtime::domain::chat_engine` |
| Host 外消费编排 | `oclive_kernel_host::domain::process_message` | 假设仍在 `src-tauri/src/domain`（**已迁出**） |
| Tauri 命令 impl | `oclive_kernel_host::service::*_impl` | 在 `api/` 重复业务逻辑 |
| 前端 TS 类型 | 与 `dto.rs` 对齐的手写类型 / 生成类型 | 字段名 `response`（应为 **`reply`**） |

### 4.3 过渡期结束计划（文档级，无代码承诺日期）

| 阶段 | 目标 | 完成判据 |
|------|------|----------|
| **P0（当前）** | 新 PR 遵循 §4.2；`rg` 不再新增 `runtime` 直引 DTO | CR / clippy 注释 + Agent 规则 |
| **P1** | `src-tauri` 去除仅因 re-export 存在的 `oclive_kernel_runtime` 依赖 | **Done（2026-06-07）**：`src-tauri` 经 `oclive_kernel_host` / `oclive_kernel_types` canonical import；`rg 'oclivenewnew_tauri::domain' src-tauri` 零命中 |
| **P2** | `oclive_kernel_runtime` 移除 `pub use oclive_kernel_types::*` | runtime `lib.rs` 仅导出引擎 + 常量 |
| **P3** | `domain/ports` 改为 `pub use oclive_kernel_contracts::*` 直连 | 删除经 runtime 绕路 |

参考：[handoff/ARCHITECTURE_LAYERING.md](../handoff/ARCHITECTURE_LAYERING.md)、[oclive_kernel_host/src/domain/ports/mod.rs](../crates/oclive_kernel_host/src/domain/ports/mod.rs)。

### 4.4 函数动词表（D-NAME-01 · 2026-06-11）

新 Rust 函数优先按下表选前缀；**`resolve_*` 仅用于跨宿主/回合策略裁决**（见保留清单）。

| 前缀 | 语义 | 示例 |
|------|------|------|
| **`load_*`** | 从 DB / 磁盘 / 远程读入数据 | `load_remote_token`、`load_memories` |
| **`find_*`** | 在候选路径/目录中定位唯一目标 | `find_migrations_dir`、`find_roles_dir` |
| **`pick_*`** | 从多个候选中选一个（含 env/配置默认） | `pick_mirror_enabled`、`pick_portrait_emotion`、`resolve_visual_state` |
| **`visual_state_id`** | 立绘 catalog 条目 id（第 3 设施输出） | `SendMessageResponse`（草案） |
| **`performance_directive`** | 视觉表现渲染指令（第 4 设施输出） | JSON 体；非 `reply` |
| **`build_*`** | 构造配置/URL/初始化产物 | `build_init_config`、`build_git_clone_url` |
| **`merge_*`** | 合并 includes / 叠加配置 | `merge_blueprint_includes_lenient` |
| **`compute_*`** | 纯计算、拓扑排序 | `compute_preset_target_ms`、`compute_plugin_install_order` |
| **`invoke_*`** | 对外 RPC / 远程调用适配 | `invoke_turn_rpc` |
| **`resolve_*`** | **策略裁决**：多来源优先级、trait 端口、跨仓契约锚点 | 见下表 |

**保留的 `resolve_*` 锚点（22，禁止改名）**：

| 函数 | 保留原因 |
|------|----------|
| `resolve_kernel_action` | 跨宿主内核 attach/spawn/replace 策略（VS Code / 桌面共享） |
| `resolve_effective_ollama_model` | 会话 → 云端 → 包 → env 模型链 |
| `resolve_active_user_identity` | 会话身份 → catalog → legacy 优先级 |
| `resolve_reply_post_processor` | 角色包 + HostProfile 后处理链合并 |
| `resolve_for_role` / `resolve_for_effective_backends` | `PluginHostPort` 六槽策略 |
| `resolve_turn` | `ComplexEmotionProvider` trait 回合策略 |
| `resolve_current_emotion` | `EmotionPolicy` trait 人格映射 |
| `resolve_effective_user_relation_key` | 跨 `load_role` / turn 的身份键 |
| `resolve_plugins_for_session` | 会话 namespace → 有效后端 |
| `resolve_dual_core_degraded` | `dual_core` feature gate 降级 |
| `resolve_user_emotion_for_turn` / `resolve_relation_before_turn` | 回合前编排策略 |
| `resolve_complex_emotion` / `resolve_with_session_backends` | 多实例 slot last-wins |
| `resolve_ollama_model` | manifest → env → global 回退 |
| `resolve_caller_requirements` | 发行版 capability profile |
| `resolve_api_port` | CLI/env/default 端口 |
| `resolve_interaction_ui_snapshot` / `resolve_relation_state_for_ui` | UI 快照策略 |
| `resolve_project_root` / `resolve_project_root_for_registry` | CLI SSOT 项目根 |

轮次 12 全量裁决：**35** 处非锚点 `resolve_*` 已改为上表动词；全仓剩余 **`fn resolve_` ≈ 40**（含 trait 方法、测试名、内部 helper）。

---

## 5. 蓝图 / 调度术语对齐

### 5.1 三个不同的「pipeline」

| 名称 | 是什么 | 不是什么 | 能否 rename |
|------|--------|----------|-------------|
| **`pipeline.ocblueprint`** | 角色包磁盘上的**蓝图文件**（v2 SSOT） | 调度 DSL 文件 | **文件名冻结** |
| **`slot_registry`** | 多实例后端配置总表 | 执行顺序表 | 键名冻结 |
| **蓝图 JSON 键 `pipeline`** | v3 双核下的 `{ stable, experimental }` 步骤 DAG | 与文件名 `pipeline.` 前缀同义 | v3 冻结 |
| **`dual_pipeline.rs`** | Rust 模块：`DualPipelineRunner` 运行时编排 | 蓝图文件 | 代码模块；见 §5.3 |
| **主编排路径 `co_present`** | Stable 核固定六槽顺序的实现 | 读 blueprint `steps[]` | 代码名保留 |

**权威表述**：

- 口语 / 文档：优先说 **「蓝图文件」**（blueprint file），需要 disambiguate 时再写 `` `pipeline.ocblueprint` ``。
- **禁止**说「pipeline 文件负责调度 steps」—— v2 明确 **禁止** 磁盘 `steps` / `entry` / `module_relations`（校验报错）；顺序由 **`process_message` → `co_present`** 代码审计。

### 5.2 `slot_registry` vs legacy `plugin_backends`

| 形态 | 权威场景 | 说明 |
|------|----------|------|
| **`slot_registry`** | v2+ 角色包、架构图、`save_role_slot_registry` | 多实例；折叠六槽时 **last-wins** |
| **`plugin_backends`** | legacy `settings.json`、Rust struct 名、会话快照字段 | 六键固定形状；**迁移对照 only** |
| **`PluginBackends`** | Rust 运行时折叠结果 | 非磁盘 SSOT |

### 5.3 `dual_core` vs `dual_pipeline`

| 名称 | 层次 | 职责 |
|------|------|------|
| **`dual_core`** | Cargo feature（`oclivenewnew-tauri` / `oclive_kernel_host`） | 编译期门控；未启用则 `dual_core_gated()` 走 Stable |
| **`runtime_config.dual_core.enabled`** | 蓝图配置 | 角色是否请求实验核 |
| **`dual_core_gated()`** | Rust 谓词 | feature **且** 蓝图均开才进实验路径 |
| **`dual_pipeline` / `DualPipelineRunner`** | Rust 模块 | 实验核 DAG 执行 + 快照降级 Stable |
| **`pipeline.experimental`** | 蓝图 JSON | 实验核步骤定义（action + depends_on） |
| **`pipeline.stable`** | 蓝图 JSON | Stable 步骤表（可省略由宿主注入默认） |
| **`dual_core_degraded`** | DTO 字段 | 蓝图要双核但宿主未编 `dual_core` feature |

**命名关系（一句话）**：`dual_core` = **开关**；`dual_pipeline` = **开关打开后的运行时_runner**；蓝图 **`pipeline.*`** = **开关的配置数据**。

### 5.5 发行版内核与调度（文档层）

| 术语 | 含义 | 禁止混淆 |
|------|------|----------|
| **单核** | 单进程 `127.0.0.1:8420` + 单写者 `app.db` | ≠ 多端口并行多内核 |
| **发行版 bundled 内核** | 安装包 / VSIX 自带的 `oclive-kernel-server` | spawn **首选**；discovery `SCORE_BUNDLED` 仅为 tier 标签 |
| **shared 兜底核** | `%LOCALAPPDATA%/OCLive/runtime/` 全量构建 | bundled 故障时 spawn；**同** `OCLIVE_APP_DATA` + profile |
| **需求单** | `DistroProfileRequirements`（自 `distro.oclive.toml`） | 调度 attach/replace 用；≠ 六槽合并 |
| **槽位 fallback** | remote/directory → builtin（单回合） | ≠ 换内核二进制 |
| **`binary_upgrade`** | replace 原因枚举（Rust 保留） | 产品面 **Freeze** — 见 KERNEL_SCHEDULER_RESCOPE |
| **logical seed** | 旧称 | 新文档写 **发行版 bundled 内核** |

SSOT：[DISTRO_KERNEL_LIFECYCLE.md](../kernel/DISTRO_KERNEL_LIFECYCLE.md) · [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) · [DISTRO_DEFAULT_PLUGINS.md](../kernel/DISTRO_DEFAULT_PLUGINS.md)

### 5.6 术语调整方案（文档层，不改冻结名）

| 问题 | 建议 | 改动类型 |
|------|------|----------|
| 文件名 `pipeline.ocblueprint` 暗示 DSL | 在所有新文档首次出现时用「蓝图文件 ``pipeline.ocblueprint``」 | 文档 |
| `dual_pipeline` 与蓝图 `pipeline` 键混淆 | 架构图注释：Rust 模块 `dual_pipeline` ↔ JSON `pipeline.experimental` | 文档 + 代码注释 |
| README 仍写 `src-tauri/.../process_message` | 统一指向 `oclive_kernel_host/.../process_message.rs` | 文档（[BUS_FACTOR_NOTES.md](../handoff/BUS_FACTOR_NOTES.md) 已部分正确） |
| 「capability-first」指 spawn 顺序 | 改为 **profile-aware attach + bundled-first spawn** | 文档 |
| 「apply_within_ceiling」指 profile 合并 | 改为 **`apply_host_ceiling` 整表替换**（或省略 profile） | 文档 |

## 6. 禁止使用的别名

| 禁止 | 权威替代表述 | 原因 |
|------|-------------|------|
| `response`（作 AI 回复字段名） | **`reply`** | `SendMessageResponse.reply` |
| `memory_backend` / `affect_backend` | `plugin_backends.memory` / `.emotion` 或 `slot_registry type: memory` | 早期愿景草案 |
| `Joy` / `Fearful` 等未定义 Emotion 变体 | [emotion.rs](../crates/oclive_kernel_types/src/models/emotion.rs) 枚举 | DTO 契约 |
| 「第 7 模块」指 directory 插件 | **第 K 模块的 xxx 插件实现** | 插件不占模块号 |
| 「专家模型设施模块」（中间大类） | **专家模型设施子模块** 或 **专家路由** | 架构规定 |
| `mcp_http` / `directory_plugin_process_spawn`（权限键） | `mcp:http` / `process:spawn` / `network:*` | Breaking 2026 Unreleased |
| 「pipeline 调度 steps」指 v2 主路径 | **`process_message` 代码编排** | 蓝图 steps 已禁止 |
| `oclive_kernel_runtime` 作「编排 crate」 | **`oclive_kernel_host`** | 2026-05 拆分后职责 |
| 把 Monolith 叫「双核」 | **宏核态** vs **运行时双核** | 正交概念 |

### 6.1 已移除 CLI 别名

见 [crates/oclive-cli/DEPRECATED_COMMANDS.md](../crates/oclive-cli/DEPRECATED_COMMANDS.md)（`publish`、`plugin search` 在线版等）。

---

## 7. 预留概念：后处理链（post-process chain）

**状态**：**尚未落地**为独立扩展点；仅见于 [DISTRO_CAPABILITY_PROFILE.md](kernel/DISTRO_CAPABILITY_PROFILE.md) P4 草案（`post_process.chain`）与 CLI 模板阶段名 `postprocess`。

| 项 | 约定 |
|----|------|
| **定义** | LLM 生成 **`reply` 之后**、写入会话 / 返回用户 **之前** 的可插拔修饰链 |
| **不是什么** | 不是六槽模块；不是设施子模块；**不是** Experimental 核 |
| **与现有代码** | `turn_pipeline/post.rs` 是 **内置** Stable 后处理（好感、持久化等），不是插件扩展点 |
| **落地前命名** | 英文 **post-process chain**；中文 **后处理链**；配置键预留 `post_process.*` |

---

## 8. 前端 ↔ 后端术语对照

### 8.1 Tauri invoke 映射

Tauri 将 Rust **`snake_case` 形参** 映射为前端 **`camelCase` 键**。权威封装：`src/api/*.ts`。

| Rust 命令 | TS 封装 | 常见参数（TS → Rust） |
|-----------|---------|----------------------|
| `send_message` | `src/api/chat.ts` | `req` → `SendMessageRequest` |
| `set_session_slot_override` | `src/api/settings.ts` | `roleId`, `slotType`, … |
| `save_role_slot_registry` | `src/api/settings.ts` | `roleId`, `slotRegistry` |
| `get_kernel_connection_status` | `src/api/kernel.ts` | — |
| `list_mcp_servers` | `src/api/agent.ts` | — |

**规则**：新增命令时，`src/api/` 封装必须与 `src-tauri/src/api/*.rs` 形参一致；禁止手写 snake_case 载荷。

### 8.2 核心 DTO 字段

| 概念 | Rust / JSON | 前端 TS | 禁止 |
|------|-------------|---------|------|
| AI 回复正文 | `reply: String` | `reply` | `response`, `content`（作契约字段） |
| 角色 id | `role_id` / `roleId` | `roleId` | — |
| 场景 id | `scene_id` / `sceneId` | `sceneId` | — |
| 会话 id | `session_id` / `sessionId` | `sessionId` | — |
| 双核降级标记 | `dual_core_degraded` | `dualCoreDegraded` | — |
| 槽位注册表 | `slot_registry` | `slotRegistry` | `pluginBackends`（v2 新 UI） |
| 六槽折叠 | `PluginBackends` | 仅调试/legacy | 与 v2 架构图混用 |

### 8.3 架构概念（前后端共用英文键）

| UI / 文档中文 | 代码 / JSON 英文键 |
|---------------|-------------------|
| 记忆 | `memory` |
| 情感 | `emotion` |
| 事件 | `event` |
| 提示词 | `prompt` |
| 大模型 | `llm` |
| 智能体 | `agent` |
| 复杂情感叙事 | `narrative_hint` / `complex_emotion` |
| 专家路由 | `expert_routing` |
| 蓝图分组 | `groups` |
| 沉浸模式 | `interaction_mode: immersive`（`runtime_config`） |

### 8.4 发行版特有

| 发行版 | 固定 `scene_id` | 内核连接 |
|--------|-----------------|----------|
| 桌面 | 用户可选 | attach `:8420` 或 spawn |
| VS Code | **`vscode`** | 同左；见 [CROSS_HOST_MEMORY.md](role-pack/CROSS_HOST_MEMORY.md) |

---

## 9. 模块文档语言规范

与 [creator-docs-en/README.md](../creator-docs-en/README.md) **双语收尾基线**一致，并细化到模块级：

| 文档类型 | 语言 | 说明 |
|----------|------|------|
| **创作者契约**（`creator-docs/role-pack`、`plugin-and-architecture`） | **中文 SSOT** + 英文镜像 | 规范句以中文为准 |
| **handoff / RFC** | 中文为主 | 关键表头可双语 |
| **Crate `lib.rs` / `README.md`（`crates/*`）** | **英文正文** | 模块职责、依赖方向；中文仅一句摘要可选 |
| **`crates/oclive_kernel_host/src/domain/README.md` 类** | 中文或双语表 | 逐步改为英文正文 + 链接中文 handoff |
| **代码注释** | 与所在 crate 文档语言一致 | 业务非显而易见处再写 |

**禁止**：同一 README 段落中英随机混排（如一句英文一句中文）；应 **整节** 分语言或单一语言 + 链接镜像。

**清理优先级（文档 PR，非代码）**：

1. `crates/README.md`、`oclive_kernel_*/src/lib.rs` — 已 mostly EN
2. `handoff/ARCHITECTURE_LAYERING.md` — 中文；保留
3. 根 `README.md` 过时路径 — 指向 `oclive_kernel_host`

---

## 10. 调整方案汇总（影响范围）

| 项目 | 建议 | 类型 | 影响 |
|------|------|------|------|
| Rename 6 kernel crates | **不做** | — | — |
| 新增 `NAMING_CONVENTIONS.md` | **做** | 文档 | 低；本文 |
| 统一 `process_message` 文档路径 | 做 | 文档 | 根 README、旧 blog |
| re-export P1–P3 收口 | 排期 | 代码 | 中；仅 import 变更 |
| 蓝图文件概念改称「蓝图文件」 | 做 | 文档 | 低 |
| `dual_pipeline` 模块加 rustdoc 消歧 | 可选 | 注释 | 低 |
| 后处理链扩展点 | RFC 后定名 | 未来 | 新 trait + 配置 |
| blueprint v3 / Monolith 命名 | **冻结** | — | — |

---

## 11. 延伸阅读

| 主题 | 文档 |
|------|------|
| 架构总述 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| 角色包 vs 蓝图 | [handoff/ROLE_PACK_BOUNDARY.md](../handoff/ROLE_PACK_BOUNDARY.md) |
| Crate 速查 | [crates/README.md](../crates/README.md) |
| 分层纪律 | [handoff/ARCHITECTURE_LAYERING.md](../handoff/ARCHITECTURE_LAYERING.md) |
| 双核 RFC | [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) |
| 关键路径 | [handoff/BUS_FACTOR_NOTES.md](../handoff/BUS_FACTOR_NOTES.md) |
| Agent 约束 | [AGENTS.md](../AGENTS.md) |

---

**维护**：架构或 crate 拆分变更时，同步更新 §3、§4、§6；发版前检查 §8 与 `dto.rs` 一致。
