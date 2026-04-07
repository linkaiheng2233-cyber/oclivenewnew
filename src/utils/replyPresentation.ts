/**
 * 与 Rust `SendMessageResponse`（`src-tauri/src/models/dto.rs`）对齐的回复展示策略。
 * 主文案字段名为 **`reply`**（不是 `response`）。
 * 知识包本回合命中条数见 `knowledge_chunks_in_prompt`（开发面板用 `debugStore` 记录）。
 */
import type { PresenceMode, SendMessageResponse } from "./tauri-api";

/** 从后端快照推导的 UI 展示提示（不替代 Pinia 中的 ChatMessage，仅作派生） */
export interface ReplyPresentation {
  /** 主对话文本（与 `reply` 一致） */
  replyText: string;
  /** 共景 / 异地占位 / 异地心声 */
  presenceMode: PresenceMode;
  /** `send_message` 契约版本（调试） */
  apiVersion: number;
  /** DTO 结构版本（调试 / 迁移） */
  schemaVersion: number;
  /** 主 LLM 失败时使用了备用短句 */
  replyIsFallback: boolean;
  /** 用于气泡样式的 presence（与 ChatMessage / ChatMessageList 一致） */
  presenceVariant: PresenceMode;
  /** 助手气泡用情绪：remote_stub 用立绘情绪，否则 bot_emotion */
  assistantEmotionLabel: string;
}

export function presentationFromSendResponse(res: SendMessageResponse): ReplyPresentation {
  const replyIsFallback = Boolean(res.reply_is_fallback);
  const presenceMode = res.presence_mode;
  const assistantEmotionLabel =
    presenceMode === "remote_stub"
      ? res.portrait_emotion
      : (res.bot_emotion ?? res.portrait_emotion);

  return {
    replyText: res.reply,
    presenceMode,
    apiVersion: res.api_version,
    schemaVersion: res.schema,
    replyIsFallback,
    presenceVariant: presenceMode,
    assistantEmotionLabel,
  };
}
