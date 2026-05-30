/**
 * Reply presentation aligned with Rust `SendMessageResponse` (`src-tauri/src/models/dto.rs`).
 * Primary text field is **`reply`** (not `response`).
 * Knowledge pack hits this turn: `knowledge_chunks_in_prompt` (dev panel via `debugStore`).
 */
import type { PresenceMode, SendMessageResponse } from '../api'

/** UI presentation hints derived from backend snapshot (does not replace Pinia ChatMessage; derived only). */
export interface ReplyPresentation {
  /** Main dialogue text (same as `reply`). */
  replyText: string
  /** Co-present / remote stub / remote inner voice */
  presenceMode: PresenceMode
  /** `send_message` contract version (debug). */
  apiVersion: number
  /** DTO schema version (debug / migration). */
  schemaVersion: number
  /** Fallback short line used when primary LLM failed. */
  replyIsFallback: boolean
  /** Presence for bubble styling (matches ChatMessage / ChatMessageList). */
  presenceVariant: PresenceMode
  /** Assistant bubble emotion: remote_stub uses portrait emotion, else bot_emotion. */
  assistantEmotionLabel: string
}

export function presentationFromSendResponse(res: SendMessageResponse): ReplyPresentation {
  const replyIsFallback = Boolean(res.reply_is_fallback)
  const presenceMode = res.presence_mode
  const assistantEmotionLabel
    = presenceMode === 'remote_stub'
      ? res.portrait_emotion
      : (res.bot_emotion ?? res.portrait_emotion)

  return {
    replyText: res.reply,
    presenceMode,
    apiVersion: res.api_version,
    schemaVersion: res.schema,
    replyIsFallback,
    presenceVariant: presenceMode,
    assistantEmotionLabel,
  }
}
