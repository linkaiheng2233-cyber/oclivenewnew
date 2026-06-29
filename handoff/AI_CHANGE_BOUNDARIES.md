# AI / Agent 改动边界（防脱轨 SSOT）

**用途**：约束自动化助手与外部 Agent 的**允许改动范围**；与 [`.cursor/rules/oclivenewnew.mdc`](../.cursor/rules/oclivenewnew.mdc) 摘要互补（rules 简短，本文详述）。

**元纪律**：遵守 [RECURRING_OPTIMIZATION_PLAYBOOK.md](./RECURRING_OPTIMIZATION_PLAYBOOK.md) §9 — 防回退，非追完美；**冻结 ≠ 无代码**（`dual_core` / `expert_routing` 等默认关但仓库内可有实现）。

---

## 全局硬约束

| # | 约束 | 违反后果 |
|---|------|----------|
| G1 | **角色包任务**不改蓝图 `slot_registry`、六槽 `plugin_backends`、发行版 `runtime_config` | 破坏管理员/蓝图边界 → 见 [ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) |
| G2 | **不把 RFC Draft 当「未实现」**而删除已接线 wiring | 设施子模块 / 独立通道可能已进主路径 |
| G3 | **不引用归档文档当 truth**（`handoff/archive/*`、`04_4.6` 快照、`WEEKLY_DEV_GUIDE`） | 路径与行为已与源码脱节 |
| G4 | 改 **`Cargo.lock`** 后须 `cargo audit` 并更新 [KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md) | 供应链门禁失败 |
| G5 | 改 **monorepo 路径**须 grep `roles/`、`src-tauri`、`join("roles")`；Rust 用 `chat_pro_roles_dir()` / `resolve_project_roles_dir()`；JS 用 `scripts/lib/chat-pro-roles-dir.mjs` | CI `check-stale-paths` 红 |
| G6 | **编排**只在 `oclive_kernel_host::process_message` 及 `turn_pipeline/`；Tauri `api/*.rs` 薄封装，**不在 `lib.rs` 堆业务** | 分层 ratchet 红 |
| G7 | DTO / 错误码以 `oclive_kernel_types` + [KERNEL_ERROR_CODE_CONVENTION.md](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) 为准；回复字段 **`reply`** | 前后端契约断裂 |
| G8 | 改 **公开 DTO 字段 / trait 签名 / crate 名 / 公开 re-export** 后须 `cargo test --workspace --doc`（`check:rust` 与 `--lib` **不跑 doctest**） | rustdoc 示例漂移 → CI `rust` 硬门禁红（本地 `--lib` 全绿掩盖）；见 [AI_VERIFICATION_PROTOCOL.md](./AI_VERIFICATION_PROTOCOL.md) §2.1 |
| G9 | **简洁优先 / 反冗余**：在你**已因其它原因改动**的代码里，顺手收敛明显重复（多分支手写同一大 struct、复制粘贴字段块、未用 import、自己引入的死代码）——struct 用 `#[derive(Default)]` + `..Default::default()` 或共享 base helper，只列差异字段。**收敛须行为等价 + 相关测试绿**；**禁止**为清而清做与当前任务无关的大重构（§9 防过度工程）。新加字段时优先让构造点用 `..Default::default()`，避免 N 处手写全字段。 | 认知负担累积 / 或反向触发过度重构与行为漂移 |

---

## 代码编写纪律（correct-by-construction · 反屎山）

> G9 管「把已有的重复删掉」；本节管「一开始就别写出重复 / 易错 / 耦合的代码」。**写功能前先读完本节。**

**1. 先调研，后动手（写任何功能前必做）**
- 先 `grep` 既有实现：同类 helper / DTO / trait / 路由是否已存在？**优先复用，禁止平行造第二份**（canonical import 见 [NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md) §4.2）。
- 读该路径的 SSOT 与最近同类写法再下笔：编排 → `process_message` / `turn_pipeline`；DTO → `oclive_kernel_types`；trait → `oclive_kernel_contracts`；路径 → `chat_pro_roles_dir()`。
- 边界不清就**停下问**，不要先写一版「应该差不多」的猜测代码再返工。

**2. 源头防错（correct-by-construction，让编译器替你挡 bug）**
- 用类型系统兜底：`enum` + **穷尽 `match`**（慎用 `_ =>` 吞掉新分支）；`Option` / `Result` 显式处理；`#[derive(Default)]` + `..Default::default()` 避免漏字段（G8/G9 已被此类漂移咬过）。
- 生产路径 **禁 `.unwrap()` / `.expect()`**（测试除外）；错误走 `Result` + 既有错误码（G7）。
- 新增结构体字段时，优先让所有构造点走 `..Default::default()`，避免 N 处手写全字段（这是「加一个字段改 6 处」屎山的源头）。

**3. 精简而非冗余（同功能取更短、更少分支）**
- 同等功能选代码量 / 分支更少的写法；同一逻辑复制粘贴 **≥2 次即抽 helper**。
- 不写「将来可能用得上」的参数 / 泛型 / trait（speculative generality = §9 点名的过度工程）。

**4. 解耦与可维护（人类开发者认知负担优先）**
- 守分层（G6）：编排只在 `process_message` / `turn_pipeline`；`api/*.rs` 薄封装；`lib.rs` 不堆业务。
- 一事一 **SSOT**；函数短、单一职责；公共逻辑下沉到一个 helper，不在多处镜像。
- 改动面最小化：与当前任务无关的代码**不顺手重写**（避免「修 A 带崩 B」；大重构走 §9 判定）。

**收尾自检（声称「写完」前）**：`cargo test --workspace --doc`（G8）+ 受影响的集成测 + `node scripts/dimension5-acceptance.mjs --ci`；`cargo clippy` 无新增 warning。

---

## 五列边界表

### 1. 六槽（后端模块宿主槽）

| 槽键 | SSOT | 允许改动条件 | 禁止 |
|------|------|--------------|------|
| `plugin_backends.*` | 蓝图 `pipeline.ocblueprint` · 角色 `settings.json` · [MODULE_NONE_SEMANTICS.md](../creator-docs/kernel/MODULE_NONE_SEMANTICS.md) | 管理员/蓝图任务；须同步 `oclive_validation` | 在「只改 mumu 人设」类任务里改默认槽矩阵 |
| 槽解析 / `PluginHost` | `kernel/crates/oclive_kernel_host` · `slot_runner` | 修 bug、remote/directory 协议对齐 | 未经 RFC 改槽语义或新增第七槽 |
| Agent / MCP | `domain/agent.rs` · `mcp_client.rs` · [AGENT_REMOTE_PROTOCOL.md](../creator-docs/plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md) | Agent 功能迭代；权限弹窗必经 | 跳过 `network:*` / `process:spawn` 授权 |

### 2. 设施子模块（第 1–4 设施 · 主链内）

| 设施 | SSOT | 允许改动条件 | 禁止 |
|------|------|--------------|------|
| 复杂情感 `narrative_hint` | `complex_emotion.rs` · `turn_pipeline/pre` | 设施 bug、Prompt 段落公式 | 写入 `slot_registry` |
| 立绘 `portrait_catalog` | `config.json` · `persistence.rs` post_llm | RFC 合入后的表现导演 | 用文件名当 SSOT |
| 视觉表现 `visual_presentation` | RFC 草案 · `config.json` | **RFC 未合入前**仅文档/占位 | 默认开启或二次 LLM 选图 |
| 专家路由 `expert_routing.json` | **冻结** · 见 TECHNICAL_DEBT §2 | 仅解冻后 | 扩大默认开启面 |

### 3. 独立通道能力增强（非六槽 · 非设施编号）

| `id` | SSOT | 允许改动条件 | 禁止 |
|------|------|--------------|------|
| `user_identity` | `user_identities/` · `turn_pipeline/pre` | 身份模板、API 扩展 | 进六槽或 blueprint 六键 |
| `reply_post_process` | `config.json` → `reply_post_processor` | 后处理链、directory 插件 | 默认 `enabled: true` 无审核 |
| `theater_director` | `theater_director.rs` · `POST /theater/scene` | 剧场导演、插件目录 | 进 `process_message` stage |

### 4. 角色包（创作者面）

| 项 | SSOT | 允许改动条件 | 禁止 |
|----|------|--------------|------|
| 身份 / 人格 / prompts | `distros/chat-pro/roles/<id>/` · [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) | 角色内容、立绘资源、`reply_quality_anchor` | 改 `slot_registry` / 蓝图 groups |
| `manifest.json` / `settings.json` | 同上 + `oclive_validation` | 合法新键 + 校验同步 | 虚构表名 / 未文档化顶层键 |
| Monorepo 角色目录 | **`distros/chat-pro/roles/`** only | 官方示例包 | 根级 `roles/` 作真源 |

### 5. 蓝图 / 发行版 profile

| 项 | SSOT | 允许改动条件 | 禁止 |
|----|------|--------------|------|
| `pipeline.ocblueprint` | 角色包内 · v2 磁盘真源 | 管理员、架构图写盘路径 | 用 `steps[]` 作首轮调度 DSL |
| `distro.oclive.toml` | [DISTRO_CAPABILITY_PROFILE.md](../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) | 发行版差异化 | 在角色任务里改 profile |
| `runtime_config.dual_core` | 蓝图 · **默认关** | 解冻后 | 默认开启 Experimental 核 |

---

## 路径与测试 SSOT（代码）

| 语言 | Helper | 位置 |
|------|--------|------|
| Rust | `chat_pro_roles_dir()` · `resolve_project_roles_dir()` | `kernel/crates/oclive_kernel_runtime/src/kernel_discovery.rs` |
| Rust 集成测 | `common::roles_dir()` | `distros/desktop-tauri/tests/common/mod.rs` |
| JS / 脚本 | `chatProRolesDir()` · `resolveRepoRoot()` | `scripts/lib/chat-pro-roles-dir.mjs` |
| E2E 二进制 | `findKernelBinary()` 等 | `scripts/lib/e2e-binary.mjs` |

**脚手架生成项目**内 `roles/` **保持不动**（`oclive-cli init` 输出布局，与 monorepo 真源分离）。

---

## 文档纪律（精简）

- **入口 SSOT**：人类 [`human-docs/README.md`](../human-docs/README.md) · AI [`AGENTS.md`](../AGENTS.md) + 本文
- **契约 SSOT**：[`creator-docs/`](../creator-docs/) 中文；[`creator-docs-en/`](../creator-docs-en/) 镜像
- **工程态 / 债**：[`handoff/README.md`](./README.md) · [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md)
- **禁止**：新建顶层 handoff 文（无 RFC/maintainer 确认）；复制他文大表；把 `archive/` / `04_4.6` 当现行 truth（G3）
- **深链**：人类用 [`08_REFERENCE_MAP.md`](../human-docs/08_REFERENCE_MAP.md)；全量索引用 [`DOCUMENTATION_INDEX.md`](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

---

## 门禁与验收

- `node scripts/check-stale-paths.mjs` — 文档 + 代码路径（dimension5 拆为 doc/code 两检）
- `node scripts/dimension5-acceptance.mjs --ci` — **13** 项（含 `cargo deny`）
- 关键路径索引：[BUS_FACTOR_NOTES.md](./BUS_FACTOR_NOTES.md)
- 技术债 / 冻结：[TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md)
- **审查 / 汇报核实**（带数字的质量报告、第三方审查入账前）：[AI_VERIFICATION_PROTOCOL.md](./AI_VERIFICATION_PROTOCOL.md)

---

## 相关

- [NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md) §4.2 canonical import  
- [INVOKE_HOTPATH_MATRIX.md](./INVOKE_HOTPATH_MATRIX.md) — invoke 条数 SSOT  
- [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)
