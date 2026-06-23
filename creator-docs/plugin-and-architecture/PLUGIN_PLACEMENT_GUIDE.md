# 插件放置指南（贡献者决策树）

物理安装路径：`{app_data}/plugins/<manifest.id>/`（目录插件）；开发时还可被 `roles/` 同级 `plugins/`、工作目录 `plugins/` 扫描到（见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)）。

## 三问决策树

### 1. 要替换六槽后端（memory / emotion / event / prompt / llm / agent）？

→ 在 `manifest.json` 声明 `provides` 含对应槽能力（如 `llm`、`memory`），并在角色蓝图 `slot_registry` / `plugin_backends` 中绑定该 directory 或 remote 插件。

→ 契约：[PLUGIN_V1.md](PLUGIN_V1.md) · [HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)

### 2. 要在 LLM 出字之后润色展示回复（不进 Prompt、不占六槽）？

→ `provides: reply_post_process` · RPC `reply_post_process.process`

→ 角色包 `config.json` → `reply_post_processor`；发行版 `[post_process].chain` 可合并策略。

→ 解析：`resolve_reply_post_processor`（[reply_post_processor.rs](../../crates/oclive_kernel_host/src/domain/reply_post_processor.rs)）

### 3. 要为剧场生成场景 Prompt（不进 `send_message`、不占六槽）？

→ `provides: theater_director` · RPC `theater.build_prompt`

→ 发行版 `distro.oclive.toml` → `[theater].director_plugin = "<manifest.id>"`；开发 env `OCLIVE_THEATER_DIRECTOR_PLUGIN` 覆盖 profile。

→ 解析：`resolve_theater_director`（[theater_director.rs](../../crates/oclive_kernel_host/src/domain/theater_director.rs)）；入口 `generate_theater_scene` / `POST /theater/scene`。

→ 官方示例：`plugins/com.oclive.theater_director_official/`

## 附录（非六槽、非上述独立通道）

| 能力 | 放置位置 | 说明 |
|------|----------|------|
| **复杂情感 `narrative_hint`** | 第 1 设施子模块 | `complex_emotion` provider；非 directory 六槽键 |
| **用户身份 Prompt** | 角色包 `user_identities/` | 独立通道 `user_identity`；非插件目录 |
| **Vitest 测试运行器** | `provides: test_runner` | 编写器工具；如 `official-vue-test-runner` |

## 相关文档

- 独立通道注册表：[RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)
- 目录插件扫描与权限：[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)
- 剧场路线图：[../../handoff/theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md)
