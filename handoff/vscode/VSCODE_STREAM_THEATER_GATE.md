# VS Code 流式 SSE 与 Theater v0 冻结 — 解冻立场（2026-06-10）

**Gate 结论：允许落地 `POST /chat/stream`。**

## 依据

[theater/DEVELOPMENT_ROADMAP.md](../theater/DEVELOPMENT_ROADMAP.md) §5.5 冻结的是：

- 新增 `process_message` **编排阶段**
- 六槽扩展
- 蓝图 v3 DSL
- `dual_core` / `expert_routing` 解冻

`POST /chat/stream` **不**违反上述边界：

| 维度 | `/chat/stream` 行为 |
|------|---------------------|
| 编排阶段 | 与 `/chat` 共用同一 `run()` 路径；仅 LLM 调用改为 `generate_stream`，post-LLM 副作用仍在流结束后执行 |
| 六槽 | 无新槽位 |
| 端点 | **新增**、向后兼容；`/chat` 完全保留 |
| Agent / remote / dual_core | 整段 fallback（单次 `on_token`），不扩展 Experimental 核 |

## 范围

- **In scope**：Ollama 真流式、远程/目录 LLM 默认 trait 降级、`oclive-vscode` `chatStream` + `oclive.chat.streaming`
- **Out of scope**：Theater 产品、桌面 Theatre 戳点、编排新 stage

## 验收

- `POST /chat` 回归不变
- `POST /chat/stream`：`token` 事件 + 末帧 `done`（= `ChatApiResponse` / `SendMessageResponse`）
- VS Code F5：停止按钮中断 SSE；`oclive.chat.streaming=false` 回退 `/chat`

**Status:** Approved for implementation (Phase 3 plan).
