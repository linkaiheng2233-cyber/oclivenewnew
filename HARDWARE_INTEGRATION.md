# OClive 硬件接入指南（Hardware Integration Guide）

> **读者**：硅胶人偶 / 机器人 / 智能硬件 / 网关固件开发者。  
> **定位**：OClive 提供 **「灵魂内核」**（记忆、情绪、人格、对话编排）；**麦克风、扬声器、舵机、屏幕、RTOS/BSP 由硬件方实现**，经本文契约接入。  
> **版本**：2026-06-11 · 对齐主仓 `0.3.x` / `SendMessageResponse.schema` **15**  
> **深度文档**：[`creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md`](creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) · [`examples/headless-kernel-minimal/README.md`](examples/headless-kernel-minimal/README.md)

---

## 0. 今晚讲解提纲（5 分钟版）

1. **OClive 不是整机方案**——它是设备上的「大脑运行时」；你们做身体，我们提供可替换的灵魂与 HTTP/插件契约。  
2. **推荐接入路径**：设备内跑 `oclive-kernel-server`（或 `oclivenewnew-tauri --api`）→ 本机 `http://127.0.0.1:8420` → 你们的中控循环：`听 → POST /chat → 读 reply + 情绪/表现指令 → TTS + 舵机`。  
3. **每轮对话你们最少要读的字段**：`reply`（要说的话）、`bot_emotion` / `portrait_emotion`（表情标签）、可选 `performance_directive`（动作/立绘指令）。  
4. **模型在本地**：Ollama / llama.cpp 目录插件；人设与记忆在 OClive；**长剧情不是硬件刚需，短回合 + 连续陪伴更合适**。  
5. **定制灵魂**：换角色包目录即可换人格，无需改内核；商用垂直品可自备 `distro.oclive.toml` 发行版配置。

---

## 1. 架构：谁负责什么

```text
┌─────────────────────────────────────────────────────────────┐
│  硬件产品层（你们实现）                                        │
│  麦克风 / ASR · 扬声器 / TTS · 舵机 / 表情电机 · 屏幕 / LED   │
│  唤醒词 · 电源管理 · 安全与合规 · 外壳与结构                    │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTP JSON（推荐）或目录插件 JSON-RPC
┌───────────────────────────▼─────────────────────────────────┐
│  OClive 灵魂内核（本仓库）                                     │
│  process_message：记忆 · 情绪 · 关系 · Prompt · 六槽 LLM      │
│  输出：reply · bot_emotion · visual_state_id · performance_directive │
└───────────────────────────┬─────────────────────────────────┘
                            │ ollama / remote / directory
┌───────────────────────────▼─────────────────────────────────┐
│  推理后端（可本地）                                            │
│  Ollama · llama.cpp 侧车 · 自研微调模型                         │
└─────────────────────────────────────────────────────────────┘
```

**原则**（见 [`PURE_KERNEL_BOUNDARY.md`](creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)）：

- 内核 **不包含** BSP、电机驱动、TTS/ASR SDK。  
- 内核 **保证** 每轮对话的编排顺序与 JSON 契约。  
- 「陪伴好不好」= 角色包内容 + 槽位实现 + 你们硬件的延迟与表现，不是改 `process_message` 一行能解决的。

---

## 2. 三种接入方式（由易到难）

| 方式 | 适用 | 说明 |
|------|------|------|
| **A. HTTP `--api`（推荐联调与量产首选）** | 主控 Linux / Android 盒 + 独立应用进程 | 跑 `oclive-kernel-server` 或 `oclivenewnew-tauri --api`，本机 `127.0.0.1:8420` |
| **B. 目录插件（Directory Plugin）** | 电机、传感器、自定义 TTS 管线 | 子进程 JSON-RPC；可挂 `llm` / `agent` 或经 `bridge/dispatch` 调工具 |
| **C. 进程内 library（进阶）** | 强嵌入式、单一固件镜像 | 链接 `oclive_kernel_runtime`；编排完整度见 [`KERNEL_PLATFORM_DEVELOPER_PATH.md`](creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md) §5 |

**今晚建议**：先打通 **方式 A**；电机/传感器用 **方式 B** 或你们直接在收到 `performance_directive` 后映射 GPIO。

---

## 3. 快速联调（30 分钟内跑通）

### 3.1 环境

- 仓库根目录，已安装 **Rust**、**Ollama**（若用本地模型）。  
- 角色包：默认 `roles/mumu`，或机器人最小包 [`examples/robot-soul-minimal`](examples/robot-soul-minimal/README.md)。

### 3.2 启动无头内核

**Windows（PowerShell，逐条执行）**

```powershell
cargo build -p oclive-kernel-server
$env:OCLIVE_USE_CANONICAL_APP_DATA = "1"
$env:OCLIVE_ROLES_DIR = "D:\oclivenewnew\roles"
$env:RUST_LOG = "info"
# 联调可先 mock；量产接 Ollama 时去掉下一行
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
.\target\debug\oclive-kernel-server.exe --api
```

**Linux / 设备**

```bash
cargo build -p oclive-kernel-server
export OCLIVE_USE_CANONICAL_APP_DATA=1
export OCLIVE_ROLES_DIR=/path/to/roles
export RUST_LOG=info
export OCLIVE_HTTP_API_MOCK_LLM=1   # 联调；量产去掉
./target/debug/oclive-kernel-server --api
```

- 默认端口：**8420**（`OCLIVE_API_PORT` 或 `--port` 可改）。  
- 当前实现 **仅绑定 `127.0.0.1`**（同机中控安全）；跨机访问需你们加网关或后续协商改绑定策略。  
- 数据目录：见 [`creator-docs/kernel/OCLIVE_APP_DATA.md`](creator-docs/kernel/OCLIVE_APP_DATA.md)（Windows 默认 `%LOCALAPPDATA%/OCLive/data`）。

### 3.3 健康检查

```bash
curl -s http://127.0.0.1:8420/health
# 或 JSON：
curl -s -H "Accept: application/json" http://127.0.0.1:8420/health
```

### 3.4 发一轮对话（硬件主循环的核心）

```bash
curl -s -X POST http://127.0.0.1:8420/chat \
  -H "Content-Type: application/json" \
  -d "{\"role_path\":\"/绝对路径/roles/mumu\",\"message\":\"你好\",\"session_id\":\"doll-001\"}"
```

**请求体**（`ChatApiRequest`）：

| 字段 | 必填 | 说明 |
|------|------|------|
| `role_path` | 是 | 角色包目录绝对路径 |
| `message` | 是 | 用户文本（由你们 ASR 填入） |
| `session_id` | 否 | 同设备多会话区分；建议硬件固定或按用户切换 |
| `scene_id` | 否 | 场景 id；短场景演绎时可切换 |
| `include_raw_reply` | 否 | 后处理调试 |

**响应**：`ChatApiResponse` = `SendMessageResponse` 全字段 + `personality_source` + 回显 `session_id`。  
**助手文本字段名是 `reply`，不是 `response`。**

### 3.5 黑盒回归（可选）

```bash
cd examples/oocp-test-suite
node run.mjs
```

见 [`creator-docs/testing/OOCP_TEST_SUITE.md`](creator-docs/testing/OOCP_TEST_SUITE.md)。

---

## 4. 硬件主循环参考（伪代码）

```text
启动:
  拉起 oclive-kernel-server --api
  等待 GET /health == ok
  加载 TTS / ASR / 舵机驱动

循环:
  音频 ← 麦克风
  text ← ASR(音频)
  若 text 为空: continue

  resp ← POST /chat { role_path, message: text, session_id }

  播放 TTS(resp.reply)
  表情 ← map_emotion(resp.bot_emotion 或 resp.portrait_emotion)
  若 resp.performance_directive 存在:
      动作 ← map_directive(resp.performance_directive)
  驱动舵机/屏幕(表情, 动作)

空闲:
  可选 idle 动画（不经过 OClive，纯硬件）
```

**延迟建议（陪伴 / 玩偶类）**：

- 目标：**ASR 结束 → 开始播放 TTS** 尽量 &lt; 1–2s（取决于模型与硬件）。  
- 优先 **本地小模型**；长生成会削弱「活着」感。  
- 可用 `POST /chat/stream`（SSE）边生成边 TTS，降低首字延迟（见 §5.2）。

---

## 5. HTTP API 速查（硬件常用）

完整路由定义：`crates/oclive_kernel_host/src/http_api/mod.rs`。

| 方法 | 路径 | 用途 |
|------|------|------|
| GET | `/health` | 就绪探测 |
| POST | `/chat` | **主路径**：一轮对话 |
| POST | `/chat/stream` | 流式 `reply`（SSE，适合边播边说） |
| POST | `/role/load` | 预加载角色包 |
| GET | `/role_info` | 角色元数据 |
| POST | `/scene/switch` | 切换场景（短场景演绎） |
| POST | `/user_identity/set` | 切换用户身份模板（可选） |
| POST | `/theater/scene` | 剧场多角色场景（非单人玩偶主路径） |
| POST | `/bridge/dispatch` | 调目录插件能力（进阶） |

错误体：`{ "error": { "code", "message", "hint?" } }`，码表见 [`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`](creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)。

### 5.1 每轮必读的响应字段

| 字段 | 类型 | 硬件用途 |
|------|------|----------|
| **`reply`** | string | **TTS 朗读正文** |
| `bot_emotion` | string | 英文情绪标签（如 `happy` / `shy`） |
| `portrait_emotion` | string | 立绘情绪（legacy 七类） |
| `visual_state_id` | string? | 角色包 catalog 中的状态 id |
| **`performance_directive`** | object? | **动作/立绘渲染指令**（见下表） |
| `favorability_current` | number | 好感度（可选驱动关系型表情） |
| `relation_state` | string | 关系阶段文案 |
| `reply_is_fallback` | bool | 为 true 时表示 LLM 失败用了兜底句 |

### 5.2 `performance_directive` 形状

定义：`crates/oclive_kernel_types/src/models/visual_presentation_config.rs`

```json
{
  "visual_state_id": "shy_smile",
  "kind": "image",
  "path": "assets/portraits/shy.png",
  "expression": "shy",
  "motion": "nod",
  "fallback_image": "...",
  "live2d_model": "...",
  "rig3d_model": "...",
  "context": "..."
}
```

**硬件映射建议**：

| `kind` | 你们可映射为 |
|--------|----------------|
| `image` | 屏幕换图 / 简单表情屏 |
| `live2d` / `rig3d` | 若设备带 Live2D/3D 运行时则消费；否则映射到预设舵机序列 |
| `procedural` | 自定义程序化动画 |

启用条件：角色包 `config.json` → `portrait_catalog` + `visual_presentation.enabled`；发行版 `distro.oclive.toml` → `[visual_presentation].mode` 非 `off`。  
详见 [`creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md`](creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md)。

**若未启用**：仍可用 `bot_emotion` / `portrait_emotion` 做简单表情表（七类 → 舵机预设）。

### 5.3 流式对话（降低首字延迟）

`POST /chat/stream`：SSE 事件流，适合边收 token 边送 TTS 引擎。  
集成测参考：`examples/oocp-test-suite` 场景 S15。

---

## 6. 模型与推理（设备侧）

OClive **不自带模型权重**；`llm` 槽可换：

| 后端 | 配置 | 适用 |
|------|------|------|
| **ollama** | 蓝图 `plugin_backends.llm = ollama` + 本机 `ollama pull` | 设备有足够 RAM/显存 |
| **remote** | 侧车 JSON-RPC + `OCLIVE_REMOTE_LLM_URL` | 模型跑在网关/手机 |
| **directory** | 目录插件实现 `llm.generate` | 如 [`examples/directory-plugin-llamacpp`](examples/directory-plugin-llamacpp/README.md) |

用户向云端 Key：[`creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md`](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)。

**玩偶 / 陪伴类产品建议**：

- **本地小参 + 微调/Modelfile** 锁口吻与尺度，OClive 负责记忆与情绪连续。  
- **短回复、短回合**；长剧情对硬件是负担。  
- 区分 **陪伴模式**（记用户、情绪连续）与 **短场景模式**（预设情境、回合上限），见产品讨论；硬件上可共用一个 `POST /chat`，用不同 `scene_id` / 角色包区分。

---

## 7. 灵魂数据：角色包与机器人最小包

**灵魂** = 可版本化的角色包目录，不是写死在固件里。

| 组成 | 路径 / 说明 |
|------|-------------|
| 蓝图 SSOT | `pipeline.ocblueprint`（`meta` + `slot_registry`） |
| 人设与 prompt | `prompts/`、`meta.personality`、`meta.relations` |
| 立绘/表现 | `portrait_catalog.json`、`config.json` → `visual_presentation` |
| 校验 | `cargo run -p oclive-cli -- pack validate … --profile robot-soul` |

机器人最小示例：[`examples/robot-soul-minimal`](examples/robot-soul-minimal/README.md)。  
规范：[`creator-docs/role-pack/ROLE_PACK_SPEC.md`](creator-docs/role-pack/ROLE_PACK_SPEC.md)。

**换灵魂**：更新 `roles/{id}/` 或改 `OCLIVE_ROLES_DIR`，`role_path` 指向新目录即可，**无需重刷内核固件**。

---

## 8. 发行版配置（硬件 OEM 定制）

商用硬件可自备 **`distro.oclive.toml`**，与官方桌面/剧场发行版 **品牌分离**：

- 示例目录：[`examples/distro-profiles/`](examples/distro-profiles/README.md)  
- 契约：[`creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md`](creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)  
- 启动时注入：`OCLIVE_DISTRO_ID`、`OCLIVE_DISTRO_PROFILE`  
- `/health` 返回 `distro_id`、`active_profile_summary` 便于中控识别

可配置项包括：记忆检索条数、用户身份默认、`[visual_presentation].mode`、`[post_process]` 链等（**不是**六槽本身，六槽仍在角色包蓝图）。

---

## 9. 外设扩展：目录插件与 MCP

电机、灯效、专有传感器等 **高风险或进程型** 能力，推荐 **目录插件**：

- 文档：[`creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)  
- 最小范例：[`examples/directory-plugin-minimal`](examples/directory-plugin-minimal/README.md)  
- 首次 `process:spawn` 需用户授权 **`high_risk_grants.json`**（`process:spawn`）  
- HTTP 调插件：`POST /bridge/dispatch`（进阶）

**分工建议**：

- 简单舵机：硬件固件直接映射 `performance_directive` / `bot_emotion`  
- 复杂外设（多轴、安全联锁）：独立插件进程 + JSON-RPC

---

## 10. 数据、隐私与部署

| 项 | 说明 |
|----|------|
| 默认存储 | 本机 SQLite `{OCLIVE_APP_DATA}/app.db` + 聊天/记忆表 |
| 出站 | 仅当 `llm=remote` 或目录插件声明 `network:*` 时离开本机 |
| 免责声明 | [`creator-docs/legal/DISCLAIMER.md`](creator-docs/legal/DISCLAIMER.md) |
| 许可证 | 内核 **Apache-2.0**；硬件固件与垂直应用可闭源商用（见 [`creator-docs/LICENSE_POLICY.md`](creator-docs/LICENSE_POLICY.md)） |

量产建议在设备上固定 `OCLIVE_APP_DATA` 路径并做好 **恢复出厂**（删库即清记忆，需在硬件 UI 说明）。

---

## 11. 边界与常见误解

| 误解 | 事实 |
|------|------|
| OClive = 完整硅胶玩偶方案 | **否**；仅灵魂运行时，TTS/ASR/电机你们做 |
| 接一个大模型就能稳人设 | **否**；需角色包 + OClive 记忆/情绪；模型只管生成 |
| 必须长剧情才像陪伴 | **否**；玩偶类多为 **短回合 + 状态连续** |
| HTTP 可公网直连 | 当前 **仅 127.0.0.1**；公网需网关与鉴权 |
| 官方 OClive 提供成人/玩偶成品 | **否**；Apache-2.0 下由 **第三方垂直发行版** 自行负责内容与合规 |

---

## 12. 推荐集成里程碑

| 阶段 | 交付 | 验收 |
|------|------|------|
| **M0** | 同机跑通 `/health` + `POST /chat`，TTS 播 `reply` | 30 分钟内对话 |
| **M1** | `bot_emotion` → 表情/舵机预设表 | 3 种情绪可见变化 |
| **M2** | 启用 `portrait_catalog` + `performance_directive` 映射 | 与角色包立绘一致 |
| **M3** | 本地 Ollama/微调模型 + 固定 `session_id` 记忆 | 跨轮记得称呼 |
| **M4** | 自备 `distro.oclive.toml` + OEM 角色包 | `/health` 带 `distro_id` |
| **M5** | 延迟优化：`/chat/stream` + 唤醒词 + idle | 首字 &lt; 2s 量级（视硬件） |

---

## 13. 文档索引

| 主题 | 路径 |
|------|------|
| 无头最小闭环 | [`examples/headless-kernel-minimal/README.md`](examples/headless-kernel-minimal/README.md) |
| 纯净内核边界 | [`creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md`](creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) |
| 平台开发者路径 | [`creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md`](creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md) |
| 角色包规范 | [`creator-docs/role-pack/ROLE_PACK_SPEC.md`](creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 六槽与插件 | [`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 目录插件 | [`creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md`](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| 数据目录 | [`creator-docs/kernel/OCLIVE_APP_DATA.md`](creator-docs/kernel/OCLIVE_APP_DATA.md) |
| 多轮巡检 / 愿景对齐 | [`handoff/RECURRING_OPTIMIZATION_PLAYBOOK.md`](handoff/RECURRING_OPTIMIZATION_PLAYBOOK.md) |

---

## 14. 联系与协作说明

- **内核/契约问题**：主仓 Issue（附 `/health` JSON、`POST /chat` 请求与错误 `code`）。  
- **硬件垂直品**：由设备厂商自行维护固件与合规；OClive 维护者仅保证内核 API 与角色包契约稳定（Breaking 流程见 [`handoff/BREAKING_CHANGE_PROCESS.md`](handoff/BREAKING_CHANGE_PROCESS.md)）。  
- **今晚演示建议顺序**：§0 提纲 → §1 架构图 → §3 现场 curl → §4 主循环 → §5.1 字段表 → §12 M0–M2 里程碑。

---

*本文档面向硬件对接；产品战略与审查维度见 `handoff/RECURRING_OPTIMIZATION_PLAYBOOK.md` 前置区。*
