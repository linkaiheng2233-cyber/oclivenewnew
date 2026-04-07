# 创作者架构指南（完整版）

本文是 **oclive 可替换子系统**的创作者向说明：如何在**不改宿主**或**fork 宿主**的前提下扩展能力；如何配置 **HTTP 侧车**；以及「本地替换模块」「线上更新逻辑」在工程上的**真实含义**。

**文档索引（全库导航）**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**JSON-RPC 字段与完整示例**：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)  
**settings 枚举契约**：[PLUGIN_V1.md](PLUGIN_V1.md)  
**Rust 替换步骤**：[HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)

---

## 第一部分：架构在解决什么问题

oclive 把对话管线拆成可替换块：**记忆检索、用户句情绪、事件估计、Prompt 组装、LLM 调用**。角色包通过 `settings.json` → `plugin_backends` 声明每块用 **builtin / builtin_v2 / remote / ollama** 等（见 PLUGIN_V1）。

- **builtin**：逻辑编译在宿主内，稳定、离线友好。  
- **remote**：逻辑可在**独立 HTTP 服务（侧车）**中实现，宿主只发 JSON-RPC，按约定解析结果。  
- **llm: ollama**：使用应用启动时注入的本地/兼容 **Ollama** 客户端。  
- **llm: remote**：使用 **`OCLIVE_REMOTE_LLM_URL`** 指向的 JSON-RPC（`llm.generate` / `llm.generate_tag`）。

这样创作者可以：  
- 只写**角色包**（剧本、场景、人设）；或  
- 自建**侧车**（Python/Node/Go 等）实现自定义记忆排序、网关大模型、自定义 Prompt 策略；或  
- **Fork 仓库**改 Rust，在 `PluginHost` 注册新的编译期后端。

---

## 第二部分：三种扩展方式（对照表）

| 方式 | 你需要准备什么 | 何时生效 | 「热更新」在工程上的含义 |
|------|----------------|----------|---------------------------|
| **A. 角色包** | `roles/{角色id}/` 下 manifest、settings、场景、文案等 | 保存后由应用 **`load_role`**（或你们提供的重载）加载 | 更新内容**无需重编译宿主**；对话逻辑仍由**内置引擎**执行，除非该角色显式使用 remote |
| **B. HTTP 侧车** | 可访问的 URL + 实现 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) 中的 **method** | 启动应用**前**设置环境变量；角色包 `plugin_backends.* = remote` | **更新侧车进程/容器**即可换新逻辑，**桌面应用可不重新编译**；需保持 JSON-RPC **向后兼容** |
| **C. Fork 改宿主（Rust）** | Rust 工具链；在 `domain` / `PluginHost` / `plugin_backends` 注册新枚举与实现 | `cargo build` / 发布**新安装包** | **不是**进程内动态换 `.dll`/插件；发新版 exe 才算替换宿主模块 |

**选型建议**

- 只想写剧本与人设 → **A**。  
- 希望「线上改 AI 策略/网关/记忆算法、用户不用下新版桌面端」→ **B**。  
- 要改引擎内核、性能路径、新枚举分支 → **C**。

---

## 第三部分：HTTP 侧车 — 你需要准备什么

### 3.1 环境变量（最终用户在宿主机器上设置）

| 变量 | 是否必填 | 含义 |
|------|----------|------|
| `OCLIVE_REMOTE_PLUGIN_URL` | 想用 **memory/emotion/event/prompt** 的 remote 时 **必填** | 单个 **POST** 端点 URL；四类共用，靠 `method` 区分 |
| `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS` | 否 | 默认 `8000`（毫秒） |
| `OCLIVE_REMOTE_PLUGIN_TOKEN` | 否 | `Authorization: Bearer …` |
| `OCLIVE_REMOTE_LLM_URL` | `plugin_backends.llm = remote` 时 **必填**（否则回退进程内 LLM 并警告） | **LLM** 专用端点 |
| `OCLIVE_REMOTE_LLM_TIMEOUT_MS` | 否 | 默认 `120000` |
| `OCLIVE_REMOTE_LLM_TOKEN` | 否 | Bearer |

端点必须是**完整 URL**（含 `http://`/`https://` 与路径），例如：`http://127.0.0.1:8765/rpc`。

### 3.2 角色包 `settings.json`

在 `plugin_backends` 中为要交给侧车的子系统设为 **`remote`**，其余可保持 `builtin` 或 `ollama`：

```json
{
  "schema_version": 1,
  "plugin_backends": {
    "memory": "remote",
    "emotion": "remote",
    "event": "remote",
    "prompt": "remote",
    "llm": "remote"
  }
}
```

若环境变量未设置对应 URL，宿主会**回退内置实现**并可能记录警告；**不会**仅因缺侧车而崩溃。

### 3.3 JSON-RPC 方法清单（侧车必须实现的方法名）

宿主客户端会调用下列 **`method`**（完整 params/result、JSON 示例见协议文档）：

| method | 用途 |
|--------|------|
| `memory.rank` | 返回记忆 `id` 的排序 |
| `emotion.analyze` | 返回七维 `EmotionResult` |
| `event.estimate` | 返回 `EventImpactEstimate`（**`event_type` 的 JSON 形状见协议 §3**） |
| `prompt.build_prompt` | 返回主 prompt 字符串 |
| `prompt.top_topic_hint` | 可选；话题提示 |
| `llm.generate` | 主生成 |
| `llm.generate_tag` | 短标签生成 |

**重要**：`event.estimate` 的 `event_type` 必须使用 Rust **serde 默认枚举编码**（外部标签对象），例如 `Ignore` → `{"Ignore": null}`，**不能**写成裸字符串 `"Ignore"`。详见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) 第三节。

---

## 第四部分：联调步骤（从 0 到通）

1. **启动参考侧车**（仓库内仅用于开发演示）：  
   - 目录：[examples/remote_plugin_minimal/README.md](../examples/remote_plugin_minimal/README.md)  
   - 默认监听示例 URL（以该 README 为准）。  

2. **设置环境变量**后再启动 oclive：

**PowerShell（Windows）示例**

```powershell
$env:OCLIVE_REMOTE_PLUGIN_URL = "http://127.0.0.1:8765/rpc"
$env:OCLIVE_REMOTE_LLM_URL = "http://127.0.0.1:8765/rpc"
# 然后启动应用（如 npm run tauri:dev）
```

**bash 示例**

```bash
export OCLIVE_REMOTE_PLUGIN_URL="http://127.0.0.1:8765/rpc"
export OCLIVE_REMOTE_LLM_URL="http://127.0.0.1:8765/rpc"
```

3. 将测试角色 `settings.json` 中需要走侧车的项设为 `remote`，**加载角色**后发一条消息。  

4. 观察侧车日志与宿主日志（过滤 `oclive_plugin`）确认请求到达。

---

## 第五部分：本地「写模块替换」的含义

| 说法 | 实际做法 |
|------|----------|
| **替换内置 Rust 模块** | Fork 仓库 → 实现 trait → 在 `PluginHost` / `plugin_backends` 注册 → **重新编译发布宿主** |
| **不编译宿主，只换业务逻辑** | 实现 HTTP 侧车 → 配置环境变量 + `plugin_backends` → **滚动发布侧车** |

---

## 第六部分：「线上热更新」边界（避免误解）

| 目标 | 是否可行 | 做法 |
|------|----------|------|
| 更新侧车里的模型路由、Prompt 策略、记忆算法 | ✅ | 部署新侧车版本，保持 JSON-RPC 兼容 |
| 更新角色台词、场景、人设 | ✅ | 更新角色包文件 + 用户 **`load_role`** |
| **不重启 oclive** 且 **不换安装包**，替换 **已编译进 exe 的 Rust 逻辑** | ❌（当前架构） | 无动态 `cdylib` 热插拔；可变部分应放侧车或角色包 |

---

## 第七部分：常见问题（排错）

| 现象 | 可能原因 |
|------|----------|
| 仍走内置、日志提示 remote 未连接 | 未设置 `OCLIVE_REMOTE_*` URL，或 URL 拼写错误 |
| `event.estimate` 总回退内置 | `result.event_type` 用了裸字符串，应为 `{"Ignore": null}` 等形式 |
| LLM 仍像本机 Ollama | `llm` 仍为 `ollama`，或未设 `OCLIVE_REMOTE_LLM_URL` |
| 请求未到侧车 | 防火墙、HTTPS 证书、URL 非 POST 可达、侧车未监听同机地址 |

更细的 HTTP/JSON 形状见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。

---

## 第八部分：源码与文档索引

| 内容 | 路径 |
|------|------|
| 宿主聚合 | `src-tauri/src/domain/plugin_host.rs` |
| Remote HTTP 客户端 | `src-tauri/src/infrastructure/remote_plugin/` |
| 运行时解析 | `AppState::resolved_plugins_for` — `src-tauri/src/state/mod.rs` |
| 全库文档导航 | [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md) |

---

## 第九部分：与相关文档的关系

- **角色包怎么写**： [../getting-started/CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)、[roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)  
- **枚举与默认值**： [PLUGIN_V1.md](PLUGIN_V1.md)、[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)  
- **只关心替换 Rust 模块步骤**： [HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)  
