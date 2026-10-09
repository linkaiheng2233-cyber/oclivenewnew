# 六槽开工包 · `llm`

> **读者**：编写基础文本生成实现，或维护参考 Host 的 Ollama / remote / directory 后端的作者。
> **读完能做什么**：选择 Base 生成或丰富后端路径，找到消费者和既有 Host 客户端适配。
> **耗时**：基础入口为短导航；参考 Host 流程约 **45 min**
> **SSOT 范围**：人类路由与 checklist；公共接入点见 [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，参考 Host 定义见 [§8](../../../handoff/MODULE_MAP_AND_HANDOFF.md#8-第-5-模块--llm)
> **最后更新**：2026-10-09
> **下一篇**：[agent](agent.md) · [plugin-author 路径](../../paths/plugin-author.md)

---

## 0. 先选接入路径

基础文本生成从下方开始；改 `LlmClient`、流式传输、目录插件或蓝图后端则读 **§1–§6**。后者的配置与产品验收属于参考 Host 丰富路径，不自动成为所有 Base 实现的前置要求。

### LlmBase：复用调用方选定的生成能力

1. 在[公共契约](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs)找到 `LlmBase::generate`。实现接收已准备的输入；模型、客户端与资源授权由调用方装配，不由请求文本授予。
2. 参考 Host 已提供 [`OcliveKernel::text_generation_base()`](../../../kernel/crates/oclive_kernel_host/src/role_kernel.rs)，其内部 [`HostTextGenerationBase`](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/minimal_llm.rs) 借用既有客户端，不重新装配 provider。runtime 没有单独的 `base_llm.rs`；独立 Host 可以实现公开 trait，不需要复制参考 Host。
3. 看[`MinimalRoleTextConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs)的显式 Prompt→LLM 文本操作，或六 Base 消费者的独立 `generate` 委托。前者在 Prompt 正常完成后才调用 LLM；后者不会隐式先调 Prompt。现有内存装配例子见 [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs)。

**基础路径 checklist**：

- [ ] 选定实现与输入约定相容；正常空文本和完整失败保留，不按诊断字样伪造超时或取消分类。
- [ ] 资源、授权和执行器由调用方明确；丢弃 future 不当作远端停止或可安全重试的证据。

请求、失败和绑定查[接入点清单](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。基础文本结果不承诺 SSE、统一 remote wire 或真实模型质量；这些按具体 Host 与 provider 另验。

---

## 1. 参考 Host：你插在哪

- **MODULE_MAP**：[§8 第 5 模块 · `llm`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#8-第-5-模块--llm)
- **当前配置 / 运行时折叠**：蓝图 `slot_registry.type: llm` → `PluginBackends.llm`；legacy v1 才是 `settings.json.plugin_backends.llm`
- **Trait**：`LlmClient`（`oclive_kernel_contracts`）
- **主链 hook**：`co_present` generate / stream（经 `slot_runner`）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| Ollama 适配、`directory` RPC、remote JSON-RPC、TTFT 客户端选项 | UI 内二次调 LLM 选立绘；共景路径 `none` backend |
| 蓝图 `slot_registry` 中 `type: llm` 的 backend 声明 | 角色任务改 `slot_registry` 结构（G1） |
| 多 `llm` 实例时理解非流式 `ensemble` / `fastest` / `fallback` 与流式串行限制 | 把 LLM 逻辑写进 `distros/desktop-tauri/src/api/*.rs` |

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
- [ ] 目录插件：源码仓 `distros/chat-pro/plugins/` / 用户安装 `{app_data}/plugins/` · 权限 `network:*` 须授权
- [ ] `npm run check` 绿；可选 `cargo test` 相关 invoke 热路径

---

## 5. 验收

- [ ] 一轮共景对话能 stream / 非 stream 拿到 **`reply`** 字段
- [ ] 多 llm 实例时，非流式行为符合最后一个 LLM 实例声明的 `policy`；流式行为符合当前串行 last-wins 限制（见 MODULE_MAP §3.3）
- [ ] 未在 Vue 层绕过槽位直连接模型 API
- [ ] PR 描述链 MODULE_MAP §8，未粘贴 24 格矩阵全文

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `prompt` | 上游组装完整 prompt 字符串 |
| `event` | Fast 轮 Turn Thinking 可能影响 event LLM 路径开关（HostProfile，非 llm 槽） |
| `agent` | Agent 短路时可能不再调用 llm |
