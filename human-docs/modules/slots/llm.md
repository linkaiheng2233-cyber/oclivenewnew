# 六槽开工包 · `llm`

> **读者**：开发 Ollama / remote / directory LLM 后端的插件作者。  
> **读完能做什么**：在不读 `process_message` 全文的前提下，接入或调试 `llm` 槽。  
> **耗时**：约 **45 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §8](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-07-14
> **下一篇**：[agent](agent.md) · [plugin-author 路径](../../paths/plugin-author.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§8 第 5 模块 · `llm`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#8-第-5-模块--llm)
- **`plugin_backends` 键**：`llm`  
- **Trait**：`LlmClient`（`oclive_kernel_contracts`）  
- **主链 hook**：`co_present` generate / stream（经 `slot_runner`）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| Ollama 适配、`directory` RPC、remote JSON-RPC、TTFT 客户端选项 | UI 内二次调 LLM 选立绘；共景路径 `none` backend |
| 蓝图 `slot_registry` 中 `type: llm` 的 backend 声明 | 角色任务改 `slot_registry` 结构（G1） |
| 多 `llm` 实例时理解 **last-wins** 合并 | 把 LLM 逻辑写进 `distros/desktop-tauri/src/api/*.rs` |

---

## 3. 阅读清单

1. [MODULE_MAP §8](../../../handoff/MODULE_MAP_AND_HANDOFF.md#8-第-5-模块--llm)
2. [PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) — `send_message` 中 llm 阶段顺序
3. [DIRECTORY_PLUGINS](../../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) · [REMOTE_PLUGIN_PROTOCOL](../../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)
4. [SLOT_BACKEND_REALITY_MATRIX](../../../handoff/SLOT_BACKEND_REALITY_MATRIX.md) — `llm` 行真值
5. 示例：[`examples/directory-plugin-minimal/`](../../../examples/directory-plugin-minimal/)

---

## 4. 开发流程

- [ ] L0–L3 + [02 跑通](../../02_THIRTY_MINUTE_START.md)  
- [ ] 选定 backend：`ollama` · `remote` · `directory`（BYOK / 用户 LLM 设置见 HostProfile）  
- [ ] 实现 `LlmClient` 或目录插件 manifest 声明 `llm` capability  
- [ ] 蓝图或 legacy `plugin_backends` 指向你的 backend  
- [ ] 目录插件：`{app_data}/distros/chat-pro/plugins/` · 权限 `network:*` 须授权  
- [ ] `npm run check` 绿；可选 `cargo test` 相关 invoke 热路径

---

## 5. 验收

- [ ] 一轮共景对话能 stream / 非 stream 拿到 **`reply`** 字段  
- [ ] 多 llm 实例时行为符合 last-wins（见 MODULE_MAP §3.3）  
- [ ] 未在 Vue 层绕过槽位直连接模型 API  
- [ ] PR 描述链 MODULE_MAP §8，未粘贴 24 格矩阵全文

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `prompt` | 上游组装完整 prompt 字符串 |
| `event` | Fast 轮 Turn Thinking 可能影响 event LLM 路径开关（HostProfile，非 llm 槽） |
| `agent` | Agent 短路时可能不再调用 llm |
