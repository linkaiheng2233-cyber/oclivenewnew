# 六槽 × 后端真实性矩阵（24 格 · 只读巡检）

**Last updated:** 2026-07-18（低风险债与愿景对标复核）
**证据源：** `kernel/crates/oclive_kernel_host/src/infrastructure/backend_registry.rs`（`*_for_plugin_backends`）+ `directory_slots_impl.rs`

> 表为 6×4=24 物理格；memory / emotion / event / prompt 四槽 **无独立 V2 实现**（D-SLOT-01 已完成）。`builtin_v2` 仅为 serde 读兼容 alias，行为等同 `builtin`。

图例：✅ 真跑通 · ⚠️ 占位/静默回退 · ❌ 未实现

| 槽 | builtin | remote | directory | none |
|----|---------|--------|-------------|------|
| **memory** | ✅ `BuiltinMemoryRetrieval` | ⚠️ HTTP；缺 endpoint 有启动警告并解析为 builtin；调用失败是否降级受全局策略控制 | ✅ `RemoteMemoryRetrievalHttp` + spawn | ✅ `NoopMemoryRetrieval` |
| **emotion** | ✅ `BuiltinUserEmotionAnalyzer` | ⚠️ 同上 | ✅ `RemoteUserEmotionAnalyzerHttp` | ✅ `NoopUserEmotionAnalyzer` |
| **event** | ✅ `BuiltinEventEstimator` | ⚠️ 同上 | ✅ `RemoteEventEstimatorHttp` | ✅ `NoopEventEstimator` |
| **prompt** | ✅ `BuiltinPromptAssembler` | ⚠️ 同上 | ✅ `RemotePromptAssemblerHttp` | ✅ `NoopPromptAssembler` |
| **llm** | ✅ Ollama（`llm_ollama`） | ⚠️ `RemoteLlmHttp`；失败是否 fallback 受全局策略控制 | ✅ `RemoteLlmHttp` directory RPC | ✅ `NoopLlmClient` |
| **agent** | ✅ `BuiltinReActAgent` | ✅ `AgentRpcProvider` + fallback | ✅ directory + `FallbackAgentProvider` | ✅ `NoopAgentProvider` |

**注：** `memory.local` 走 `LocalPluginMemoryRetrieval`，委托 builtin 排序；`complex_emotion` 为设施子模块（非六槽），见 `pick_complex_emotion_*`。

**兼容边界：** `builtin_v2` 字符串经 serde alias 读入为 `builtin`；前端显示/保存也会归一化。它不是可选的第二套内置模块。
