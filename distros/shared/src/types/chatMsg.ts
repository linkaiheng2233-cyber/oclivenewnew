/** Chat list display message (subset aligned with chatStore.ChatMessage fields) */
export interface ChatMsg {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  presenceVariant?: 'co_present' | 'remote_stub' | 'remote_life'
  replyIsFallback?: boolean
  aside?: string
  streaming?: boolean
}
