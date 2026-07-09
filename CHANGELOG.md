# Changelog

> **English mirror**: [CHANGELOG.en.md](CHANGELOG.en.md) — 用户可见变更请与中英两份同步维护。

## [Unreleased]

### Added

- **[docs] English mirror wave 2**：`README.en.md` 与中文首页结构对齐（四例子 · 三发行版 · 生态 · 路线图）；`creator-docs-en` 补齐 CREATOR_GOLDEN_PATH、role-pack 深读 8 篇、dual-core、PLUGIN_MARKET、RELEASE_VERSIONING、RFC summary；`human-docs-en` 补齐 modules 全槽/设施/侧通道 + paths 三路径；路径归一（`NAMING_CONVENTIONS`、`development/LIGHTWEIGHT_PROFILE`、合并 `APPLICATION_SCENARIOS`）；新增 **`scripts/check-doc-mirror.mjs`** 接入 `npm run check:rust` 与 dimension5。

- **语音首字发声优化**：流式朗读经 `streamingVoiceChunker` 过滤旁白/内心/动作行；首块更早出句；CosyVoice2 `/warm` 默认 **prime** dummy 合成；角色切换触发侧车预热与 directive 预取；插件 manifest `rpcTimeoutsMs` 声明长 RPC 超时；CSP `connect-src` 收窄至侧车默认端口 `50000`。
- **语音扩展 v0.4（情感 TTS · 可选）**：`com.oclive.voice.asr` 升至 v0.4 · 默认纯文字；`tts_expansion_enabled` 开启后 CosyVoice2 侧车 + 模型 DLC（`voice_model_pack.json`）+ `synth_provider`（bundled / local_http / cloud）；RPC 增 `voice.probe_tts` · `voice.warm` · `voice.list_model_packs`；`rules-v1` 产出 `emo_text` + 角色包 `ref_map`；流式首句 `voice:stream-sentence` 提前 TTS；**移除 Piper 产品路径**（dev loop `--tts-sherpa` 保留）。详见插件 [`README.md`](distros/chat-pro/plugins/com.oclive.voice.asr/README.md) · [`TRACK_VOICE`](human-docs/team/TRACK_VOICE_RECOGNITION.md)。
- **统一键位绑定系统（Phase 1–4）**：设置 → 常规 → 高级新增「键位绑定」（应用内 + 全局快捷键统一 UI）；全局插件快捷键继续复用 `save_hotkey_bindings` 注册系统级监听；`ShortcutHelp` 改为动态读取当前键位；语音插件新增 **V 按住说话**（`voice.holdToTalk`，窗口聚焦时生效，输入框聚焦不抢键）。
- **Chat Pro Windows 98 彩蛋皮肤**：Konami 解锁 → `data-skin=win98`（`oclive-runtime-skin`）；设置 → 常规开关；Fluent + Tool 正交叠加于 `data-theme` / `data-shell` / UI 缩放；合成 Win98 标题栏（`Win98TitleBar` + Tauri `setDecorations`）与对话框 3D 窗框；见 [`MODULE_MAP_AND_HANDOFF.md`](handoff/MODULE_MAP_AND_HANDOFF.md) §13.2。
- **独立通道 `voice.asr`（Windows 已交付 · v0.2–0.3）**：官方目录插件 [`distros/chat-pro/plugins/com.oclive.voice.asr/`](distros/chat-pro/plugins/com.oclive.voice.asr/) · `provides: voice.asr` · **不进**六槽 / `process_message`；`chat_toolbar` 按住说话 + `plugin_rpc_invoke`（`voice.probe` / `voice.transcribe` / `voice.import_model` / `voice.list_profiles` / `voice.speak` / **`voice.build_directive`**) → `com.oclive.voice.asr:submit` → `send_message` 或 `chat:set_input_draft`（`mode: fill`）；**v0.3** 增 TTS `tts_profile` · `auto_tts` · `rules-v1` 导演 · 角色包可选 `voice_profile.json`；sherpa-onnx 引擎 SSOT 在 [`examples/voice-loop-minimal/asr/`](examples/voice-loop-minimal/asr/) · [`tts/`](examples/voice-loop-minimal/tts/) 经 `rpc_server.mjs` spawn；实验 synth：`edge-tts` · `pilot-tts` · `cosyvoice` adapter；官方角色包 `ui.json` 默认启用工具栏/设置插槽；Win98 覆写见 `win98/component-plugin-toolbar.css` / `component-voice-settings.css`；`plugin_bridge` RPC 白名单单测；Linux/macOS profile 返回 `unsupported_platform`；注册表见 [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) §4.1。
- **Domain layering ports（#101 解阻塞）**：`LlmClient::supports_prefix_cache` / `generate_with_opts` / `generate_stream_with_opts`；`TurnThinkingStatePort`；`co_present` / `slot_runner` / `post` 去除 domain→infra 直连；`npm run check:rust` 前置 layering + CHANGELOG parity 守门。
- **Affect 展示通道 `display_metrics`**：`RoleData` / `RoleInfo` / `SendMessageResponse` 增 UI-only 指标（`favor` / `traits[7]` / `relation_summary`）；旧标量字段标 deprecated；前端 `roleStore` 优先读新字段。
- **CI flake 自动重跑**：`.github/workflows/ci-rerun-flake.yml` 对 `rust` / `e2e-tauri` 失败限次 `gh run rerun --failed`。
- **Affect WS4.2–4.4（情感解耦）**：`apply_profile_evolution_atomic` 档案+七维同事务；深度档案 LLM 门控（强事件 OR 每 N 轮 OR 雷达 `radar_deep_pending`，默认 N=3）；`get_display_metrics` GET-only（Tauri + HTTP `/display_metrics`）；Tauri `affect:metricsChanged` 推送 + `roleStore` listen。
- **RFC affect 漂移闸**：`scripts/check-rfc-affect-drift.mjs` 接入 dimension5。
- **Wave E · Turn Thinking 持久化分流**：`[turn_thinking] fast_persistence = "strong_only"`（默认 `legacy`）；Fast 闲聊不写 long_term / favor / evolution；**Quarrel / Apology / Confession / Praise** 仍正常写入；RFC [`creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md`](creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)。
- **Wave F · Turn Thinking 包级路由**：`config.json` → `turn_thinking`（OR/AND · Deep latch · ephemeral 局面摘要 TTL）；迁移 `035_turn_thinking_runtime.sql`；本句 rule event Router 前 prepass；RFC §8–12。
- **Chat Pro 流式取消**：新消息发出时 `AbortController` 打断上一轮 SSE；清理悬空 `streaming` 气泡。
- **Chat Pro 流式开关**：设置 → 常规 → 高级「流式回复」（`localStorage` `oclive.chat.streamEnabled`，默认开启）。
- **Wave D · Deep persona capsule**：`prompts/deep_capsule.txt`（≤2500 字）· `meta.deep_capsule_enabled` · Small+Deep 时 `PromptBuilder` 用 capsule 替代全量 Tier0；mumu 样例已启用。
- **Wave C · Chat Pro 流式**：主 UI 经 `sendMessageStream` 接 `POST /chat/stream`（SSE `event:token`）；失败自动回退 blocking `/chat`。
- **Chat Pro 正式 profile**：`desktop.oclive.toml` 启用 `[turn_thinking]` Auto/Fast/Deep（`event_impact_llm` 默认仍 true，仅 Deep 轮调 event LLM）。
- **`measure-ttft.mjs --profile desktop|desktop-latency`**：区分正式 profile 与开发 bench；[`handoff/TTFT_BENCHMARK.md`](handoff/TTFT_BENCHMARK.md) 双表 + OOCP S15 命令。
- **Monorepo 目录重组（kernel / distros）**：`kernel/` 收纳 Rust crates；`distros/{shared,chat-pro,theater,desktop-tauri}` 拆分桌面发行版；RFC 见 [`handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md`](handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md)。
- **Theater Track A 工程卫生（轮次 16）**：[`handoff/theater/MODE2_UNFREEZE.md`](handoff/theater/MODE2_UNFREEZE.md) 模式 2 解冻 checklist；`theater-prompt-drift` 接入 `dimension5-acceptance.mjs` 与 `test:theater:smoke`；minimal 导演插件示例自包含 `prompts/`；`data/plugins.json` 登记 `com.oclive.theater_director_official`。
- **AI 剧场 Patch 涟漪升级**：poke 默认 `mode=patch`（局部 prose 小剧情 + 保留 skeleton 尾部）；`patch_variant` 双候选后台生成 + `TheaterVariantBackdrop` 拖拽切换；设置 → 舞台 Tab 可选 ripple 降级与自定义 poke 主角。
- **`CODE_OF_CONDUCT.md`**（Contributor Covenant）。
- **`human-docs-en/`** 最小集（L0–L3 + 08/09/10 英文摘要）。
- **`human-docs/08_PR_GATE_MATRIX.md`**、**`09_GLOSSARY.md`**、**`10_SETUP_WINDOWS.md`**。
- **`handoff/GOOD_FIRST_ISSUES.md`** 策展表。
- **`npm run check:ci-local`**；`package.json` `engines.node >=20`、**`.nvmrc`**。
- 前端：`distros/shared/src/api/plugin/*`、`useMainShell*`、`useChatStorageSettings`、`chatStoreSend`。

### Changed

- **Voice ASR v0.2.1（识别质量）**：聊天栏录音 WebM/Opus 经 `audioCapture.ts` 解码并重采样为 **16 kHz mono WAV** 再送 sherpa（修复此前误当 PCM 导致的识别极差）；麦克风约束启用 echoCancellation / noiseSuppression / autoGainControl；最短录音 350ms；引擎侧识别器缓存、过静音门控（`audio_too_quiet`）、可选 **ffmpeg** 压缩音频回退；新增 **medium** ASR profile 占位（设置里切换，需自行导入模型）。

- **Win98 皮肤 CSS 分层重构**：单体 `theme-win98.css` 拆为 `distros/shared/src/styles/win98/`（L0 tokens · L1 primitives · L2 壳 · L3 面板/组件 co-locate unscoped import）；最大化满框无青绿边、主窗 2px 圆角、对话框 navy 标题条贴边；见 [`MODULE_MAP_AND_HANDOFF.md`](handoff/MODULE_MAP_AND_HANDOFF.md) §13.2 样式依赖表。
- **Win98 皮肤抛光**：补全 `modal-backdrop` / `TimeDial.backdrop` 遮罩；Tool `UiSidePanel` navy 标题条与 Win98 ✕；合成标题栏改用 OCLive 应用图标（`public/oclive-icon.png`）。
- **Fluent「更多」面板 IA**：动作按钮顺序改为 设置 → 模型 → 插件 → 市场 → 快捷键说明；磁贴按 核心 / 插件 / 场景 / 开发 分组，Debug 移至末尾；设置 → 常规移除无内容的「快捷键」占位小节（说明入口保留在「更多」与 Ctrl 长按）。面板磁贴改为自适应栅格（`auto-fill minmax`），按占地大小从左到右排列（设置 / 场景跨两列，其余单列），日常聊 / 剧情两种模式下均整齐对齐。
- **Chat Pro 默认壳 Fluent**：`resolveOcliveShell()` fallback 由 `tool` 改为 **`fluent`**（安静客厅）；`VITE_OCLIVE_SHELL=tool` 仍可显式启用 ToolShell；早启动 `index.html` `data-shell` 同步；暗色品牌黛绿与浅色同色相（`--fluent-accent`）；角色 `primaryColor` 轻度染色（focus / 用户气泡 / runtime rail）；FluentShell 挂载 `InteractionModeBar` 为壳内唯一模式切换；互动模式 IA 见 [`MODULE_MAP_AND_HANDOFF.md`](handoff/MODULE_MAP_AND_HANDOFF.md) §13.1。
- **Prompt 力学彻底文本化（RFC #2 深化）**：`PromptBuilder` 不再向对话 prompt 注入任何好感/关系数值、关系阶段、事件块、边界语气指引或七维数值派生口吻（删除 `build_event_relation_state` / `build_boundary_tone_guideline` / `build_current_state` 及其数值辅助函数）；人设与语气完全由核心档案 + `mutable_personality` 叙事 + 用户情绪线索驱动，仅保留无数值的「真实性约束」防编造守门；七维/好感退场为 `display_metrics` 只读展示。
- **Chat Pro 默认 profile**：`desktop.oclive.toml` 启用 `fast_persistence = "strong_only"`（Fast 闲聊不涨好感/不进长期记忆；强关系事件仍巩固）。旧 session 数据不回滚。
- **仓库物理布局**：根 `crates/`、`src-tauri/`、`src/` 分别迁至 `kernel/crates/`、`distros/desktop-tauri/`、`distros/{shared,chat-pro,theater}/`；根 `npm run tauri:dev` / `tauri:dev:theater` 行为不变。
- **Theater 文档 SSOT 扫尾**：`theater_director` 由「拟/Deferred」统一为**已交付（2026-06）**（DISTRO_DEFAULT_PLUGINS · ARCHITECTURE · NAMING · ROADMAP §7 · IA）；[`TECHNICAL_DEBT_INVENTORY.md`](handoff/TECHNICAL_DEBT_INVENTORY.md) 轮次 16；验收链指向 [`PLAYTEST_MATRIX.md`](handoff/theater/PLAYTEST_MATRIX.md)。
- **hybrid 聊天镜像**：`rebuild_mirror_best_effort` / `delete_mirror_best_effort`（K-ROBUST-01）。
- **`canonical_llm_sync` / `plugin_state` / MCP·Ollama 降级**：`tracing::warn!`（K-ROBUST-02/03）。
- **内核快照与存储能力探测 degraded UI**（`kernel.ts`、`useKernelStatus`、`ChatStorageSettingsPanel`）。
- **`process_message` 可读性收尾**：`preflight_turn` / `PostLlmCtx` / `PreLlmOutput` 分组；`events.rs` / `blueprint_v2_slot_registry.rs` 模块提取；`SettingsView` Tab 子组件；`role_runtime` 子模块；`blueprint_v2` 测试外移。
- **handoff 目录整理**：`THEATER_*` → `handoff/theater/`、`VSCODE_*` → `handoff/vscode/`；新增 `handoff/launcher/`、`handoff/pack-editor/`、`handoff/studio/` 发行版附带文档入口；修复 theater greenfield 后断链。
- **README / CONTRIBUTING / SECURITY** 社区基建更新；PR 模板链到 PR 门矩阵；可选 `scripts/setup-dev.ps1`。
- **`chat_storage` pack 校验**：`oclive pack validate` 现校验 `config.json` → `chat_storage`（backend / location / 正整数 / replay 阈值 0.1–1.0），与 `reply_post_processor` 同级；`CHANGELOG.en.md` parity 同步。
- **五维审查收口（Batch 1–3）**：架构总览共景主链与 Stable 代码对齐；VS Code / 跨宿主文档改为 policy-first；`user_identities` 校验语义与 `load_role` 一致；`reply_post_processor` 在 `enabled` + `directory` 时校验非空 `plugin_id`；`ProcessMessageError` stage 保留于对外 `AppError` message；聊天回合 `role_runtime` 预取合并、`memory decay` 单条 CASE UPDATE、`SessionCache` 跳过重复 interaction_mode seed；workspace `default-members` 排除 `fuzz`；`chatStore` 加载与 `addMessage` 小优化。
- **Prompt guardrails 升格与页脚去重**：`KERNEL_DIALOGUE_GUARDRAILS` 恒含「状态延续」「倾诉优先」「篇幅随输入」，包级 `reply_quality_anchor` 无法换走；删除独立 `【回复结构】` 段；语气区块去除 `warmup_level` / `影响因子` 等系统术语；官方 mumu/shimeng/枫侵月锚点瘦身为仅人设差异。
- **许可证变更**：主程序由 AGPL-3.0 + 插件例外改为 **Apache-2.0**（根 `LICENSE` + `NOTICE`）；支持闭源商业发行版与嵌入式下游自由组合内核；`LICENSE_POLICY.md` 已同步。
- **官方发行版 · 日常聊 / 剧情模式分界**：`distro.oclive.toml` 新增 `[interaction]`；新增 `desktop-chat` profile；`desktop` / `vscode` 默认 `pure_chat`；首启 seed 优先级为发行版 → 角色包 → `pure_chat`。
- **纯聊 UI 瘦身与设置分层**：日常聊隐藏场景/时间/插件侧栏与精确好感分；设置分「常用 / 更多选项」；聊满 N 轮提示开启剧情模式。
- **用户身份惊喜解锁**：首屏不展示身份选择；聊满 5 轮或关键词触发「原来你还可以是…」身份 sheet。
- **互动模式默认与归一**：首启固定日常聊；用户切换后写入 `role_runtime` 下次沿用；模式切换归一至输入框上方 `InteractionModeBar`；顶栏「更多」移除语言/外观/插件重复入口。
- **日常聊隐藏插件**：纯聊模式下隐藏插件槽、市场、快捷键 Ctrl+Shift+F 与设置「插件」分栏；剧情模式恢复完整插件能力。
- **产品叙事对齐**：README / AGENTS / 定位文档统一为「AI 角色组装平台」；冻结项（dual_core、blueprint v3、expert_routing）表述为「机制已预埋，默认关闭」；聊天存储明确 hybrid 为生产路径。
- **Profile 调度 UX**：桌面状态栏与设置 → 内核与连接、VS Code 状态栏统一 profile 适配文案（attach / mismatch / pin / replace / degraded）。
- **发行版 Profile 解析 SSOT**：`distro.oclive.toml` 统一经 `oclive_kernel_runtime::distro_oclive_file` 解析（K-PROFILE-01）。
- **Host domain 再导出**：runtime 引擎模块 re-export 标记 deprecated；`check-host-reexport-imports.mjs` ratchet（D-OPUS-05）。
- **`resolve_*` 命名裁决（D-NAME-01）**：35 处非策略函数改为 `load_*` / `find_*` / `pick_*` / `build_*` / `merge_*` / `compute_*` / `invoke_*`；22 个跨宿主/回合策略锚点保留；动词表见 `NAMING_CONVENTIONS.md` §4.4。

### Added

- **Theater Release 打包链**：`npm run tauri:build:theater` · `OCLIVE_TAURI_SHELL=theater` · roles 子集（`theater-breakfast-a/b`）经 [`scripts/filter-theater-roles.mjs`](scripts/filter-theater-roles.mjs) 写入 `src-tauri/resources/roles/`。
- **Theater 15s 工程代理**：[`scripts/theater-stranger-proxy.mjs`](scripts/theater-stranger-proxy.mjs) · 聚合于 `npm run test:theater:smoke`（CI `frontend` job）。
- **Theater 思路与路线 SSOT**：[`handoff/theater/DEVELOPMENT_ROADMAP.md`](handoff/theater/DEVELOPMENT_ROADMAP.md)（模式 1 greenfield；旧 `THEATER_*` 文档已移除）。
- **三发行版内核 smoke（Pro / Flash）**：`npm run test:distro:smoke` 聚合 profile mirror · distro kernel e2e · Tauri bundled-first；`e2e-distro-kernel` 新增 **theater** scenario；CI **`cross-host-e2e`** 追加 `e2e-tauri-bundled-kernel` 与 VS Code profile diff。结项见 [`handoff/THREE_DISTRO_KERNEL_CLOSURE.md`](handoff/THREE_DISTRO_KERNEL_CLOSURE.md)。
- **Chat Pro bundled-first spawn（K-SCHED-05/01）**：Tauri `bundle-kernel-for-tauri.mjs` · `pick_best_for_spawn` bundled → shared → dev；`binary_upgrade` replace 默认关。
- **VS Code Flash profile 镜像**：`examples/distro-profiles/vscode.oclive.toml` ↔ 姊妹仓 `distro.oclive.toml` · `npm run test:distro-profile-mirror`。
- **契约扩展信封（V-CONTRACT Phase 0）**：`SlotExtension { schema_id, data }`；`EmotionResult` / `ComplexEmotionOutput` 可选 `extension`；`PromptInput.extra_sections` 在锚点前注入通用段落；演化规则见 `EXTENSION_POINTS.md`。
- **热路径 stage tracing（K-PERF-02）**：`oclive_turn` target 输出 per-`ChatStage` `elapsed_ms`；采样见 `creator-docs/getting-started/PERFORMANCE.md` §6。
- **CHANGELOG CI 门（K-DOC-02）**：`scripts/check-changelog-parity.mjs` 接入 `dimension5-acceptance.mjs`。
- **AI 剧场 v0（theater 发行版）**：`examples/distro-profiles/theater.oclive.toml`；`TheaterShell` 首屏（隐藏六槽/蓝图）；早饭场景 + 双反差角色包 + 预生成 `skeleton.json`；3 戳点芯片 + 本地 Ollama 局部 beat 改写（失败降级）。
- **产品冻结声明**：内核停扩直至 Theater v0 陌生人验证 — 见 [`handoff/theater/DEVELOPMENT_ROADMAP.md`](handoff/theater/DEVELOPMENT_ROADMAP.md) §4.8。
- **创作者黄金路径**：`creator-docs/getting-started/CREATOR_GOLDEN_PATH.md`（与内核文档分离）。

### Performance

- **记忆衰减写盘批处理（K-PERF-01/06）**：`DbManager::persist_memory_decay_batch` 由「每条记忆一次独立 `UPDATE`」改为「单事务批量提交」；rank 后每回合仅调用一次（衰减写回 + `accessed_at` 触达合并）。见 `long_term_memory.rs` 与 `turn_pipeline/pre.rs`。
- **前端外壳懒加载（K-PERF-09）**：`App.vue` 改用 `defineAsyncComponent` 动态导入 `FluentShell` / `ToolShell`，仅按 `resolveOcliveShell()` 结果加载当前外壳，未渲染的外壳不再进入首屏主 chunk。
- **热路径 DB 合并（K-PERF-03~06）**：每回合一次 `EffectiveSessionConfig`；`get_role_runtime_snapshot` 单查；`TurnPrefetch` 共享 / `agent=none` 跳过 agent DB；记忆 decay 单事务。基线见 `handoff/OPUS_48_PERF_BASELINE.md`。
- **长驻内存/SQLite（K-PERF-07/08/12）**：`SessionCache` 六 map cap+TTL；`personality_vector` 复合索引 migration `033`；`hybrid_store` 去掉多余 `get_chat_session`；`role_cache` LRU(32)；LLM startup probe 后台化。
- **前端壳内懒加载与轮询退避（K-PERF-10/11）**：`FluentShell`/`ToolShell` 非首屏面板 `defineAsyncComponent`；`useKernelStatus` 在 tab hidden 时 60s 退避。
- **RoleRuntimeSnapshot 下游复用（K-PERF-20）**：`relation_snapshot` / `post` / `pre` Profile 路径共享快照；每回合 `get_current_emotion` ≤1（写后刷新除外）。
- **Ollama 模型 settings 批量读（K-PERF-21）**：`resolve_effective_ollama_model` 经单次 `get_app_settings([provider, remote_model])` 批量读取。
- **聊天 session 列表与 upsert（K-PERF-22）**：session 列表 snippet 窗口函数 JOIN；`upsert_chat_session` `RETURNING` 消除写后重读。
- **长期记忆与操作日志索引（K-PERF-23）**：migration `034_perf_indexes.sql`（`idx_ltm_role_content` / `idx_operation_logs_role`）。
- **post 阶段 Role clone 减少（K-PERF-24）**：`TurnContext.role_arc` 供 profile evolution spawn 复用。
- **`pre_llm` Wave 1 并行（K-PERF-14）**：`turn_pipeline/pre.rs` 以 `tokio::try_join!` 并行 context / emotion / 模型 / narrative hint / 记忆五路只读；`oclive_turn` 输出 `pre_llm_wave1` 汇总；采样见 `PERFORMANCE.md` §6。

### Fixed

- **历史聊天记录在剧情场景下消失**：冷启动统一 `bootstrapChatForRole`（await 拉取 + `beginNewChatSessionOnRestart` 折叠）；移除 `interactionMode` watch 的 `immediate` 竞态；`loadedBucketKeys` 防止空占位桶短路；切角色时按后端有会话的场景 / 角色包场景 / IDB 索引回退加载。守门 `chatStoreScene.test.ts`、`chatStoreLoad.test.ts`，见 [`CHAT_STORAGE_ARCHITECTURE.md`](handoff/CHAT_STORAGE_ARCHITECTURE.md)。
- **Ctrl+Shift+S 打开设置失效**：`useGlobalHotkeys` 误引用未传入的 `opts.openSettingsView`（运行时为 `undefined`），改为调用本地 `openSettingsView`；theater 壳仍发 `theater:settings`。
- **语音插件 `get_plugin_settings_ui` 桥接失败**：`ui_slots` 经 `plugin_bridge_invoke` 调用插件设置读/写时，桌面未在 `dispatch_local_bridge_command` 分发 `get_plugin_settings_ui` / `set_plugin_settings_config`，报 `unsupported bridge command`；已路由至 `plugin_config.rs`。

---

## [0.4.0] - 2026-06-12

### Added

- **立绘 catalog（A2/B1）**：`portrait_catalog.json` SSOT；7 固定槽 + 高级多条目；`visual_state_id` / `performance_directive` additive DTO。
- **表现导演**：`pick_portrait_with_catalog` + 复杂情感 `narrative_hint` 闭环；legacy `portrait_emotion` 七 tag 零回归。
- **视觉表现 v1**：`materialize_directive`（image/live2d/rig3d/procedural）；distro `[visual_presentation].mode` gating（`off` / `image_only` / `stage_full`）。
- **OOCP S16**：catalog fixture 断言 `visual_state_id` + `performance_directive`；mumu 无字段。
- **编写器**：`PortraitCatalogEditor`、分级导出 profile（`desktop-full` / `vscode-lite` / `theater`）、`visual_presentation` UI。
- **VS Code Flash**：HTTP 解析 `visual_state_id` / `performance_directive`；catalog 路径优先于 tag 文件名。
- **Theater**：`TheaterStagePanel` + `Live2DStageAdapter` 接线（Cubism defer，PNG fallback）。

### Changed

- RFC 立绘/视觉表现状态更新为 Phase 1–4 delivered。
- `theater.oclive.toml` bundled profile 同步 `stage_full`。

---

## [0.3.0] - 2026-06-07

**桌面宿主 `0.3.0`** · **VS Code 扩展 `0.3.0`** · **`SendMessageResponse.schema` 14**

### Breaking

- **`SendMessageResponse.schema`** 升至 **14**：可选字段 `raw_reply`（仅当请求 `include_raw_reply: true` 且 post-processor 改变文本时返回）。
- **`high_risk_grants.json`**：仅接受规范权限键（`mcp:http`、`mcp:stdio`、`process:spawn`、`network:*`）。旧版 `mcp_http` / `directory_plugin_process_spawn` 等别名不再读取；请手动迁移文件后重授。

### Changed

- **实验性双核运行时**：`oclivenewnew-tauri` 新增 Cargo feature **`dual_core`**（默认关闭）。启用后编译 `dual_pipeline*` 并在 `role.dual_core_gated()` 时走实验核路径；`cargo build -p oclivenewnew-tauri --features dual_core`。
- **双核状态口径**：运行时双核文档状态更新为 **Opt-in Beta（默认关闭）**；对外交付仍以 Stable 路径为默认。

### Added

- **User Identity & Reply Post-Processor Phase 2（收尾）**：HostProfile `[user_identity]` / `[post_process]` 合并；remote/directory 后处理后端；HTTP `/user_identity/*`；桌面与 VS Code 身份切换；`RoleInfo` / `GET /role_info` 后处理只读字段；调试面板后处理状态行。见 [handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md](handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md)。
- **文档**：ROLE_PACK_SPEC §1.1 / §9.7、架构总览「正交能力单元」、USER_MANUAL §3.4–3.5、RFC Phase 2 验收勾选。
- **遗忘曲线与关系演化（`config.json`）**：艾宾浩斯长期记忆衰减（`memory.decay_halflife_days`）；重复提及强化（`mention_count` + `reinforcement_factor`）；沉浸模式下亲密值疏远与关系阶段降级（`relation.*`）；虚拟时间流速（`time.speed`）与首次沉浸对齐 `life_schedule` 起点；强化记忆微幅推动七维人格 / 可变档案「记忆塑造」。规范见 [ROLE_PACK_SPEC §9](creator-docs/role-pack/ROLE_PACK_SPEC.md)。
- **双核质量验收补强**：OOCP 新增可选 **S14**（experimental 合法 DAG 成功路径）；`oocp-test-suite` CI job 现以 `--features dual_core` 构建并执行 `run.mjs --include-dual-core`（覆盖 S13 降级 + S14 成功路径）；新增 `src-tauri/tests/dual_core_happy_path.rs` 集成测验证 `DualPipelineRunner::run_experimental` 成功路径。

#### Chat Storage（phase 3）

- **插件化后端**：支持 `hybrid`（默认，SQLite + JSON 镜像）、`file`（纯 JSON）、`sqlite`（纯数据库）三种 `ConversationStore` 实现；选择方式：环境变量 `OCLIVE_CHAT_STORAGE_BACKEND` 或角色包 `config.json` → `chat_storage.backend`；`oclive-cli init` 交互步骤已加入后端选择。
- **记忆回放**：`replay_memory_extraction` / `get_replay_progress` — 从聊天记录合并重提取 AI 记忆（去重按关键词相似度，可配 `replay_similarity_threshold`，默认 0.6）；设置 → 存储管理支持角色 / 场景 / 会话三级回放与进度轮询。
- **File 后端功能补齐**：`search_messages`（按 `chats/{role_id}/` 遍历 JSON）；`replay_memory_extraction`（聊天读文件、记忆写入 SQLite `long_term_memory`）；`list_sessions_by_role` 供 role 范围回放。
- **能力探测与 UI**：`get_chat_storage_capabilities` 返回 `supports_search` / `supports_replay` / `supports_cleanup` / `backend_kind`；存储管理面板按后端动态显示搜索、清理、回放入口，并展示当前后端友好名称（i18n）。
- **可配置项**：`config.json` 新增 `chat_storage.backend`、`chat_storage.replay_similarity_threshold`（可选，向后兼容）。
- **开发者**：`ConversationStore` trait 扩展 `list_sessions_by_role`、`supports_*`；`replay.rs` 的 role 范围收集改走 trait 而非直接查 DB。架构见 [handoff/CHAT_STORAGE_ARCHITECTURE.md](handoff/CHAT_STORAGE_ARCHITECTURE.md)。

---

## [0.2.0] - 2026-05-22

**桌面宿主 `0.2.0`** · **`oclive-cli` `0.1.0`** · **`oclive_kernel_runtime` `0.2.0`**（独立 SemVer，见 [RELEASE_VERSIONING.md](creator-docs/development/RELEASE_VERSIONING.md)）。

### Breaking

- **角色包 v2**：新包以 **`pipeline.ocblueprint`**（`schema_version: 2`）为唯一配置中枢；`oclive pack validate` **默认 v2**。旧包迁移：[V1_TO_V2_MIGRATION.md](creator-docs/role-pack/V1_TO_V2_MIGRATION.md)。
- **CLI**：移除顶层 `publish`、`plugin search/update`、`registry login`（见 [DEPRECATED_COMMANDS.md](crates/oclive-cli/DEPRECATED_COMMANDS.md)）。

### Added

- **蓝图 v2 与架构图**：`slot_registry` / 会话 `set_session_slot_override`、写盘 **`save_role_slot_registry`**；黄金包 **`roles/mumu`** 等已迁 v2。
- **双核（Dual-core）**：`runtime_config.dual_core` + `pipeline.experimental` 实验步，失败静默降级稳定核 `co_present`（默认关）。
- **`oclive-cli` 工具链**（22 个顶层子命令）：`init`（含 **`--monolith`**）、`build`、`bench`（`--matrix` / `--cold-start` / `--soak` / `--save`）、`dev`、`pack`、`doctor`、`test --oocp`、`explain` 等；见 [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md)。
- **Monolith 焊接模式**：`init --monolith` → `build` → 双二进制 **`bench`**；[RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)。
- **HTTP `--api`**：`GET /health`、`POST /chat`；CI **OOCP 黑盒 S0–S11** + 进程重启烟测。
- **Agent / MCP**、目录插件高风险授权、插件 HTML **`OclivePluginBridge`**、市场索引安装。
- **启动自检** `startup_health`；**`oclive explain`** 全量 `AppError` 词条；**`oclive doctor`** 蓝图三项检查。
- **编排**：`TurnContext` 收敛回合参数；`AppStateBuilder` + 策略注册表拆分；滚动文件日志（`OCLIVE_LOG_DIR` / `--api`）。

#### 聊天记录跟随角色包

- **`config.json` → `chat_storage.location`**：新增 `"role_pack"` / `"global"`（默认 `"global"`，向后兼容）。`"role_pack"` 时聊天记录存在角色包目录下的 `chats/` 子目录，角色包目录不可写时自动回退到全局路径并打印 warn 日志。
- **init 脚手架**：`oclive-cli init` 新增「聊天记录存储位置」交互步骤（跟随角色包 / 全局位置）。
- **存储管理面板**：选中角色后显示当前存储位置标签（📁 跟随角色包 / 🗄️ 全局位置）。
- **导出格式变更**：`export_role_chats` 格式由 ZIP+base64 改为组合 JSON（`application/json`）；内容不变，移除 `zip` 依赖。

### Changed

- **主编排**：Tauri 与 HTTP 均经 **`process_message`**；入口蓝图 **不再**作首轮 DSL 调度。
- **角色包格式**：`pack validate` 默认 v2（`--profile legacy` 保留旧包）；manifest/settings 顶层键白名单收紧。
- **Tauri**：`generate_handler!` 按域分组注释；移除 `reqwest` `blocking` 与 `@tauri-apps/api/fs` 直连（改自定义 command）；插件 bridge 脚本外置为前端 IIFE 资源。
- **架构图 v2**：移除手拖连线 composable（边由 `slot_registry` 派生）。
- **前端**：i18n 域拆分、`tauri-api` 模块化、Vite vendor chunk 拆分；`App.vue` 顶栏面板抽取。

### Fixed

- **错误处理**：统一 **`AppError` / `KernelErrorBody` JSON** + 前端 **`apiErrors`** 映射（含 invoke 与 HTTP 同形）。
- **SQLite**：WAL + 连接池（`sqlite_pool.rs`）；Release profile 调优（`opt-level=3`、`codegen-units=1`）。
- **并发**：内存 **`Cache`** 读锁优先 + 容量上限；角色冷加载 **`DashMap` inflight**（不再依赖 `Arc::strong_count`）。
- 插件事件订阅竞态、自定义事件被 `bridge.events` 误拦、Remote 未配置 URL 时的可见警告等。

### Performance

- Release 二进制采样约 **12 MiB PE / 7.6 MiB .text**（见 [PERFORMANCE.md](creator-docs/getting-started/PERFORMANCE.md)）。
- 目录插件 IPC in-flight 合并（catalog / bootstrap / plugin_state）；`pluginStore` 刷新与 slot memo 优化。

### Engineering

- 工作区 **`cargo clippy -D warnings`** 与 CI 对齐；共享 **`oclive_validation`**；**`invoke` 热路径**集成测 11 条。
- **`npm run check:release`** 发版闸门；Playwright **`vite preview`** 首屏（Ubuntu CI）。

### Documentation

- [COMPATIBILITY.md](creator-docs/COMPATIBILITY.md)、[PRODUCT_RELEASE_CHECKLIST.md](handoff/archive/PRODUCT_RELEASE_CHECKLIST.md)、双语 **creator-docs-en** 镜像与蓝图 v2 文档收口。

---

## [0.2.0] — 2026-04-02

（0.2.x 周期内较早合入项；已包含在上列 **0.2.0** 发版说明中。）

### Added

- 大角色包导入进度：后端 `import_progress` 事件 + 前端导入进度条模态框。
- 角色包导入前预览（`manifest.json` peek）与冲突处理：当角色 ID 已存在时弹出“覆盖/取消”确认。
- 角色包导入支持 **`.zip`**（与 `.ocpak` 相同容器）以及 **已解压目录**（与 `roles/{角色id}/` 布局一致）；见 `roles/README_MANIFEST.md`。
- 场景切换欢迎语：`switch_scene` 成功后读取 `scene.json` 的 `welcome_message`（或稳定随机 monologue）并自动插入聊天区人设消息。
- 关系阶段升级提示：`send_message` 响应增加 `relation_state`，前端在“升级”时插入系统消息。

### Changed

- 虚拟滚动策略：`ChatMessageList` 在有消息时始终启用虚拟滚动（减少 DOM 压力）。
- 角色包导出命名：导出文件默认改为 `{role_name}_{version}.ocpak`（安全化文件名）。

### API

- `send_message` 响应新增 `relation_state`；`emotion` 仍表示用户输入侧七维分析。

---

## [0.1.0]

- 初始公开基线（以仓库内首次标记版本为准）。
