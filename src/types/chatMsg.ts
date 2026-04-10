/** 聊天列表展示用消息（与 chatStore.ChatMessage 字段对齐的子集） */
export type ChatMsg = {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  presenceVariant?: "co_present" | "remote_stub" | "remote_life";
  replyIsFallback?: boolean;
};
