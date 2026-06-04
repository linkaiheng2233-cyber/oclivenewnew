# 发行版能力配置（Distro Capability Profile）

**状态**：P1 契约（Schema + 示例）；运行时加载见 P4（`HostProfile`）。  
**受众**：桌面、VS Code、启动器、硬件发行版集成方。  
**SSOT 模块形状**：与角色包 `settings.json` → `plugin_backends` 对齐，见 [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md) 与 `crates/oclive_validation/src/plugin_backends.rs`。

---

## 1. 定位与边界

| 层级 | 文件位置 | 作用 |
|------|----------|------|
| **发行版** | 发行版根目录 `distro.oclive.toml`（与 bundled `bin/` 同级） | 声明该宿主连接内核时的**能力上限**与默认偏好 |
| **角色包** | `roles/<id>/settings.json` | 在发行版上限内微调 `plugin_backends` |
| **会话** | 宿主 DB / 会话覆盖 | 临时覆盖，**不可突破**发行版上限 |

**不承载于**：`pipeline.ocblueprint` / blueprint v3 `runtime_config`（v3 冻结，见 handoff）。**不替代** Monolith `monolith.toml`（仅编译期）。

**与内核二进制的关系**：配置文件描述「连接方需要什么」；二进制进化见 [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)（discovery / promote / attach）。

---

## 2. 文件位置与命名

| 发行版 | 建议路径 |
|--------|----------|
| 桌面（开发树） | 示例：`examples/distro-profiles/desktop.oclive.toml`；安装包可置于资源目录旁 |
| VS Code 扩展 | 扩展根 `distro.oclive.toml`；主仓镜像：`examples/distro-profiles/vscode.oclive.toml` |
| 自定义 | 任意路径，由环境变量 `OCLIVE_DISTRO_PROFILE` 指向 |

**发行版标识**（P4 传入内核）：

- 环境变量：`OCLIVE_DISTRO_ID`（如 `desktop`、`vscode`）
- HTTP（可选）：请求头 `X-OCLive-Distro-Id: vscode`

---

## 3. Schema（`schema_version = 1`）

```toml
schema_version = 1
distro_id = "vscode"          # 必填，稳定小写 id
display_name = "OCLive VS Code"

# --- 模块上限（可选；省略 = 不额外收紧，仅用 host_flags / slots）---
[plugin_backends]
memory = "builtin"            # builtin | builtin_v2 | remote | local | directory
emotion = "builtin"
event = "builtin"
prompt = "builtin"
llm = "ollama"                # ollama | remote | directory
agent = "builtin"             # 若 host_flags.skip_agent = true，运行时忽略此项

# --- 槽位（第 7 模块等，非 plugin_backends 字段）---
[slots]
complex_emotion = "off"       # on | off

# --- 宿主级开关（表达「关闭」而不扩展枚举）---
[host_flags]
skip_agent = true
skip_complex_emotion = true

# --- Prompt / 记忆 / 后处理（P4 映射；P1 先约定语义）---
[prompt]
profile = "concise"           # full | concise

[memory]
retrieval = "default"         # default | light

[post_process]
chain = "standard"            # standard | minimal
```

### 3.1 `plugin_backends` 枚举

与角色包 `settings.json` 相同（`snake_case`）：

| 键 | 合法值 |
|----|--------|
| `memory` | `builtin`, `builtin_v2`, `remote`, `local`, `directory` |
| `emotion` | `builtin`, `builtin_v2`, `remote`, `directory` |
| `event` | `builtin`, `builtin_v2`, `remote`, `directory` |
| `prompt` | `builtin`, `builtin_v2`, `remote`, `directory` |
| `llm` | `ollama`, `remote`, `directory` |
| `agent` | `builtin`, `remote`, `directory` |

`directory_plugins.*` 仅在对应 backend 为 `directory` 时使用（与角色包一致）。

### 3.2 `host_flags` 与 `slots`

- **`host_flags.skip_agent`**：为 `true` 时，内核不执行 Agent 编排（第七模块产品槽）。
- **`host_flags.skip_complex_emotion`**：为 `true` 时，跳过共景复杂情感解析（`co_present` 阶段）。
- **`slots.complex_emotion`**：`off` 等价于 `skip_complex_emotion`（二者任一为 off 即关闭）。

> 说明：`AgentBackend` 尚无 `none` 枚举；发行版「不要 Agent」必须用 `host_flags`，见 AGENTS.md 与后续 `MODULE_NONE_SEMANTICS` 文档。

### 3.3 Prompt / 记忆 / 后处理（P4 映射表）

| 字段 | `full`（桌面默认） | `concise`（VS Code 示例） |
|------|-------------------|---------------------------|
| `prompt.profile` | 角色包 + 引擎锚点完整叠加 | 额外叠加「简洁回复」overlay，不删减包级人设 |
| `memory.retrieval` | 默认检索深度 |  lighter 检索（更少上下文条数） |
| `post_process.chain` | `standard` | `minimal`（跳过非必要后处理） |

---

## 4. 合并规则（P4 实现对照）

内核在 `effective_plugin_backends_for_session` 路径上合并：

1. **基础上限**：从发行版 `distro.oclive.toml` 解析 `HostProfile`（能力上限 + 默认）。
2. **角色包**：`role.plugin_backends` 或 `slot_registry` 推导的 backends。
3. **合并**：`merged = role.apply_within_ceiling(host_ceiling)` — 角色请求超出上限的模块保持上限值。
4. **会话覆盖**：`PluginBackendsOverride` 仅能在上限内覆盖字段。
5. **LLM 用户设置**：现有 `user_llm_provider`（local/cloud）仍生效，但不把 `remote` LLM 强加给声明仅 `ollama` 的发行版上限之外（上限内允许）。

**单进程多宿主（v1）**：`HostProfile` 为进程级；最后成功 bind 的 `OCLIVE_DISTRO_ID` 生效。约定：同一时刻仅一个发行版 UI 作为主对话宿主（与单写者 `:8420` 一致）。

---

## 5. 逻辑种子（与 bundled 二进制）

- **逻辑种子**：发行版自带的 **完整** `oclive-kernel-server`（非裁剪体积），discovery 层 `SCORE_BUNDLED = 50`。
- **首次安装**：无 shared runtime 时 spawn bundled，保证开箱对话。
- **进化**：当本机存在更强构建（manifest 更新或 score ≥ 88）时，**宿主**执行 `promote_with_backup` 写入 `%LOCALAPPDATA%/OCLive/runtime/`，后续宿主 attach 共享副本。
- **不是**：种子进程内自我升级、连接移交（见 P3b 取消项）。

详见 [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)「Logical seed」小节（P2a 文档补充）。

---

## 6. 示例文件

| 发行版 | 路径 |
|--------|------|
| 桌面 | [`../../examples/distro-profiles/desktop.oclive.toml`](../../examples/distro-profiles/desktop.oclive.toml) |
| VS Code | [`../../examples/distro-profiles/vscode.oclive.toml`](../../examples/distro-profiles/vscode.oclive.toml) |

---

## 7. 验收（P1）

- [ ] 本文档与 `plugin_backends` / slot 命名无冲突
- [ ] 桌面、VS Code 各有一份可复制的示例 TOML
- [ ] 合并规则与 `DISTRO_KERNEL_LIFECYCLE` 交叉引用完整

---

## Related

- [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)
- [VSCODE_DISTRIBUTION.md](../role-pack/VSCODE_DISTRIBUTION.md)
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- [OCLIVE_APP_DATA.md](./OCLIVE_APP_DATA.md)
