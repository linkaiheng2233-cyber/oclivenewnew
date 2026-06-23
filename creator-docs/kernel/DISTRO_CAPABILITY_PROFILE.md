# 发行版能力配置（Distro Capability Profile）

**状态**：P1 契约（Schema + 示例）**Done**；P4 profile 调度（`HostProfile` 加载与合并）**Done**（`host_profile.rs` / spawn 时 `OCLIVE_DISTRO_PROFILE`）。  
**受众**：桌面、VS Code、启动器、硬件发行版集成方。  
**SSOT 模块形状**：与角色包 `settings.json` → `plugin_backends` 对齐，见 [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md) 与 `crates/oclive_validation/src/plugin_backends.rs`。

---

## 1. 定位与边界

| 层级 | 文件位置 | 作用 |
|------|----------|------|
| **发行版** | 发行版根目录 `distro.oclive.toml`（与 bundled `bin/` 同级） | spawn 时加载的 **HostProfile**：prompt/memory/post_process、`host_flags`、可选 **`[plugin_backends]` 整表替换** |
| **角色包** | `roles/<id>/pipeline.ocblueprint` → `slot_registry`（v2）；legacy `settings.json` | 六槽默认；可被发行版 profile **整表替换**（若 profile 声明 `[plugin_backends]`） |
| **会话** | 宿主 DB / 会话覆盖 | 在有效 backends 上临时覆盖字段 |

**不承载于**：蓝图文件 `pipeline.ocblueprint` / blueprint v3 `runtime_config`（v3 冻结，见 handoff）。**不替代** Monolith `monolith.toml`（仅编译期）。后处理链扩展点 RFC（预留）：[RFC_OCLIVE_POST_PROCESS_CHAIN.md](../rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)。

**与内核二进制的关系**：配置文件描述「该发行版 spawn 时期望的有效模块矩阵 + prompt/memory 偏好」；**不**声明裁剪内核二进制。进程选择见 [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)（bundled-first spawn · attach/replace）；范围裁定见 [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md)。

---

## 2. 文件位置与命名

| 产品 | `distro_id` | 建议路径 |
|------|-------------|----------|
| **Chat Pro**（桌面 Release hero） | `desktop` | 示例：`examples/distro-profiles/desktop.oclive.toml`；**安装包**：`resources/distro-profiles/desktop.oclive.toml`（Tauri bundle，默认 spawn） |
| **VS Code Flash** | `vscode` | 扩展根 `distro.oclive.toml`；主仓镜像：`examples/distro-profiles/vscode.oclive.toml` |
| **dev lab**（非 Release） | `desktop-chat` | 示例：`examples/distro-profiles/desktop-chat.oclive.toml` |
| **AI Theater**（Deferred） | `theater` | 示例：`examples/distro-profiles/theater.oclive.toml`；**安装包**：`resources/distro-profiles/theater.oclive.toml`（`OCLIVE_SHELL=theater` 时自动选用） |
| 自定义 | 任意 | 由环境变量 `OCLIVE_DISTRO_PROFILE` 指向 |

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
memory = "builtin"            # builtin (builtin_v2 read alias) | remote | local | directory | none
emotion = "builtin"           # … | none
event = "builtin"             # … | none
prompt = "builtin"            # … | none（共景路径禁止 none，见 MODULE_NONE_SEMANTICS.md）
llm = "ollama"                # ollama | remote | directory | none（共景路径禁止 none）
agent = "builtin"             # builtin | remote | directory | none；若 host_flags.skip_agent = true，运行时强制 agent = none

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

[user_identity]
default_id = "classmate"      # optional; used when session has no explicit identity
allowed_ids = ["classmate"]   # optional whitelist for set_user_identity API

[interaction]
default_mode = "pure_chat"    # pure_chat | immersive — first-run seed when DB unset
allow_mode_switch = true
immersive_unlock_hint_after_turns = 10

[state_expression]
favor_high = "…"              # optional; appended to 【角色当前状态】 when favor ≥ 65
favor_mid  = "…"              # optional; 40 ≤ favor < 65
favor_low  = "…"              # optional; favor < 40
```

### 3.1 `plugin_backends` 枚举

与角色包 `settings.json` 相同（`snake_case`）：

| 键 | 合法值 |
|----|--------|
| `memory` | `builtin`（`builtin_v2` 读兼容 alias）, `remote`, `local`, `directory` |
| `emotion` | `builtin`（`builtin_v2` alias）, `remote`, `directory` |
| `event` | `builtin`（`builtin_v2` alias）, `remote`, `directory` |
| `prompt` | `builtin`（`builtin_v2` alias）, `remote`, `directory` |
| `llm` | `ollama`, `remote`, `directory` |
| `agent` | `builtin`, `remote`, `directory` |

`directory_plugins.*` 仅在对应 backend 为 `directory` 时使用（与角色包一致）。

### 3.2 `host_flags` 与 `slots`

- **`host_flags.skip_agent`**：为 `true` 时，运行时强制 `plugin_backends.agent = none`（与角色包声明 `agent: none` 等效）。
- **`host_flags.skip_complex_emotion`**：为 `true` 时，跳过共景复杂情感解析（`co_present` 阶段）。
- **`slots.complex_emotion`**：`off` 等价于 `skip_complex_emotion`（二者任一为 off 即关闭）。

> 说明：六槽 `none` 语义见 [MODULE_NONE_SEMANTICS.md](./MODULE_NONE_SEMANTICS.md)。发行版关闭 Agent 可用 `host_flags.skip_agent` 或 `[plugin_backends] agent = "none"`。

### 3.3 Prompt / 记忆 / 后处理（P4 映射表）

| 字段 | `full`（桌面默认） | `concise`（VS Code 示例） |
|------|-------------------|---------------------------|
| `prompt.profile` | 角色包 + 引擎锚点完整叠加 | 额外叠加「简洁回复」overlay，不删减包级人设 |
| `memory.retrieval` | 默认 8 条相关记忆 | `light`：4 条（`HostProfile.memory_retrieval`） |
| `post_process.chain` | `standard` | `minimal`（强制 builtin `profile=minimal`；`enabled=false` 仍关闭） |
| `visual_presentation.mode` | 未设（跟随角色包 `visual_presentation.enabled`） | `off` \| `image_only` \| `stage_full`（草案；Theater 示例 `stage_full`） |
| `user_identity.default_id` | 未设 | 会话无显式身份且非 sentinel 时作为默认 catalog id |
| `user_identity.allowed_ids` | 未设（不限制） | API 层拒绝列表外 id |
| `state_expression.favor_*` | 未设 | 按好感分档追加一句语气调节到 Prompt「角色当前状态」 |
| `[theater].director_plugin` | 未设（builtin prompt 模板） | Theater：`com.oclive.theater_director_official`；可被 env `OCLIVE_THEATER_DIRECTOR_PLUGIN` 覆盖 |

```toml
[theater]
director_plugin = "com.oclive.theater_director_official"
```

**合并规则（Theater Scene Director）**：仅当 profile 或 env 声明 `director_plugin` 且 `{app_data}/plugins` 中存在对应 manifest（`provides: theater_director`）时使用 directory RPC `theater.build_prompt`；否则 **builtin**（`scene_director.rs` / `patch_scene.rs`）。RPC 失败不 500，fallback builtin。

**合并优先级（User Identity）**：DB 会话/场景覆盖 → `HostProfile.user_identity.default_id` → catalog `default_identity_id` → legacy `user_relations.prompt_hint`。

**合并规则（Reply Post-Processor）**：`post_process.chain=minimal` 时 effective `builtin.profile=minimal`；remote/directory 仍可按角色包配置解析，失败降级 builtin → raw。

**合并规则（Visual Presentation · 草案）**：`visual_presentation.mode=off` 时宿主不下发 `performance_directive`；`image_only` 仅 `kind=image`；`stage_full` 允许 `live2d` / `rig3d` adapter（Theater）。

```toml
[visual_presentation]
mode = "off"   # off | image_only | stage_full
```

---

## 4. 合并规则（P4 实现对照）

内核在 `effective_plugin_backends_for_session` 路径上合并（`host_backends.rs`）：

1. **角色基础**：从 **`slot_registry`**（v2）或 legacy `plugin_backends` 解析六槽；`directory_plugins` 取自角色包。
2. **用户 LLM 设置 / env**：`resolve_effective_ollama_model` 等 override（在 profile 之前或之后按现有路径）。
3. **发行版 profile**：若 `distro.oclive.toml` 声明 **`[plugin_backends]`**，则 **`profile_override`（实现名 `apply_host_ceiling`）用 profile 值整表替换六槽**（`directory_plugins` **不**被 profile 覆盖）。
4. **`host_flags`**：`skip_agent = true` → 强制 `agent = none`；`skip_complex_emotion` / `slots.complex_emotion = off` → 跳过共景复杂情感。
5. **会话覆盖**：`PluginBackendsOverride` 在有效 backends 上再叠一层（仍受 startup health 与 none 语义约束）。

**与旧文档差异**：**不是** `role.apply_within_ceiling(host_ceiling)` 的「交集上限」模型。稳定发行版（vscode / theater）通过 **显式 `[plugin_backends]`** 锁定矩阵；实验场（desktop-chat）**省略**该段，角色蓝图说了算。详见 [DISTRO_DEFAULT_PLUGINS.md](./DISTRO_DEFAULT_PLUGINS.md) §2。

**单进程多宿主（v1）**：`HostProfile` 为进程级；最后 spawn 的 `OCLIVE_DISTRO_ID` + env 生效。同一 `:8420` 单写者；profile 冲突 → **replace 重启**，非热切换。

---

## 5. 发行版 bundled 内核与 shared 兜底

与 [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md) 对齐：

| 场景 | 行为 |
|------|------|
| **冷启动** | 优先 spawn **本发行版 bundled** `oclive-kernel-server` |
| **bundled 失败** | 同 `OCLIVE_APP_DATA` + `OCLIVE_DISTRO_PROFILE` + `OCLIVE_ROLES_DIR` 下 spawn **shared 兜底核**；`{app_data}/plugins/` 自动复用 |
| **`promote`** | 开发者将本机构建写入 shared runtime — **维护通道**，非终端用户默认路径 |
| **Deferred** | 每发行版裁剪 binary · 进程内自升级（P3b） |

**术语**：旧称 **logical seed** = 今日 **发行版 bundled 全量核**（非裁剪体积）；spawn **优先级**由产品策略决定，与 discovery `SCORE_BUNDLED = 50` 数值无关。

---

## 6. 示例文件

| 发行版 | 路径 |
|--------|------|
| 桌面 | [`../../examples/distro-profiles/desktop.oclive.toml`](../../examples/distro-profiles/desktop.oclive.toml) |
| VS Code | [`../../examples/distro-profiles/vscode.oclive.toml`](../../examples/distro-profiles/vscode.oclive.toml) |

---

## 7. 验收（P1）

- [x] 本文档与 `plugin_backends` / slot 命名无冲突
- [x] 桌面、VS Code、theater 各有可复制的示例 TOML
- [x] 合并规则（整表替换 vs 省略 profile）与 `DISTRO_DEFAULT_PLUGINS` / `DISTRO_KERNEL_LIFECYCLE` / `KERNEL_SCHEDULER_RESCOPE` 交叉引用完整

---

## Related

- [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)
- [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) — 进程调度收窄 · 与插件矩阵分责
- [DISTRO_DEFAULT_PLUGINS.md](./DISTRO_DEFAULT_PLUGINS.md) — 发行版默认六槽矩阵与三 personas
- [VSCODE_DISTRIBUTION.md](../role-pack/VSCODE_DISTRIBUTION.md)
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- [OCLIVE_APP_DATA.md](./OCLIVE_APP_DATA.md)
