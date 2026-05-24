# kernel_contracts trait 方法审计（2026-05-20）

**范围**：`oclive_kernel_contracts` 根导出 trait 的每个方法在 `oclivenewnew-tauri` / `oclive_kernel_runtime` 中的引用情况。

**结论**：无废弃删除项；带默认实现的方法为可选扩展点，已在 trait 文档中标注。

| Trait | 方法 | 调用方 | 说明 |
|-------|------|--------|------|
| `AgentProvider` | `process` | `domain/agent.rs`, `plugin_host`, `slot_resolver` | 主编排 |
| `EventEstimator` | `estimate` | `slot_runner`, `event_impact_ai`, remote HTTP | 主编排 |
| `LlmClient` | `generate` | 全仓 | 主编排 |
| `LlmClient` | `generate_tag` | `portrait_emotion_engine` | 立绘标签 |
| `LlmClient` | `startup_probe` | `startup_health`（默认体成功） | 可选探活；默认实现保留 |
| `PluginHostPort` | `resolve_for_role` / `resolve_for_effective_backends` | `plugin_host`, `chat_engine` | 主编排 |
| `SlotRegistryResolver` | `resolve` | `slot_resolver` | 蓝图 v2 |
| `MemoryRepository` | `save_memory` 等 | `infrastructure/db` | 持久化 |
| `FavorabilityRepository` | `get` / `apply_delta` | `infrastructure/db` | 持久化 |
| `MemoryRetrieval` | `rank_memories` / `build_context` / `search_memories` | `memory_retrieval`, `slot_runner` | 主编排 |
| `MemoryRetrieval` | `diagnostic_local_provider_id` | 测试 / 遥测（默认 `None`） | 可选诊断 |
| `ComplexEmotionProvider` | `resolve_turn` | `co_present`, remote | 主编排 |
| `PromptAssembler` | `build_prompt` | `prompt_builder`, `slot_runner` | 主编排 |
| `PromptAssembler` | `top_topic_hint` | `chat_engine/scene` | 场景话题 |
| `UserEmotionAnalyzer` | `analyze` | `emotion_analyzer`, `slot_runner` | 主编排 |
| `EmotionPolicy` | `resolve_current_emotion` | `state`, policy engines | 主编排 |
| `EventPolicy` | `detect` / `impact` / `confidence` | `event_engine`, `slot_runner` | 主编排 |
| `MemoryPolicy` | `build_memory_entry` 等 | `memory_engine` | 主编排 |
| `LocalPluginBridge` | `bridge_name` / `discover_providers` | `local_plugin` 发现 | 主编排 |
