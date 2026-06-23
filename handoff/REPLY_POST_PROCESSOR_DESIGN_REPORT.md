# Reply Post-Processor · 设计汇报（v0.4 选项）

**状态**：设计级交付（Phase 3）；**默认不开启 LLM 润色**。  
**读者**：插件作者、宿主集成方。  
**English summary**：见文末 §9。

---

## 1. 定位

| 维度 | 说明 |
|------|------|
| **独立通道 `reply_post_process`** | 与六宿主槽、`slot_registry`、蓝图 `runtime_config` 六槽键**无关**；注册表 [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · 细节 [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md](../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md) |
| **与 Prompt 分工** | **`meta.reply_quality_anchor`**（或内核 `DEFAULT_REPLY_QUALITY_ANCHOR`）管**生成**；后处理管**落地**（格式、口癖修正、长度、安全、可选 LLM 重写） |
| **与 RFC 草案差距** | [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md) 多步 `chain` 尚未落地；今日仅 builtin profile + 单后端 |

---

## 2. 现行管线（代码锚点）

主路径：`process_message` → `turn_pipeline/post.rs`（约 L302+）

```text
LLM raw_reply
  → resolve_reply_post_processor (builtin | remote | directory)
  → display_reply
  → HybridConversationStore + SendMessageResponse.reply
```

- **`raw_reply`**：情绪/好感/立绘/档案进化等仍基于 LLM 原文（`raw_reply_before`）
- **`display_reply`**：用户可见文本；聊天存储与 API **`reply`** 字段

相关实现：

- 编排：`crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post.rs`
- 解析：`crates/oclive_kernel_host/src/domain/reply_post_processor.rs`
- builtin：`crates/oclive_kernel_runtime/src/domain/builtin_reply_post_processor.rs`
- trait：`crates/oclive_kernel_contracts/src/reply_post_processor.rs`

---

## 3. 三后端能力矩阵

| backend | 今日能力 | 润色插件目标 |
|---------|----------|--------------|
| **`builtin`** | 空白/引号/`max_chars` 等格式治理 | **保持格式层，不做 LLM** |
| **`remote`** | JSON-RPC `reply_post_process.process` | 你的 **LLM 润色 HTTP 服务** |
| **`directory`** | 同上经目录插件 stdio/http | 你的 **本地/脚本润色插件** |

包级开关：`roles/{id}/config.json` → `reply_post_processor`（**默认 `enabled: false`**）。

---

## 4. 插件契约

基于 `ReplyPostProcessor` trait + 示例 [`examples/directory-plugin-reply-post-process-minimal/`](../examples/directory-plugin-reply-post-process-minimal/)。

| 项 | 约定 |
|----|------|
| **Method** | `reply_post_process.process` |
| **Params** | `raw_reply`, `user_message`, `role_id`, `scene_id`, `locale` |
| **Returns** | `display_reply`, `diagnostic?` |
| **manifest** | `provides: ["reply_post_process"]` |
| **权限** | `network:*`（remote）/ `process:spawn`（directory stdio） |

润色脚手架（pass-through + LLM 注释）：[`examples/reply-post-process-polish/`](../examples/reply-post-process-polish/)。

---

## 5. 与 Prompt / 导出契约（Phase 1–2 定稿）

- **人设 Tier0**：`core_personality.txt`；`prompts/system.md` **不参与** `PromptBuilder`
- **质量锚点 SSOT**：`meta.reply_quality_anchor` 或内核默认；`prompts/reply_quality_anchor.md` 仅为编写器镜像
- **编写器导出**：双写 JSON + md；zip 含 `config.json`、`user_identities/`（可选）

---

## 6. 已知缺口（供 v0.4 设计）

| 缺口 | 现状 |
|------|------|
| **`locale`** | `post.rs` 写死 `"zh"` |
| **多步 chain** | `[post_process].chain` 仅覆盖 builtin HostProfile |
| **会话级开关** | 无；包级 `enabled` 默认 false |
| **`diagnostic`** | 未暴露给 UI |
| **`directory.plugin_id`** | 包校验不完整 |
| **builtin LLM** | **明确排除**；润色只走 remote/directory |

---

## 7. v0.4 设计选项表

| 选项 | 含义 | 适用 |
|------|------|------|
| **A** | 仅插件润色：directory 插件 + 包 `config.json` 启用 | 最快闭环 |
| **B** | 扩展 `PostProcessInput`（`persona_excerpt`, `interaction_mode`, 真实 `locale`） | 润色需更多上下文 |
| **C** | 发行版默认链（如 `desktop-chat`：minimal builtin + 可选插件） | 产品化默认体验 |
| **D** | 多步 chain RFC 落地（builtin → plugin 有序） | 与 RFC 完全对齐 |

---

## 8. 启用示例（directory 润色 · preset 缓存方案已落地）

实现：[`examples/reply-post-process-polish/`](../examples/reply-post-process-polish/)（`preset_cache` / `preset_builder` / `polish_rules` / `ollama_client`）。

- 宿主 spawn 时注入 **`OCLIVE_ROLES_DIR`**（`DirectoryPluginRuntime` → `spawn_child_handshake`）。
- 插件按 `role_id` 缓存 preset：`polish_prompt.md` 优先，否则 `core_personality.txt` 摘要 + `meta.reply_quality_anchor`。
- 规则门控命中后才调 Ollama；未配置 `OCLIVE_POLISH_MODEL` 或 Ollama 不可用时降级 raw。

`config.json`：

```json
{
  "reply_post_processor": {
    "enabled": true,
    "backend": "directory",
    "directory": { "plugin_id": "reply-post-process-polish" }
  }
}
```

本地烟测包：`roles/polish-dev/`（`dev_only: true`，默认不出现在角色列表；设 `OCLIVE_LIST_DEV_ROLES=1` 可见）。

插件目录：`{app_data}/plugins/reply-post-process-polish/`（或 roles 同级 `plugins/`）。

---

## 9. English summary

Reply Post-Processor is an **orthogonal** facility (not a six-slot backend). **Prompt** uses `core_personality.txt` + `meta.reply_quality_anchor` (or kernel default) for generation; **post-process** shapes the **display** reply. Today: **builtin** format-only; **remote/directory** via `reply_post_process.process`. Default **off**; LLM polish is **plugin-only**. v0.4 choices: A plugin-only, B richer input, C distro defaults, D multi-step chain.
