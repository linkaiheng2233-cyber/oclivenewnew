# 设施 crate `classic` 算法审计（任务 B）

> 日期：2026-05（与实现同步）  
> 范围：`oclive_memory_builtin`、`oclive_emotion_builtin`、`oclive_complex_emotion_builtin`、`oclive_agent_builtin`  
> 关联实现：`各 crate/Cargo.toml` 的 **`classic`** feature；`oclive_kernel_runtime` 的 **`facility-classic-algorithms`**。

---

## 1. 总表

| 设施 crate | 含 `classic` 模块？ | 被调用方（直接或间接） | 可否 feature 门控 | 备注 |
|------------|---------------------|-------------------------|---------------------|------|
| **oclive_memory_builtin** | 是（`classic/`：`full` / `stub`） | `BuiltinMemoryRetrieval*`（`providers`）；`oclive_kernel_runtime`：`memory_engine`、`disabled_default_providers`、`remote_plugin/memory_http` | **已实施**：`classic` 默认开；关时用桩（`get_relevant_memories` 为 FIFO，搜索/上下文与完整版一致） | 纯算法、无文件 I/O；**不适用**迁移到 `tokio::fs` / `spawn_blocking` |
| **oclive_emotion_builtin** | 是 | `BuiltinUserEmotionAnalyzer*`（`providers`）；`kernel_runtime`：`emotion_analyzer`（重导出）、`user_emotion_analyzer` 单测、`co_present` / `role_manager` 经 `EmotionAnalyzer` | **已实施**：关时强中性桩（无关键词表） | 同上，纯 CPU |
| **oclive_complex_emotion_builtin** | 是 | `kernel_runtime`：`complex_emotion` 重导出 `affect_metrics_from_seven_dim` → `process_message`、`co_present`；**未**被本 crate `providers` 直接调用 | **已实施**：关时恒 `(0.0, 0.0)` | 体量小；门控主要为与情绪桩组合时语义一致 |
| **oclive_agent_builtin** | **否** | — | **无可裁剪对象** | 无 `classic` 目录；**阶段 7 待定**仅当未来引入与上述同类的「可拆算法库」时再审计 |

---

## 2. 依赖性质说明

- **编排层**：`process_message` / `MemoryEngine` 等依赖 **`classic` 的公开 API 形状不变**（函数签名与 `EmotionAnalyzer` 方法集不变），以便侧车 / Remote 回退与单测不因 feature 分叉而改接口。
- **侧车对齐**：`memory_http` 等路径依赖 `oclive_memory_builtin::classic` 的 `build_context` / `search_memories`；桩在 **`search` / `build_context`** 上与完整版行为一致，避免 HTTP 层行为漂移。
- **单测**：依赖完整排序或关键词的断言已用 **`#[cfg(feature = "facility-classic-algorithms")]`**（runtime）或设施 crate 内仅在 `classic` 开启时编译的 `full` 测例覆盖。

---

## 3. Runtime 聚合 feature

- **`facility-classic-algorithms`**：为三个设施 crate 同时打开 `classic`。**`full`** 默认包含该项。  
- **`--no-default-features`**：不启用该项时，设施依赖为 `default-features = false`，三个 crate 的 **`classic` 均关闭**，编排走桩路径。

---

## 4. 阻碍项（未列入「可裁剪」的）

- **Agent 设施 crate**：当前无 `classic` 模块，本次无代码门控。  
- **用 `tokio::fs` / `spawn_blocking` 替代 `classic`**：**不适用**——`classic` 均为同步纯函数，无磁盘 I/O。

---

## 5. 维护提示

- 新增依赖 `oclive_*_builtin::classic` 的代码时，应假设 **API 稳定**；若强依赖「排序」或「关键词」语义，须在 **`facility-classic-algorithms` 开启** 的配置下测试。  
- 极简 SKU 若仍需完整情绪关键词但不需要 Builtin 进程内 Provider，可只开 **`facility-classic-algorithms`**（并自行选配 LLM / memory 等 feature），不必开 `default-emotion-providers`。
