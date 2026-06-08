# 跨宿主记忆与角色包携带数据（契约）

**读者**：VS Code 扩展、赌场 POC、启动器、无头 `kernel_server` 等 **多发行版集成方**。  
**状态**：**Phase 1 已确认**（2026-05-20）；与 [`ROLE_PACK_SPEC.md`](ROLE_PACK_SPEC.md)、[`handoff/CHAT_STORAGE_ARCHITECTURE.md`](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) 对齐。

---

## 1. 一句话

**角色包** = 跨发行版可读的 **身份、内容、策略**（不含动态 runtime）。  
**内核** = 统一的 **加载规范**（`load_role`、`config.json` 语义、`POST /chat` 契约）。  
**各宿主** = **L2 私有状态** 自定；**L3 陪伴连续** 通过 **共用 `app.db`** 实现。

---

## 2. 三层模型（Phase 1 已确认）

| 层 | 名称 | 落点 | 跨宿主 |
|----|------|------|--------|
| **L1** | 角色包 SSOT | `roles/{role_id}/` | ✅ 桌面与 VS Code **同一份** `OCLIVE_ROLES_DIR` |
| **L2** | 宿主私有 | VS Code 编辑器上下文等 | ❌ 拼进 user message；**不新增 memory API** |
| **L3** | 跨宿主 runtime | `{app_data}/app.db` | ✅ **共库即时共享**（长期记忆、好感、关系阶段） |

```text
roles/{id}/  ──load_role──►  桌面 / VS Code / kernel_server
       │                           │
       │ L1 身份+策略+内容          │ L2 编辑器上下文 → message
       │                           │
       └──────── config.json ──────┴──► L3 共 app.db（单写者）
```

**宿主义务（L1）**：能加载同一角色包；解读 `config.json` 与蓝图 `memory_config`；**不篡改**包内只读内容。

---

## 3. 角色包携带哪些数据

完整格式见 [`ROLE_PACK_SPEC.md`](ROLE_PACK_SPEC.md)。摘要如下。

### 3.1 必带 / 强烈推荐

| 类别 | 路径 | 说明 |
|------|------|------|
| 入口 | `pipeline.ocblueprint`（v2） | 身份、`meta`、蓝图 `slot_registry` |
| 行为策略 | `config.json` | `time` / `memory` / `relation` / `chat_storage` 等 |
| 提示词 | `prompts/` | 系统提示、开场白 |
| 场景 | `scenes/{scene_id}/` | `scene.json`、描述、异地素材 |

### 3.2 策略 vs 内容（勿混淆）

| 概念 | 角色包 | `{app_data}/app.db` |
|------|--------|---------------------|
| 记忆衰减半衰期、强化系数 | `config.json` → `memory.*` | — |
| 已抽取的长期记忆条目 | 参数在包内 | **`long_term_memory` 表** |
| 好感初值定义 | `meta.relations` | 运行时 **`favorability`** 等 |
| 聊天记录 | `chat_storage.location=global` | SQLite `chat_*` |

**Phase 1 记忆抽取**：编辑器上下文拼进 message 后，**允许**被回合末记忆抽取写入 `long_term_memory`；scene 级过滤留后续。

---

## 4. Phase 1 运行决策（已确认）

| 项 | 决策 |
|----|------|
| **单内核写库** | 同一时刻仅 **一个** 内核进程写 `app.db` |
| **attach vs spawn** | **Capability-first**（共享 Rust 策略 `resolve_kernel_action`）：`/health` 可读则比较 `kernel_manifest`；本机有更全内核可 replace/spawn；`OCLIVE_KERNEL_BINARY` pin 时不替换。无服务 → spawn；仅 bundled → 降级。详见 [`DISTRO_KERNEL_LIFECYCLE.md`](../kernel/DISTRO_KERNEL_LIFECYCLE.md) |
| **端口** | 固定 **`8420`**（`OCLIVE_API_PORT`） |
| **`OCLIVE_ROLES_DIR`** | 桌面与扩展 **相同路径** |
| **`OCLIVE_APP_DATA`** | 品牌目录 `%LOCALAPPDATA%/OCLive/data`（spawn 必传；见 [`OCLIVE_APP_DATA.md`](../kernel/OCLIVE_APP_DATA.md)） |
| **Tauri 迁移** | 首次 canonical 启动时 **复制** 旧 `%APPDATA%/com.oclivenewnew.app` → `OCLive/data` |
| **`scene_id`** | 扩展 **`vscode`**；桌面 **`default`**（或包内场景 id） |
| **`session_id`** | 各宿主 **独立**，不双写同一会话 |
| **`chat_storage.location`** | **`global`** |
| **演示角色** | **`mumu` v2**；包内须含 **`scenes/vscode/`** |
| **HTTP 表面** | `GET /health` + `POST /chat` |
| **测试** | OOCP / Codex 轨道 A（[`CODEX_测试指南.md`](../../dev-notes/codex-testing/CODEX_测试指南.md)） |

**Phase 1 注意**：桌面与 VS Code 均为 **HTTP 客户端**（spawn-only / attach-first），唯一写者为 `oclive-kernel-server @ :8420`。详见 [`DISTRO_KERNEL_LIFECYCLE.md`](../kernel/DISTRO_KERNEL_LIFECYCLE.md)。

**不在 Phase 1**：WebSocket 状态推送、~~调度层~~（Phase 3 `oclive-runtimed` 可选）、赌场 POC、`memories/` 包内加载、scene 级 memory 过滤。

---

## 5. `scene_id` 与 `session_id`

| 字段 | 语义 | Phase 1 |
|------|------|---------|
| `role_path` | 角色包目录绝对路径 | `{OCLIVE_ROLES_DIR}/mumu` |
| `scene_id` | 聊天 / 叙事分桶 | 扩展 **`vscode`**；须在包 `meta.scenes` 中声明 |
| `session_id` | 同场景多会话 | 扩展侧 UUID，持久化于 `globalState` |

**注意**：`long_term_memory` 按角色维度，**不**因 `scene_id` 自动隔离。

---

## 6. 单内核多角色（已确认）

- 一个内核进程可服务多 **`role_id`**，上下文天然隔离。
- 多角色同场互动由 **蓝图 / 编排** 解决，非「每角色一内核」。
- 瓶颈在 **LLM**；多内核共写同一 `app.db` 会引入 SQLite 冲突——故 **单写者** 策略。

---

## 7. 路线图

### Phase 2：单 daemon 多宿主（**已落地 spawn-only 桌面**）

- 桌面与 VS Code 均为 **HTTP 客户端**；**内核调度策略已上收**至 `oclive_kernel_runtime`（`kernel_strategy.rs` + `kernel_runtime_health.rs`），`/health` 已增强（`kernel_manifest`、`distro_id`、`distro_profile_hash`）。
- 各宿主 **调用共享策略、执行 spawn/replace/attach**；桌面 `kernel_lifecycle/policy.rs`，VS Code 经 `oclive-cli kernel ensure --plan-only`，CLI 可直接 `kernel ensure`。
- P0 IPC（chat / role / 会话读）经 HTTP 代理；`/role_snapshot` 供跨宿主 UI 轮询。
- **User Identity HTTP**（Phase 2）：`GET /user_identity/state?role_id=&scene_id=`、`POST /user_identity/set`、`POST /user_identity/scene_set` — 与 Tauri `set_user_identity` / `get_user_identity_state` 同 impl；VS Code / attach 模式必走 HTTP。
- 规范：[`DISTRO_KERNEL_LIFECYCLE.md`](../kernel/DISTRO_KERNEL_LIFECYCLE.md)。
- WebSocket 推送仍留后续；**内核编排不改**。

### Phase 3：极薄调度层（**`oclive-runtimed`**）

- 可选二进制：health 监督 + **per-role** `POST /chat` 队列，转发至上游 `:8420`。
- **不碰 AI 逻辑**；可复用 `resolve_kernel_action` 作为 `:8420` 守护者策略（与 Phase 2 上收的 Rust 公共层对齐）。
- 发行版仍可直接调 `http://127.0.0.1:8420`（或经 scheduler 端口）。

### 赌场 POC（Phase 2+）

| 项 | 约定 |
|----|------|
| `scene_id` | `casino` |
| 局状态 | **L2**；局结束可选写 L3 |
| 参考 | [liars-bar-llm](https://github.com/LYiHub/liars-bar-llm) 仅规则参考 |

---

## 8. Phase 1 行动清单

| # | 任务 | 负责 |
|---|------|------|
| 1 | OOCP 烟测 / Codex 轨道 A | Codex |
| 2 | 仓库 **`oclive-vscode`** | 开发 |
| 3 | 扩展骨架：health + attach/spawn + chat | 开发 |
| 4 | **`mumu`** 增加 `scenes/vscode/` | 开发 |

---

## 9. 相关文档

| 文档 | 用途 |
|------|------|
| [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) | 磁盘格式 |
| [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) | 聊天 vs memory 表 |
| [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) | HTTP 契约 |
| [KERNEL_PLATFORM_DEVELOPER_PATH.md](../getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md) | 无头集成 |

---

## 10. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-05-20 | 初稿 |
| 2026-05-20 | Phase 1 决策基线：单写者 attach/spawn、共库、scene/session、Phase 2/3 路线图 |
