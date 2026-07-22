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
| G5 | 改 **monorepo 路径**须 grep `roles/`、`src-tauri`、`join("roles")`；Rust 用 `chat_pro_roles_dir()` / `resolve_project_roles_dir()`；JS 用 `scripts/lib/chat-pro-roles-dir.mjs`（**脚本：`scripts/check-stale-paths.mjs`**） | CI `check-stale-paths` 红 |
| G6 | **编排**只在 `oclive_kernel_host::process_message` 及 `turn_pipeline/`；Tauri `api/*.rs` 薄封装，**不在 `lib.rs` 堆业务** | 分层 ratchet 红 |
| G7 | DTO / 错误码以 `oclive_kernel_types` + [KERNEL_ERROR_CODE_CONVENTION.md](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) 为准；回复字段 **`reply`** | 前后端契约断裂 |
| G8 | 改 **公开 DTO 字段 / trait 签名 / crate 名 / 公开 re-export** 后须 `cargo test --workspace --doc`（`check:rust` 与 `--lib` **不跑 doctest**） | rustdoc 示例漂移 → CI `rust` 硬门禁红（本地 `--lib` 全绿掩盖）；见 [AI_VERIFICATION_PROTOCOL.md](./AI_VERIFICATION_PROTOCOL.md) §2.1 |
| G9 | **简洁优先 / 反冗余**：在你**已因其它原因改动**的代码里，顺手收敛明显重复（多分支手写同一大 struct、复制粘贴字段块、未用 import、自己引入的死代码）——struct 用 `#[derive(Default)]` + `..Default::default()` 或共享 base helper，只列差异字段。**收敛须行为等价 + 相关测试绿**；**禁止**为清而清做与当前任务无关的大重构（§9 防过度工程）。新加字段时优先让构造点用 `..Default::default()`，避免 N 处手写全字段。 | 认知负担累积 / 或反向触发过度重构与行为漂移 |
| G10 | **模块定义 / 划分 / 槽位与设施关系** 只改 [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md)；不在 AGENTS、OCLIVE_ARCHITECTURE、PLUGIN_V1 长文复制同表 | 文档屎山 · 改一模块牵十处 |
| G11 | **无 RFC 或关键决策记录，不新建** `handoff/*.md` / `creator-docs` 顶层文档；优先扩展现有 SSOT 一节或 `handoff/<distro>/` | 索引膨胀 · AI 无法定位 |
| G12 | **改文档只改该文 SSOT 范围**；跨主题用 **链接**；进度/债/版本 **不**写进 MODULE_MAP（走 TECHNICAL_DEBT / PROJECT_STATUS）；引用 **禁止** archive / `04_4.6` 当 truth（G3） | 牵一发而动全身 |
| G13 | **动文档前先读关联 SSOT**（[`handoff/README.md`](./README.md) §文档分责 → 该主题唯一文 → 必要时源码）；**可以慢，禁止**未读就新建/大段粘贴 | 冗余 · 与源码/他文冲突 |
| G14 | **文档零冗余**：同一事实 **一处** 维护；他处 **一行链接**；禁止把 MODULE_MAP / PLUGIN_V1 表复制进 handoff 新文 | 文档屎山 · 改一处牵十处 |
| G15 | **统一风格**（见下 §文档编写纪律）：文首 **SSOT 范围 / 最后更新**；事实用 **表**；流程用 **简图**；中文简体；状态词 **Done / OPEN / 冻结 / 草案** 与 TECHNICAL_DEBT 一致 | AI/人类无法快速扫读 |
| G16 | **新建或变更文档 SSOT 范围**时，须更新 [`handoff/README.md`](./README.md) §文档分责 **一行**（或 maintainer 确认无需登记）；**禁止** silent 新增顶层 `.md`（**由 `scripts/check-doc-registry.mjs` 强制**） | 索引失效 · 下一只 AI 找不到 |
| G17 | **按能力闭环做关联改动**：改生产者、契约或宿主行为前，先列出同一能力的生产者 → 契约/协议 → 适配层 → 消费者 → 状态/回退 → 测试；逐项核对内核、Tauri、`distros/shared`、Chat Pro/Theater、官方插件、角色包及姊妹仓，按需同步或明确“已核对无需改”。**最小改动面不等于单文件改动**；只改一端不得声称完成。 | 新旧实现并存、iframe/语音/侧栏等关联部件滞后，局部修复引出连锁回归 |

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

**5. 关联改动闭环（G17；改能力，不只改文件）**

> **经验固化**：Chat Pro 宿主升级后，语音插件、侧栏 Vue/iframe 回退与角色视觉状态曾分别保留旧路径，形成“主壳已更新、关联部件仍在旧协议”的连锁问题。此后同类任务一律按能力链核对，不按最初报错文件核对。

动手前写出最小影响链；没有对应节点时写「无」，不得省略：

```text
输入/资源 → 生产者 → wire/DTO/event/RPC → 适配与权限 → UI/宿主消费者 → 状态/回退 → 测试
```

| 变更触点 | 必查关联面 | 最小兼容证据 |
|----------|------------|--------------|
| DTO / 错误 / Tauri command | `oclive_kernel_types` → host/Tauri → `distros/shared/src/api` → 各发行版消费者 | 契约测；公开 Rust API 加 doctest；旧可选字段仍可读 |
| Host event / 插件 Bridge / RPC | 事件或命令生产者 → manifest 白名单 / `rpcMethods` → iframe bridge + Vue 注入 → 插件两种入口 | 原生 Vue 与 iframe 回退至少各一条契约/烟测；权限拒绝路径仍成立 |
| Chat Pro 壳、共享 store/composable | Fluent + Tool → `distros/shared` → 插件插槽 → Theater 是否适用 | 两壳或明确单壳范围；角色切换、刷新、卸载的状态归属测试 |
| 角色包资源 / 配置 | pack schema / catalog → loader/validation → 编排 → DTO/directive → 渲染与 legacy fallback → 编写器导出 | 实际官方包夹具 + 缺省/旧包兼容 + 编写器影响核对 |
| 语音 / 侧通道 | UI 入口 → host capture/event → plugin RPC → reply event/流式 → 播放取消与回退 | 基础能力与扩展开关；启动、切换、取消、重试；不得只测 sidecar |

- **关联修改不等于扩范围重构**：只同步同一能力链上的必要节点；发现旁支债务单独报告。
- **向后兼容优先**：优先新增可选字段、能力探测、读旧写新和明确降级；插件自身 `version` 不是宿主兼容保证。Breaking 走 [`BREAKING_CHANGE_PROCESS.md`](./BREAKING_CHANGE_PROCESS.md)。
- **完成声明**必须列出：已改节点、已核对无需改节点、兼容/回退行为、跨边界测试。模块关系与兼容层见 [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) §12.5–§12.6 与 [`COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md)。

**收尾自检（声称「写完」前）**：`cargo test --workspace --doc`（G8）+ 受影响的集成测 + `node scripts/dimension5-acceptance.mjs --ci`；`cargo clippy` 无新增 warning。涉及 Chat Pro / 目录插件 / 插槽时另跑 `npm run check:module-compat`。

---

## 文档纪律（与 G10–G12 配套）

| 你要改… | 只改 | 不要改 |
|---------|------|--------|
| 模块叫什么、属第几类、六槽关系 | [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) | AGENTS 内核长节、VISION、PLUGIN_V1 编号表 |
| DTO / 编排 stage 顺序 / wire 枚举 | [`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) | MODULE_MAP 全文 |
| backend 24 格实现真值 | [`SLOT_BACKEND_REALITY_MATRIX.md`](./SLOT_BACKEND_REALITY_MATRIX.md) | 在 MODULE_MAP 复制矩阵 |
| 活跃 OPEN / 冻结 / 下一动作 | [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) | MODULE_MAP · VISION 重复 Wave 表 |
| 文档分责与过期审计 | [`handoff/README.md`](./README.md) §文档分责 | 新建 `DOC_*.md` |
| AI 审查数字口径 | [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md) | Playbook 复制整张表 |

**新建文档准入**：须说明 (1) 现有 SSOT 无法容纳 (2) RFC 或 maintainer 关键决策 (3) 将登记进 `handoff/README` 分责表。

---

## 文档编写纪律（与 G10–G16 配套）

> **效率源于限制。** 大项目靠文档让内里有条理；文档写得越少、边界越清，AI 与人类接手越快。**慢在读 SSOT，快在改对一处。**

### 1. 动笔前：关联文档（必做，顺序不可跳）

| 步骤 | 动作 |
|------|------|
| ① 定主题 | 一句话：这份改动属于哪一类？（模块 / 契约 / 进度 / 发行版 / Theater …） |
| ② 查分责 | 打开 [`handoff/README.md`](./README.md) §文档分责 — **是否已有 SSOT？** 有则 **只扩一节**，G11 禁止新建 |
| ③ 读 SSOT | **全文或相关节**读完再改；模块类必先扫 [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) |
| ④ 对源码 | 行为描述以 **源码 / 迁移 SQL** 为准；勿信 archive / Phase closure 旧文（G3） |
| ⑤ 列关联 | 列出将 **链接** 的文档（不复制）；若需改第二份 SSOT → **停下**，通常说明边界划错 |

**禁止**：「先写一版文档占位，以后再对齐」——占位即冗余，且会被下一任 AI 当 truth。

### 2. 编写中：零冗余

- **一事一文**：进度不进 MODULE_MAP；模块定义不进 TECHNICAL_DEBT；wire 细节不进 AGENTS 长节。
- **链接 > 摘要 > 复制**：跨主题最多 **一句** 摘要 + markdown 链接；**禁止** 复制他文整表（backend 24 格、六槽边界表等）。
- **改 A 不顺手改 B**：除非 B 的 SSOT 就是当前任务；否则开独立 PR / 单独说明。
- **用户未要求不写 doc**：修 bug / 小 refactor **默认不** 新建 markdown（G11）。

### 3. 统一风格（全仓 handoff / creator-docs 工程文）

| 元素 | 规范 |
|------|------|
| **文首 5 行** | 标题 · **SSOT 范围**（本文管什么 / 不管什么）· **最后更新** · 读者（可选） |
| **状态** | `Done` · `OPEN` · `冻结` · `草案` · `已归档` — 与 [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) 用词一致 |
| **事实** | markdown **表格**；枚举与源码 **逐字一致**（`reply` 非 `response`） |
| **流程** | 短 **ASCII** 或 **mermaid**（单图够用即止）；不在 prose 里堆箭头链 |
| **路径** | 仓库根相对路径 + 反引号；monorepo 角色 **`distros/chat-pro/roles/`** |
| **中英** | 工程 handoff **中文简体**；标识符 / crate / 命令 **英文** |
| **脚注** | 大段历史放 `archive/` 或文内「已归档」块；正文只保留 **现行** |
| **模板** | 新 handoff 结构对齐 [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md) · [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) — **不**另起 DOC_STYLE.md |

### 4. 收尾自检（动文档后）

- [ ] 只改了一个 SSOT 范围？（G12）
- [ ] 未复制他文大段？（G14）
- [ ] 新建/变更 SSOT 已登记 `handoff/README` §文档分责？（G16）
- [ ] 路径存在：`node scripts/check-stale-paths.mjs`（若改路径引用）
- [ ] 模块/槽位描述与 MODULE_MAP 一致？（若相关）

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
| 立绘 `portrait_catalog` | `config.json` · `persistence.rs` post_llm | 目录解析、规则/导演选择、DTO 回填的契约演进；改动须补设施/热路径测试 | 用文件名当 SSOT |
| 视觉表现 `visual_presentation` | `config.json` · `visual_presentation` 模块 · RFC | 已交付 directive 物化与发行版 gating；渲染器仍由宿主/适配器负责 | 未经显式配置默认开启或二次 LLM 选图 |
| 专家路由 `expert_routing.json` | **冻结** · 见 TECHNICAL_DEBT §2 | 仅解冻后 | 扩大默认开启面 |

### 3. 独立通道能力增强（非六槽 · 非设施编号）

| `id` | SSOT | 允许改动条件 | 禁止 |
|------|------|--------------|------|
| `user_identity` | `user_identities/` · `turn_pipeline/pre` | 身份模板、API 扩展 | 进六槽或 blueprint 六键 |
| `reply_post_process` | `config.json` → `reply_post_processor` | 后处理链、directory 插件 | 默认 `enabled: true` 无审核 |
| `theater_director` | `theater_director.rs` · `POST /theater/scene` | 剧场导演、插件目录 | 进 `process_message` stage |
| **`voice.asr`** | [`com.oclive.voice.asr`](../distros/chat-pro/plugins/com.oclive.voice.asr/) · [`voiceAsrEvents.ts`](../distros/shared/src/lib/voiceAsrEvents.ts) · RFC §4.1 · 排查/声线合规/人设派生见 [`TRACK_VOICE_RECOGNITION.md`](../human-docs/team/TRACK_VOICE_RECOGNITION.md) §1 VX-4b · §10 | Windows ASR/TTS 插件、RPC 白名单、`examples/voice-loop-minimal/` 烟测 | 进六槽 / `process_message`；在 `slot_registry` 加 memory 类键；**克隆受版权保护的声优/角色音色进官方包或分发**（贴风格只用原创/授权/免版权音源，见 TRACK §1） |

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

### 6. Desktop 宿主（Tauri v2 capability ACL）

**SSOT：** [`distros/TAURI_V2_MIGRATION_INVENTORY.md`](./distros/TAURI_V2_MIGRATION_INVENTORY.md)（K-PLATFORM-01a **Full** · capability ACL · bump 完成）。继续：01b（前端 E2E）· 01c（CI 口径）。**勿** silent 扩大 capability / 写 remote `*`。

| 项 | SSOT | 允许改动条件 | 禁止 |
|----|------|--------------|------|
| `capabilities/` window 权限 | [`distros/desktop-tauri/capabilities/main.json`](../distros/desktop-tauri/capabilities/main.json) | Win98 合成标题栏所需 `core:window:allow-*`（minimize/maximize/unmaximize/close/start-dragging/set-decorations）；须同步 [MODULE_MAP §13.2](./MODULE_MAP_AND_HANDOFF.md) | 无产品需求时 `core:window:default` 全开或扩大无关 API |
| **`plugin_bridge_invoke`** | [`plugin_bridge.rs`](../distros/desktop-tauri/src/api/plugin_bridge.rs) · [DIRECTORY_PLUGINS.md §4.1](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) | 新桥接命令：manifest `bridge.invoke` + **`dispatch_local_bridge_command`**（桌面本地）或内核 `dispatch_bridge_command`（DB 写路径）；`plugin_rpc_invoke` 走 manifest `rpcMethods` | 假定 `ui_slots` 可直接调顶层 Tauri 命令而不经 bridge 分发 |
| **统一键位** | [`keybindings.ts`](../distros/shared/src/lib/keybindings.ts) · `useUnifiedKeybindings.ts` | 应用内 / 全局 / hold 动作目录；`voice.holdToTalk` 默认 **V** | 在 `useGlobalHotkeys` 硬编码 Ctrl+Shift 组合作 SSOT |

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
- `node scripts/check-markdown-links.mjs` — 人类中英文模块包与关键 AI/SSOT 锚点的本地 Markdown 相对链接
- `node scripts/check-doc-registry.mjs` — handoff 根级文档登记 + 重复块哨兵（G14/G16）
- `node scripts/dimension5-acceptance.mjs --ci` — 检查项总数以脚本结尾 `PASS (N checks)` 为准（含 `cargo deny` · doc registry · 人类模块链接）
- 关键路径索引：[BUS_FACTOR_NOTES.md](./BUS_FACTOR_NOTES.md)
- 技术债 / 冻结：[TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md)
- **审查 / 汇报核实**（带数字的质量报告、第三方审查入账前）：[AI_VERIFICATION_PROTOCOL.md](./AI_VERIFICATION_PROTOCOL.md)
- **模块注册表 / 文档分责**：[MODULE_MAP_AND_HANDOFF.md](./MODULE_MAP_AND_HANDOFF.md) · [handoff/README.md](./README.md) §文档分责

---

## 相关

- [NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md) §4.2 canonical import  
- [INVOKE_HOTPATH_MATRIX.md](./INVOKE_HOTPATH_MATRIX.md) — invoke 条数 SSOT  
- [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)
