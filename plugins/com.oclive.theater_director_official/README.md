# Official Theater Director Plugin

Directory plugin **`com.oclive.theater_director_official`** — implements **`theater.build_prompt`** for AI Theater (`generate_theater_scene` / `POST /theater/scene`).

**Prompt pack v0.2** — 戏剧性纪律与分 mode 模板；日常创作 **只改本目录**，Rust 内置模板为 RPC 失败 emergency fallback。

## 能力

| 项 | 值 |
|----|-----|
| **Provides** | `theater_director` |
| **RPC** | `theater.build_prompt` → `{ "prompt": "<非空字符串>" }` |
| **Modes** | `patch` · `ripple` · `cast_adapt` · `cast_rewrite` · `cast_rewrite_minimal` |

Theater 发行版默认：`distro.oclive.toml` → `[theater].director_plugin = "com.oclive.theater_director_official"`。内核 RPC 失败或空串时 fallback 至 Rust builtin（见下方 sync 清单）。

## 目录结构

```
prompts/
  index.mjs              # buildTheaterPrompt(mode router)
  constants.mjs
  drama_guardrails.mjs   # 共享戏剧性纪律 + 场景语气 hint
  scene_context.mjs      # scene_brief / setting / persona 块
  modes/
    patch.mjs            # mode=patch（戳 chip 主路径）
    ripple.mjs
    cast_adapt.mjs
    cast_rewrite.mjs
    cast_rewrite_minimal.mjs
rpc_server.mjs           # JSON-RPC 入口
manifest.json
```

## Mode 一览

| `mode` | 用途 | 纪律注入 |
|--------|------|----------|
| `patch` | 用户戳 poke chip，prose 局部补丁 | 全量 `drama_guardrails` |
| `ripple` | 涟漪区 JSON 重写 | 全量 |
| `cast_adapt` | 非默认卡司三轮适配 | 精简版 |
| `cast_rewrite` | 卡司从零写主剧本 | 精简版 |
| `cast_rewrite_minimal` | JSON 解析失败 retry | 精简版 |

## `TheaterPromptBuildInput` 常用字段

与内核契约一致（[`TheaterPromptBuildInput`](../../crates/oclive_kernel_contracts/src/theater_director.rs)）：

| 字段 | 说明 |
|------|------|
| `mode` | 上表之一 |
| `theater_scene` | `breakfast` \| `supermarket` \| `way_home` \| `bedtime` — 影响默认 scene brief 与语气 hint |
| `scene_brief` / `scene_setting_hint` | 前端 catalog 传入；空则按 `theater_scene` 默认 |
| `persona_a` / `persona_b` | 卡司人设摘要 |
| `patch_tweak` | patch 模式：`lead_cast`、`chip_label`、`drama_seed` |
| `patch_prefix_beats` / `patch_canned_patch` | 上文与罐头参考 |
| `patch_variant` | `1` = 第二版候选 |
| `ripple_*` / `base_beats` / `fork_templates` | 各 mode 专用快照 |

## Fork / 替换（无 UI）

1. **Fork** 本目录 → 修改 `prompts/`（例如只改 [`drama_guardrails.mjs`](prompts/drama_guardrails.mjs) 一句即可换整体风格）
2. 新 `manifest.json` → 新 `id`（勿与官方 id 冲突）
3. 放入 `{app_data}/plugins/<id>/`
4. 指向插件：
   - 发行版：`distro.oclive.toml` → `[theater].director_plugin = "<id>"`
   - 开发：`OCLIVE_THEATER_DIRECTOR_PLUGIN=<id>`

最小可替换示例（**自包含 `prompts/`**，可整夹复制到 `{app_data}/plugins/`）：[`examples/directory-plugin-theater-director-minimal/`](../../examples/directory-plugin-theater-director-minimal/) — 见该目录 [`README.md`](../../examples/directory-plugin-theater-director-minimal/README.md)。

改插件后同步打包 seed：

```powershell
robocopy plugins/com.oclive.theater_director_official src-tauri/resources/plugins/com.oclive.theater_director_official /MIR
```

并 bump `manifest.json` `version`。

## 本地调试

```powershell
cd plugins/com.oclive.theater_director_official
node -e "import { buildTheaterPrompt } from './prompts/index.mjs'; console.log(buildTheaterPrompt({ mode: 'patch', cast_a_name: 'A', cast_b_name: 'B', patch_tweak: { drama_seed: 'test' } }));"
```

或启动 Theater dev 并设置 `OCLIVE_THEATER_DIRECTOR_PLUGIN` 指向你的 fork id。

Drift 烟测（插件 ↔ Rust fallback 关键子串）：`node scripts/theater-prompt-drift.mjs`

## Rust fallback sync 清单

日常 **只改插件**。以下变更时需同步 [`crates/oclive_kernel_host/src/domain/theater/`](../../crates/oclive_kernel_host/src/domain/theater/)：

| 变更类型 | Rust 落点 |
|----------|-----------|
| 戏剧性纪律 / 场景 hint | `drama_guardrails.rs` |
| `mode=patch` 标题与演出要求 | `patch_scene.rs` → `build_patch_prompt` |
| `ripple` / `cast_*` 模板 | `scene_director.rs` → `build_scene_prompt` 等 |
| 输出契约 / 字段语义 | 内核解析 + 上述 builder + 契约 crate |

**不测 LLM 质量** — 仅 `scripts/theater-prompt-drift.mjs` 与 Rust 单测断言 marker 子串（如 `【剧场即兴 · 戏剧性补丁】`、`【戏剧性纪律】`）。

## 人工验收

四场景 playtest 矩阵：[`handoff/theater/PLAYTEST_MATRIX.md`](../../handoff/theater/PLAYTEST_MATRIX.md)
