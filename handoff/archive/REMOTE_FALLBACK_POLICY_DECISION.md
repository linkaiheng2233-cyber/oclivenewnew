# 架构决策：Remote 失败兜底可配置（对齐 A4 与发行版/硬件需求）

**状态**：**已实现（宿主 v0.2 补丁）**。与 **network:\*** 授权（能否出站）正交；本决策管 **出站调用失败后是否静默改用内置实现**。

---

## 1. 当前实现（可配置兜底）

当 `plugin_backends.* = remote` 且配置了远端 HTTP 客户端时，由 **`app_settings.remote_fallback_to_builtin`**（默认 `"1"`）与 **`OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`** 环境变量决定：允许降级时失败分支仍走内置并打 `tracing::warn`；**关闭**时返回 **`AppError::RemoteServiceUnavailable`**（`code` = **`REMOTE_SERVICE_UNAVAILABLE`**），不再静默替代。

| 模块 | 典型位置 |
|------|-----------|
| 情绪 / 事件 / 记忆排序 / Prompt / 复杂情感 | `src-tauri/src/infrastructure/remote_plugin/{emotion,event,memory,prompt,complex_emotion}_http.rs` |
| Remote LLM 占位 | `src-tauri/src/infrastructure/llm.rs` 的 `RemoteLlmPlaceholder`（未接 `OCLIVE_REMOTE_LLM_URL` 或侧车客户端构建失败时） |
| 进程内开关 | `src-tauri/src/infrastructure/remote_fallback_policy.rs` + `AppState::remote_fallback_allowed`；设置页与 `update_settings`（plugin bridge）可写库并同步 |

**相关但不同**：`plugin_backends.* = directory` 时，`plugin_host` 在 `ensure_rpc_url` 失败等处 **回退 Ollama / builtin**（日志路径独立）。本决策以 **Remote HTTP 槽位** 为第一目标；目录槽位可在同一 **「远端类失败策略」** 配置模型下二期对齐。

---

## 2. 问题陈述

1. **终端用户**：配置了远端，以为一直在用「高级能力」，远端不可达时实际在用内置规则，**无显式提示**，易产生「模型变笨」感受。  
2. **纯净内核 / 硬件集成**：需要 **失败即失败**，便于确定性测试与现场排障；静默降级掩盖根因。  
3. **与 A4 的关系**：**network:\***（及 MCP 等）解决 **「是否允许尝试联网」**；未解决 **「联网尝试失败后是否允许静默换实现」** — 本决策补全后半段。

---

## 3. 决策摘要

- **兜底不再由内核写死为「永远降级」**，改为：  
  - **发行版 / 脚手架可配置默认**；  
  - **官方桌面应用：终端用户可选**（高级设置项）。  
- **默认行为**：与现网一致 — **允许自动降级到内置**（桌面场景推荐）。  
- **关闭兜底**：Remote 调用链在远端失败时 **返回错误**（沿用/扩展 `KernelErrorBody` 等契约），**不调**内置替代路径（或仅记录诊断后仍返回错误，由产品定义）。

### 3.1 `oclive init` 脚手架（建议文案）

交互或 `--preset` 等价选项：

```text
? 远端服务失败时的行为:
  ● 自动降级到内置实现（推荐：适合桌面应用）
  ○ 返回错误，不做降级（适合硬件集成/调试）
  ○ 自定义（按槽位分别设置）  # 可列为 Phase 2
```

生成物写入 **单一配置源**（例如生成项目 `settings` 片段或 `monolith.toml` / 内核策略文件），与 Tauri 宿主 **同一键名**，避免分叉。

### 3.2 官方发行版 UI

- **路径**：设置 → 高级 → **「远端服务失败时自动使用本地实现」**  
- **默认**：**开启**（保持当前行为）。  
- **关闭**：Remote 路径失败 → **用户可见错误**（toast / 对话内说明 + 稳定 `code`），不静默切 builtin。

### 3.3 与 **network:\*** 授权的分工（两步独立、协同）

| 步骤 | 检查内容 | 用户可见（典型） |
|------|-----------|------------------|
| 1 | **network:\***（及同类）：该槽位/场景是否允许发起出站请求 | 未授权 → 不请求 / 明确拒绝理由 |
| 2 | **Remote 调用**：在已允许出站的前提下，远端是否成功 | `remote_fallback=true` → 可降级 builtin；`false` → 返回错误码与文案 |
| 3 | **可观测性** | 兜底关闭且失败 → 明确错误；兜底开启且降级 → 可选「已使用本地规则」轻提示（产品化迭代） |

**原则**：授权管 **入口**（能不能试）；兜底策略管 **出口**（试了以后失败怎么办）。

---

## 4. 收益（与决策表一致）

| 维度 | 说明 |
|------|------|
| 用户选择权 | 可明确选择「失败要我知道」vs「失败帮我扛」 |
| 硬件 / 嵌入式 | 关闭兜底 → 行为确定，便于认证与日志分析 |
| 包体（脚手架） | 关闭兜底且 monolith 裁剪时，可不再链接部分 `*_builtin` 路径（与 RFC monolith 路线对齐时再细化依赖图） |
| A4 闭环 | **network:\*** + **`remote_fallback`（命名待定）** 覆盖「能否连」与「失败后怎么办」 |

---

## 5. 实现时建议（供开发拆 issue）

1. **配置模型**：在 `settings`（或 `PolicyRegistry` 扩展）中增加全局布尔 +（Phase 2）按槽位覆盖；默认值 `true`。  
2. **注入点**：在 `Remote*Http` 的 `Err` 分支根据策略 **分支**：降级 vs `return Err(...)`。LLM `RemoteLlmPlaceholder` 与 `RemoteLlmHttp` 内部 generate 失败路径需一并纳入。  
3. **DTO / 迁移**：新键默认值 `true`，旧安装升级无行为变化。  
4. **文档**：`PLUGIN_V1.md` / `ERROR_CODES` 补充错误码与设置项说明；`oclive-cli` 文档同步交互选项。

---

## 6. 命名备忘

文中 **`remote_fallback`** 为语义占位；落地时可选用 `remote_degrade_to_builtin`、`plugin_backends_remote_failure_policy` 等与现有 `settings.json` 风格一致的键名，并在 `crates/oclive_validation` 与 creator 文档中同步。
