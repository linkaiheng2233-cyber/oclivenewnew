# RFC：独立通道能力增强模块 — 注册表与扩展规则

| 元数据 | 值 |
|--------|-----|
| 状态 | **Registry v1**（`user_identity` / `reply_post_process` / **`theater_director`** 已交付） |
| 受众 | Cursor / 内核 / 编写器 / 发行版 / 社区插件作者 |
| 前置 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §1.2 · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| 权威中文名 | **独立通道能力增强模块** |
| 权威英文名 | **side-channel capability enhancement module**（文档 alias：**orthogonal capability unit**，历史用语，不再作首选标题） |

[English summary in §0](#0-english-summary)

---

## 0. English summary

**Side-channel capability enhancement modules** are kernel extensions that:

- **Do not** use the six `plugin_backends` / `slot_registry` host keys
- **Do not** take a **facility submodule N** number (①–④)
- Wire through a **dedicated resolver** and a **fixed anchor** on the Stable turn chain **or** a **standalone HTTP/Tauri API** outside `process_message`
- May optionally attach a **directory plugin** via manifest `provides` (independent of slot resolution)

**Not**: Experimental `dual_core` pipeline steps, module-4 Prompt slot, or module-5 LLM slot (those only get **consumed**).

---

## 1. 定位（与四层分类对齐）

| 大类 | 占六槽？ | 设施子模块号？ | 典型接入 |
|------|----------|----------------|----------|
| **第 1–6 模块（后端模块）** | 是 | — | `PluginHost` → `process_message` |
| **设施模块** | 否 | 可选 **第 N 设施子模块** | `turn_pipeline` 编排行内 |
| **独立通道能力增强模块** | **否** | **否** | **自有 Resolver** + pre/post 锚点 **或** 独立 API |
| **后端模块插件模块** | 否（挂第 K 模块） | 否 | `provides: ["llm"]` 等 |

**与 Experimental 核**：Experimental 改的是 Stable 链上「整圈工序顺序」；独立通道是 Stable 链上**固定钩子**（身份 / 后处理）或 **圈外 API**（剧场）。二者 **不互替**。

**与 bundled / shared 核**：spawn 降级只换内核二进制；`{app_data}/distros/chat-pro/plugins/` 与独立通道 Resolver **路径不变**。

---

## 2. 注册表 v1

| `id` | 规范中文名 | Authoritative English | 锚点 / API | 配置落点 | 插件 `provides` | 交付状态 |
|------|-----------|----------------------|------------|----------|-----------------|----------|
| **`user_identity`** | 用户身份 Prompt 模板 | **User Identity Prompt Template** | `turn_pipeline/pre` → `resolve_active_user_identity` → `PromptBuilder.push_user_identity_section`（**LLM 之前**） | 角色包 `user_identities/`；发行版 `[user_identity]` | **无**（内容在角色包；非 directory 槽） | **已交付** |
| **`reply_post_process`** | 回复后处理 | **Reply Post-Processor Plugin** | 内置 `post_llm` 之后 → `resolve_reply_post_processor` → `process_reply` | 角色包 `config.json` → `reply_post_processor`；发行版 `[post_process].chain` | **`reply_post_process`** · RPC `reply_post_process.process` | **已交付** |
| **`theater_director`** | 剧场场景导演 | **Theater Scene Director** | **`generate_theater_scene`** / **`POST /theater/scene`**（**不进** `process_message`） | `distro.oclive.toml` → `[theater].director_plugin`；env `OCLIVE_THEATER_DIRECTOR_PLUGIN`；fallback 内置 `scene_director.rs` / `patch_scene.rs` | **`theater_director`** · RPC `theater.build_prompt` | **已交付**（官方 `com.oclive.theater_director_official`） |

### 2.1 附录：宿主工具向（非对话内核编排）

下列能力可走 PLUGIN_V1 `provides`，但 **不** 登记为内核独立通道（无 `process_message` / 剧场 API 锚点）：

| `id` | 说明 | 典型宿主 |
|------|------|----------|
| **`test_runner`** | 编写器「跑测试」UI | `oclive-pack-editor` · `provides: ["test_runner"]` |

新增附录项须在 PR 中说明 **为何不** 占用注册表主表（避免与六槽 / 设施子模块 / 主链钩子混淆）。

---

## 3. 消歧（写入 NAMING_CONVENTIONS）

| 能力 | **不是** |
|------|----------|
| **User Identity** | 角色身份（`prompts/`）；六槽；设施子模块 |
| **Reply Post-Processor** | 第 4 模块 Prompt 槽；post-process chain profile 本身；Experimental step |
| **Theater Scene Director** | 第 4 模块 Prompt（契约是 `TheaterSceneRequest`，不是 `PromptInput`）；第 5 模块 LLM 插件（仅 **消费** `AppState::llm`）；默认 **不** 升格「第 5 设施子模块」 |

---

## 4. `theater_director` 插件通道（已交付）

与 [`reply_post_processor`](../../kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs) 对齐：

```
generate_scene()
  → resolve_theater_director()  (../../kernel/crates/oclive_kernel_host/src/domain/theater_director.rs)
  → [directory] RPC theater.build_prompt  (provides: theater_director)
  → AppState.llm.generate_tag / generate
  → fallback: scene_director.rs / patch_scene.rs 内置模板
```

- **官方包**：`distros/chat-pro/plugins/com.oclive.theater_director_official/`（Theater 构建时复制到 `distros/desktop-tauri/resources/distros/chat-pro/plugins/`）
- **开发 env**：`OCLIVE_THEATER_DIRECTOR_PLUGIN=<manifest.id>` 覆盖 profile `director_plugin`
- **RPC**：`theater.build_prompt` · params = [`TheaterPromptBuildInput`](../../kernel/crates/oclive_kernel_contracts/src/theater_director.rs) JSON · result `{ "prompt": "..." }`（非空，≤32k）
- 社区作者可替换 prompt 拼装，**不** 改六槽与 Stable 主链顺序

---

## 5. 扩展规则

新增 **独立通道能力增强模块** 须：

1. **RFC**（或扩本注册表）登记 `id`、中英文规范名、锚点/API、`provides`（若有）
2. 实现 **`resolve_*`**（或等价独立入口），**禁止** 写入 `slot_registry` 六键或 Experimental `pipeline.experimental` step
3. 同步 [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) 注册表小节、[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) `provides` 表（若适用）、[NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §1.2

**不** 自动占用「第 7 后端模块」或「第 5 设施子模块」号；若未来某能力需升格为设施子模块，须 **单独 RFC** 说明与独立通道实例的迁移关系。

---

## 6. 相关文档

| 主题 | 文档 |
|------|------|
| 用户身份 & 后处理（Phase 2 细节） | [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md](RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md) |
| 剧场内核 | [handoff/theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md) · `scene_director.rs` |
| 发行版插件矩阵 | [DISTRO_DEFAULT_PLUGINS.md](../kernel/DISTRO_DEFAULT_PLUGINS.md) |
| 架构总述 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) §独立通道能力增强模块 |
