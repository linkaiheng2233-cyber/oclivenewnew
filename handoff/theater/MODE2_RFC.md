# AI 剧场 · 模式 2 RFC（`outline_rewrite`）

**状态**：已批准（产品门通过 · 2026-06-25）  
**配套**：[`MODE2_UNFREEZE.md`](MODE2_UNFREEZE.md) · [`INFORMATION_ARCHITECTURE.md`](INFORMATION_ARCHITECTURE.md) · [`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md) §5

---

## 1. 一句话

用户写**剧本大纲**（自由文本）+ 选双卡司；内核经圈外 API 一次 LLM 调用生成结构化 `beats[]`，前端以**独立叠层**播放——**不进** `process_message`、**不扩**六槽。

---

## 2. `mode` 名

| 字段 | 值 |
|------|-----|
| HTTP/Tauri `mode` | **`outline_rewrite`** |
| 插件 prompt router | 同左 |
| Rust builtin fallback | 同左 |

---

## 3. 请求 / 响应 JSON

### 3.1 `POST /theater/scene` · `generate_theater_scene`

在现有 [`TheaterSceneRequest`](../../kernel/crates/oclive_kernel_types/src/models/dto/mod.rs) 上扩展：

| 字段 | 必填 | 说明 |
|------|------|------|
| `mode` | 是 | `"outline_rewrite"` |
| `script_outline` | 是 | 用户大纲（≤4096 字；宿主截断） |
| `cast_a` / `cast_b` | 是 | 与模式 1 相同 |
| `scene_id` | 是 | 角色包场景 id（人设加载） |
| `fallback_beats` | 是 | LLM/解析失败时的罐头 beats（≥1） |
| `base_beats` | 否 | **可空**（大纲模式不从 skeleton 起笔） |
| `max_beats` | 否 | 默认 `OCLIVE_THEATER_RIPPLE_MAX_BEATS`（12） |
| `pair_relation_id` / `pair_relation_hint` | 否 | 与 cast_rewrite 一致 |
| `theater_scene` / `scene_brief` / `scene_setting_hint` | 否 | 场景语境增强 |

响应：现有 [`TheaterSceneResponse`](../../kernel/crates/oclive_kernel_types/src/models/dto/mod.rs)（`beats` · `source` · `model` · 可选 `failure_reason`）。

### 3.2 Prompt 输入（目录插件 `theater.build_prompt`）

[`TheaterPromptBuildInput`](../../kernel/crates/oclive_kernel_contracts/src/theater_director.rs) 增：

- `script_outline: Option<String>`

其余字段与 `cast_rewrite` 共用 persona / pair_relation / scene_context。

---

## 4. Prompt pack

| 路径 | 职责 |
|------|------|
| `distros/chat-pro/plugins/com.oclive.theater_director_official/prompts/modes/outline_rewrite.mjs` | 大纲 → JSON beats 契约 |
| `prompts/index.mjs` | `case "outline_rewrite"` |
| Rust `build_outline_rewrite_prompt` | RPC 失败 fallback |

**纪律**：复用 `drama_guardrails.mjs` + `scene_context.mjs`；输出仅 JSON 数组；双人 a/b；总拍数 ≤ `max_beats`。

Drift 锚点：`剧场大纲 · 用户剧本`（见 `scripts/theater-prompt-drift.mjs`）。

---

## 5. UI / IA 边界

| 区域 | 模式 1 | 模式 2 |
|------|--------|--------|
| 舞台主屏 | 官方 skeleton + poke | **不变**（默认仍模式 1） |
| 入口 | — | 顶栏「更多」→ **写大纲** 或 Header 次要按钮 |
| 叠层 | 设置 / 漏斗 | **`TheaterOutlineSheet`**（大纲 textarea + 生成 + 播放） |
| 状态 | `theaterSceneCatalog` | `useTheaterOutlineMode` + `localStorage` `oclive.theater.outline.v1` |

**禁止**：在模式 1 `PokeDock` 上加大纲输入；禁止模式 2 默认覆盖首屏。

---

## 6. 内核路由

[`scene_director::generate_scene`](../../kernel/crates/oclive_kernel_host/src/domain/theater/scene_director.rs)：

```
mode=outline_rewrite → generate_outline_rewrite_scene
  → outline_prompt_input → build_theater_prompt → generate_tag
  → parse JSON beats → fallback_beats on failure
```

超时 / beat 上限：复用 `OCLIVE_THEATER_SCENE_TIMEOUT_SECS` · `OCLIVE_THEATER_RIPPLE_MAX_BEATS`。

---

## 7. 验收

```powershell
node scripts/theater-prompt-drift.mjs
npm run test:theater:smoke
cargo test -p oclivenewnew-tauri --test http_api_theater
cargo test -p oclivenewnew-tauri --test theater_director_resolver
npm run tauri:build:theater
```

---

## 8. 显式不在范围

- 模式 3（`send_message` 长对话）
- 六槽 / `process_message` 编排扩展
- 大纲版本协作 / 多幕结构编辑器（后续）
