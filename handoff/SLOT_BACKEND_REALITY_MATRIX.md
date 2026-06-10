# 六槽 × 后端真实性矩阵（24 格 · 只读巡检）

**Last updated:** 2026-06-10（轮次 10 半档巡检）  
**证据源：** `crates/oclive_kernel_host/src/infrastructure/backend_registry.rs`（`pick_*` / `*_for_plugin_backends`）

图例：✅ 真跑通 · ⚠️ 占位/静默回退 · ❌ 未实现

| 槽 | builtin | remote | directory | none |
|----|---------|--------|-------------|------|
| **memory** | ✅ `BuiltinMemoryRetrieval` | ⚠️ HTTP 客户端；缺 env 时静默回退 builtin | ✅ `RemoteMemoryRetrievalHttp` + spawn | ✅ `NoopMemoryRetrieval` |
| **emotion** | ✅ `BuiltinUserEmotionAnalyzer` | ⚠️ 同上 | ✅ `RemoteUserEmotionAnalyzerHttp` | ✅ `NoopUserEmotionAnalyzer` |
| **event** | ✅ `BuiltinEventEstimator` | ⚠️ 同上 | ✅ `RemoteEventEstimatorHttp` | ✅ `NoopEventEstimator` |
| **prompt** | ✅ `BuiltinPromptAssembler` | ⚠️ 同上 | ✅ `RemotePromptAssemblerHttp` | ✅ `NoopPromptAssembler` |
| **llm** | ✅ Ollama（`llm_ollama`） | ⚠️ `RemoteLlmHttp` + fallback | ✅ `RemoteLlmHttp` directory RPC | ✅ `NoopLlmClient` |
| **agent** | ✅ `BuiltinReActAgent` | ✅ `AgentRpcProvider` + fallback | ✅ directory + `FallbackAgentProvider` | ✅ `NoopAgentProvider` |

**注：** `memory.local` 走 `LocalPluginMemoryRetrieval`，委托 builtin 排序；`complex_emotion` 为设施子模块（非六槽），见 `pick_complex_emotion_*`。

**本轮变更：** 删除四槽 `builtin_v2` 测试桩（D-SLOT-01）；`builtin_v2` 字符串经 serde alias 读入为 `builtin`。
