# oclive_emotion_builtin

**官方默认情绪模块**（工程名：设施 crate）。关键词七维情绪分析（feature **`classic`**，默认开；关时为强中性桩）与可选的 `UserEmotionAnalyzer` 内置槽（`providers`，隐含 `classic`）。门控见 `creator-docs/kernel/FACILITY_CLASSIC_ALGORITHMS_AUDIT.md`。

由 `oclive_kernel_runtime` 的 `default-emotion-providers` 聚合开启 `providers`。

Directory 侧车示例：`examples/oclive-emotion-builtin-directory/`（`emotion.analyze`）。
