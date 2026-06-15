# 术语速查（一页）

> 详细契约见 [creator-docs/role-pack/ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md)

| 术语 | 含义 |
|------|------|
| **mrid** | **Manifest role id** — 角色包目录名 / `manifest.json` 的 `id`（用户可见角色卡） |
| **srid** | **Session role id** — 会话命名空间 id；多会话时 `conversation_state_role_id(mrid, session_id)` |
| **pl** | **Plugin layer** — 本轮解析后的 `ResolvedPlugins`（六槽 + 设施后端实例） |
| **六槽** | memory · emotion · event · prompt · llm · agent — `plugin_backends` / `slot_registry` 宿主后端 |
| **plugin_backends** | 角色包或蓝图中的六槽后端选择（`builtin` / `remote` / `directory` / `none`） |
| **slot_registry** | 蓝图 `pipeline.ocblueprint` 中的槽位注册与覆盖（管理员层） |
| **reply** | HTTP/Tauri 响应字段名（**不是** `response`） |
| **OOCP** | OCLive Open Chat Protocol — HTTP 黑盒测试契约 |
| **host profile** | 发行版能力剖面（`distro.oclive.toml` / `HostProfile`） |
| **facility 子模块** | 编排行内设施（如 `narrative_hint`、专家路由）— **不是**六槽编号 |

## 对话 id 关系

```
用户选角色 mrid ──► ensure_role_loaded(mrid)
会话 session_id? ──► srid = f(mrid, session_id) ──► DB role_runtime / 记忆键
```

## 易混对照

| 不要说 | 应该说 |
|--------|--------|
| `memory_backend` | `plugin_backends.memory` |
| `response` 字段 | `reply` |
| 蓝图 steps[] 调度首轮 | `process_message` 顺序编排（steps 非首轮 DSL） |

英文镜像：[human-docs-en/09_GLOSSARY.md](../human-docs-en/09_GLOSSARY.md)
