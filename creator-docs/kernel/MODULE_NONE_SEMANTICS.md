# 六槽 `none` 语义（Module None Semantics）

**状态**：v0.3.x 运行时契约  
**SSOT 枚举**：`kernel/crates/oclive_validation/src/plugin_backends.rs`（`*Backend::None`）  
**Noop 实现**：`kernel/crates/oclive_kernel_host/src/domain/noop_slot_backends.rs`

---

## 1. 定位

`plugin_backends.<slot> = none` 表示**该编排槽在本回合不参与业务逻辑**，由零成本 Noop 后端承接 trait 调用，而不是静默回退到 `builtin`。

与发行版 **`host_flags.skip_agent`** 的关系：

| 机制 | 作用域 | 效果 |
|------|--------|------|
| `plugin_backends.agent = none` | 角色包 / 会话有效后端 | Agent 槽 Noop；`process()` 返回未处理 |
| `host_flags.skip_agent = true` | 发行版 `distro.oclive.toml` | 运行时强制 `agent = none`（见 `apply_host_ceiling`） |

二者目标一致；发行版应优先使用 `skip_agent`，角色包可直接声明 `agent: none`。

---

## 2. 各槽行为

| 槽 | `none` 行为 | 共景对话路径 |
|----|-------------|--------------|
| **memory** | 不检索长期记忆；返回空列表 | 允许 |
| **emotion** | 返回中性七维情感 | 允许 |
| **event** | 返回 `Ignore` / 0 影响 | 允许 |
| **prompt** | `build_prompt` 返回 `InvalidParameter` | **禁止**（启动健康检查拦截） |
| **llm** | `generate` 返回 `InvalidParameter` | **禁止**（启动健康检查拦截） |
| **agent** | `process` 返回 `handled: false` | 允许（跳过 Agent 短路） |

**complex_emotion** 不在六槽枚举内；关闭方式：发行版 `[slots] complex_emotion = "off"` / `skip_complex_emotion`（distro 级）与角色包级 `slot_registry` `backend: none` 或省略条目（2026-08-12 起，B 阶段 M1 落地）**并存、互不替代**；显式 `builtin` 才启用。

---

## 3. Agent `remote` / `directory`

v0.3.x+ **已实现** host-orchestrated Agent remote/directory：

- 协议：[AGENT_REMOTE_PROTOCOL.md](../plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md)
- MCP 执行统一经 `AgentMcpBridge`；remote/directory 失败（grant 拒绝除外）降级 `BuiltinReActAgent`
- 蓝图 / settings 允许 `agent` 为 `remote` / `directory` / `none`

---

## 4. 蓝图与 settings

- `pipeline.ocblueprint` → `slot_registry[].backend` 允许 `"none"`（六槽）
- `settings.json` → `plugin_backends.*` 允许 `"none"`（serde snake_case）
- `distro.oclive.toml` → `[plugin_backends]` 上限可声明 `none`

---

## 5. 校验入口

- **启动健康**：`validate_plugin_backends_slots` + `validate_co_present_dialogue_backends`（`startup_health.rs`）

---

## 6. 相关文档

- [DISTRO_CAPABILITY_PROFILE.md](./DISTRO_CAPABILITY_PROFILE.md) — `[memory].retrieval`、`skip_agent`
- [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — 六槽形状
- `AGENTS.md` — 编排与 re-export 约定
