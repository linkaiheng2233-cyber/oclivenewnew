import type { PresenceMode, SendMessageResponse } from '../api'
import { defineStore } from 'pinia'
import { hostEventBus } from '../lib/hostEventBus'
import { getRelationUpgradeMessage } from '../utils/relation'
import { presentationFromSendResponse } from '../utils/replyPresentation'
import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
  type RoleplaySplit,
} from '../utils/roleplayReplySplit'
import {
  fetchChatMessages,
  migrateIndexeddbToBackend,
} from '../api/chatStorage'
import type { StoredMessage } from '../api/chatStorage'
import {
  bucketMapKey,
  loadBucketFromIdb,
  loadMessageMapFromIdb,
  messageMapToImportBuckets,
  migrateMessageMapFromLocalStorage,
  migrateMessageMapShape,
  migrateMonolithBlobToBuckets,
  saveBucketToIdb,
  saveDirtyBucketsToIdb,
  type RoleSceneMessageMap,
} from '../utils/chatMessageDb'
import { conversationSessionId } from '../utils/conversationSessionId'
import {

  sendMessage,

} from '../api'
import { useDebugStore } from './debugStore'
import { useRoleStore } from './roleStore'
import { useUiStore } from './uiStore'

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  /** assistant：本回合 bot 情绪（小写）；user 通常不传 */
  emotion?: string
  /** assistant：异地模式（用于样式） */
  presenceVariant?: PresenceMode
  /** 主 LLM 失败时的备用短回复（与后端 `reply_is_fallback` 一致） */
  replyIsFallback?: boolean
  /** 从主回复拆出的旁白/内心/动作（仅 assistant；主 content 为对白） */
  aside?: string
}

/** 与后端默认单会话上限对齐（角色包可覆盖） */
const MAX_MESSAGES_PER_CONVERSATION = 500

const CHAT_STORAGE_MIGRATED_KEY = 'chat_storage_migrated'

function parseMessageTimestamp(iso?: string | null): number {
  if (!iso)
    return Date.now()
  const ms = Date.parse(iso)
  return Number.isFinite(ms) ? ms : Date.now()
}

function storedMessageToChatMessage(m: StoredMessage): ChatMessage {
  let emotion: string | undefined
  let replyIsFallback: boolean | undefined
  if (m.metadata) {
    try {
      const meta = JSON.parse(m.metadata) as Record<string, unknown>
      if (m.sender === 'assistant') {
        if (typeof meta.bot_emotion === 'string')
          emotion = meta.bot_emotion
        if (typeof meta.reply_is_fallback === 'boolean')
          replyIsFallback = meta.reply_is_fallback
      }
    }
    catch {
      /* ignore */
    }
  }
  const role = m.sender === 'assistant'
    ? 'assistant'
    : m.sender === 'user'
      ? 'user'
      : 'system'
  const base: ChatMessage = {
    id: m.id,
    role,
    content: m.content,
    timestamp: parseMessageTimestamp(m.created_at),
    emotion,
    replyIsFallback,
  }
  if (role === 'assistant') {
    const split = splitRoleplayReply(m.content)
    return {
      ...base,
      content: assistantDialogueFromSplit(m.content, split),
      ...(split.aside.trim() ? { aside: split.aside.trim() } : {}),
    }
  }
  return base
}

/** 进入某场景时，该桶内已有消息条数；索引小于该值的视为「历史」折叠区（按角色×场景） */
export type SceneHistorySplitIndex = Record<string, Record<string, number>>

function isLegacyRoleBucket(
  bucket: RoleSceneMessageMap[string] | undefined,
): bucket is ChatMessage[] {
  return Array.isArray(bucket)
}

function roleSceneBucket(
  map: RoleSceneMessageMap,
  roleId: string,
  sceneId: string,
): ChatMessage[] {
  const sid = sceneId || 'default'
  const roleBucket = map[roleId]
  if (isLegacyRoleBucket(roleBucket)) {
    map[roleId] = { default: roleBucket }
  }
  if (!map[roleId])
    map[roleId] = {}
  const scenes = map[roleId]!
  if (!scenes[sid])
    scenes[sid] = []
  return scenes[sid]!
}

function roleSceneAsideKey(roleId: string, sceneId: string): string {
  return `${roleId}:${sceneId || 'default'}`
}

function lastAssistantAsideFromMessages(messages: ChatMessage[]): string {
  for (let i = messages.length - 1; i >= 0; i--) {
    const m = messages[i]
    if (m.role === 'assistant') {
      const aside = m.aside?.trim()
      if (aside)
        return aside
    }
  }
  return ''
}

function syncLastAssistantAside(
  map: Record<string, string>,
  roleId: string,
  sceneId: string,
  messages: ChatMessage[],
): void {
  map[roleSceneAsideKey(roleId, sceneId)] = lastAssistantAsideFromMessages(messages)
}

function rebuildLastAssistantAsideMap(messageMap: RoleSceneMessageMap): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [roleId, roleBucket] of Object.entries(messageMap)) {
    if (isLegacyRoleBucket(roleBucket)) {
      syncLastAssistantAside(out, roleId, 'default', roleBucket)
      continue
    }
    for (const [sceneId, messages] of Object.entries(roleBucket)) {
      syncLastAssistantAside(out, roleId, sceneId, messages)
    }
  }
  return out
}

let persistMessagesTimer: ReturnType<typeof setTimeout> | null = null
const dirtyBuckets = new Set<string>()

/** 防止 split ≥ 条数导致主聊天区空白（新消息全进折叠历史） */
function clampSceneHistorySplitForBucket(
  splitIndex: SceneHistorySplitIndex,
  roleId: string,
  sceneId: string,
  messageCount: number,
  /** 本回合发送前的条数：若 split 挡住刚发的消息，回退到此 */
  sessionFloor?: number,
): void {
  const sid = sceneId || 'default'
  const roleSplits = splitIndex[roleId]
  if (!roleSplits || roleSplits[sid] === undefined)
    return
  let next = Math.min(roleSplits[sid], messageCount)
  if (
    sessionFloor !== undefined
    && messageCount > 0
    && next >= messageCount
  ) {
    next = Math.min(sessionFloor, messageCount)
  }
  if (next !== roleSplits[sid])
    roleSplits[sid] = next
}

function adjustSplitAfterTrim(
  splitIndex: SceneHistorySplitIndex,
  roleId: string,
  sceneId: string,
  removedFromHead: number,
): void {
  if (removedFromHead <= 0)
    return
  const sid = sceneId || 'default'
  if (!splitIndex[roleId] || splitIndex[roleId][sid] === undefined)
    return
  splitIndex[roleId][sid] = Math.max(0, splitIndex[roleId][sid] - removedFromHead)
}

/** 重启后若 split 把全部消息划入「折叠历史」，主聊天区会空白；恢复为直接展示。 */
function repairSplitsSoCurrentSessionVisible(
  splitIndex: SceneHistorySplitIndex,
  messageMap: RoleSceneMessageMap,
): void {
  for (const [roleId, roleBucket] of Object.entries(messageMap)) {
    if (isLegacyRoleBucket(roleBucket)) {
      const n = roleBucket.length
      if (n > 0 && (splitIndex[roleId]?.default ?? 0) >= n) {
        if (!splitIndex[roleId])
          splitIndex[roleId] = {}
        splitIndex[roleId].default = 0
      }
      continue
    }
    for (const [sceneId, messages] of Object.entries(roleBucket)) {
      const n = messages.length
      if (n > 0 && (splitIndex[roleId]?.[sceneId] ?? 0) >= n) {
        if (!splitIndex[roleId])
          splitIndex[roleId] = {}
        splitIndex[roleId][sceneId] = 0
      }
    }
  }
}

function sanitizeAllSceneHistorySplits(
  splitIndex: SceneHistorySplitIndex,
  messageMap: RoleSceneMessageMap,
): void {
  for (const [roleId, roleBucket] of Object.entries(messageMap)) {
    if (isLegacyRoleBucket(roleBucket)) {
      clampSceneHistorySplitForBucket(
        splitIndex,
        roleId,
        'default',
        roleBucket.length,
      )
      continue
    }
    for (const [sceneId, messages] of Object.entries(roleBucket)) {
      clampSceneHistorySplitForBucket(
        splitIndex,
        roleId,
        sceneId,
        messages.length,
      )
    }
  }
}

function schedulePersistMessages(
  map: RoleSceneMessageMap,
  roleId: string,
  sceneId: string,
) {
  dirtyBuckets.add(bucketMapKey(roleId, sceneId || 'default'))
  if (persistMessagesTimer)
    clearTimeout(persistMessagesTimer)
  persistMessagesTimer = setTimeout(() => {
    persistMessagesTimer = null
    const pending = new Set(dirtyBuckets)
    dirtyBuckets.clear()
    void saveDirtyBucketsToIdb(map, pending)
  }, 300)
}

export const useChatStore = defineStore(
  'chat',
  {
    state: () => ({
      messageMap: {} as RoleSceneMessageMap,
      isLoading: false,
      sceneHistorySplitIndex: {} as SceneHistorySplitIndex,
      lastAssistantAside: {} as Record<string, string>,
      messagesHydrated: false,
    }),
    getters: {
      messagesForRoleScene: (state) => {
        return (roleId: string, sceneId: string): ChatMessage[] => {
          const sid = sceneId || 'default'
          const roleBucket = state.messageMap[roleId]
          if (isLegacyRoleBucket(roleBucket))
            return roleBucket
          return roleBucket?.[sid] ?? []
        }
      },
      sceneHistorySplitForRoleScene: (state) => {
        return (roleId: string, sceneId: string): number => {
          const sid = sceneId || 'default'
          return state.sceneHistorySplitIndex[roleId]?.[sid] ?? 0
        }
      },
      lastAssistantAsideFor: (state) => {
        return (roleId: string, sceneId: string): string =>
          state.lastAssistantAside[roleSceneAsideKey(roleId, sceneId)] ?? ''
      },
    },
    actions: {
      /** 启动：迁移 IndexedDB → 后端，再从 API 加载当前角色×场景（失败则 IDB 缓存兜底）。 */
      async hydrateFromStorage() {
        if (this.messagesHydrated)
          return
        await this.runIdbMigrationIfNeeded()
        const roleStore = useRoleStore()
        const uiStore = useUiStore()
        await this.loadMessagesForRoleScene(
          roleStore.currentRoleId,
          uiStore.sceneId || 'default',
        )
        this.messagesHydrated = true
      },

      async runIdbMigrationIfNeeded() {
        if (localStorage.getItem(CHAT_STORAGE_MIGRATED_KEY) === 'true')
          return
        const fromLegacy = migrateMessageMapFromLocalStorage()
        const fromIdb = fromLegacy ?? (await loadMessageMapFromIdb())
        const map = fromIdb ? migrateMessageMapShape(fromIdb) : null
        if (fromLegacy && map)
          await migrateMonolithBlobToBuckets(map)
        if (map && Object.keys(map).length > 0) {
          try {
            const buckets = messageMapToImportBuckets(map)
            await migrateIndexeddbToBackend(buckets)
            localStorage.setItem(CHAT_STORAGE_MIGRATED_KEY, 'true')
          }
          catch (err) {
            console.warn('[chatStore] IndexedDB migration failed; will retry', err)
            return
          }
        }
        else {
          localStorage.setItem(CHAT_STORAGE_MIGRATED_KEY, 'true')
        }
      },

      async loadMessagesForRoleScene(roleId: string, sceneId: string) {
        const sid = sceneId || 'default'
        this.ensureLegacyMigrated(roleId)
        const sessionId = conversationSessionId(roleId, null)
        try {
          const stored = await fetchChatMessages(sessionId, 500, 0)
          const messages = stored
            .filter(m => m.sender === 'user' || m.sender === 'assistant')
            .map(storedMessageToChatMessage)
          if (!this.messageMap[roleId])
            this.messageMap[roleId] = {}
          this.messageMap[roleId]![sid] = messages
          await saveBucketToIdb(roleId, sid, messages)
        }
        catch (err) {
          console.warn('[chatStore] fetch_chat_messages failed; using IDB cache', err)
          const cached = await loadBucketFromIdb(roleId, sid)
          if (cached) {
            if (!this.messageMap[roleId])
              this.messageMap[roleId] = {}
            this.messageMap[roleId]![sid] = cached
          }
        }
        this.lastAssistantAside = rebuildLastAssistantAsideMap(this.messageMap)
        sanitizeAllSceneHistorySplits(this.sceneHistorySplitIndex, this.messageMap)
        repairSplitsSoCurrentSessionVisible(
          this.sceneHistorySplitIndex,
          this.messageMap,
        )
      },

      /** 退出前刷盘 IndexedDB（避免 300ms 防抖未写入）。 */
      async flushPendingPersist() {
        if (persistMessagesTimer) {
          clearTimeout(persistMessagesTimer)
          persistMessagesTimer = null
        }
        if (dirtyBuckets.size === 0)
          return
        const pending = new Set(dirtyBuckets)
        dirtyBuckets.clear()
        await saveDirtyBucketsToIdb(this.messageMap, pending)
      },

      /** 将旧版 messageMap[roleId] 为数组的结构迁入分桶（全表扫描，仅在 init / 单角色路径调用）。 */
      ensureLegacyMigrated(roleId: string) {
        const roleBucket = this.messageMap[roleId]
        if (isLegacyRoleBucket(roleBucket)) {
          const uiStore = useUiStore()
          const legacy = roleBucket
          this.messageMap[roleId] = { [uiStore.sceneId || 'default']: legacy }
        }
      },

      migrateAllLegacyMessageBuckets() {
        this.messageMap = migrateMessageMapShape(this.messageMap)
      },

      getMessageCountForRoleScene(roleId: string, sceneId: string): number {
        return roleSceneBucket(this.messageMap, roleId, sceneId).length
      },

      applySceneChange(
        nextSceneId: string,
        options?: { skipHistorySplit?: boolean },
      ) {
        const uiStore = useUiStore()
        const roleStore = useRoleStore()
        const prev = uiStore.sceneId
        const next = nextSceneId || 'default'
        if (prev !== next && !options?.skipHistorySplit) {
          const roleId = roleStore.currentRoleId
          this.ensureLegacyMigrated(roleId)
          if (!this.sceneHistorySplitIndex[roleId])
            this.sceneHistorySplitIndex[roleId] = {}
          const count = this.getMessageCountForRoleScene(roleId, next)
          this.sceneHistorySplitIndex[roleId][next] = count
        }
        uiStore.setScene(next)
        if (prev !== next) {
          void this.loadMessagesForRoleScene(roleStore.currentRoleId, next)
        }
      },

      addSystemMessage(content: string, sceneId?: string) {
        const roleStore = useRoleStore()
        const uiStore = useUiStore()
        const sid = sceneId ?? uiStore.sceneId ?? 'default'
        const ts = Date.now()
        const message: ChatMessage = {
          id: `sys-${ts}-${Math.random().toString(36).slice(2, 9)}`,
          role: 'system',
          content,
          timestamp: ts,
        }
        this.addMessage(roleStore.currentRoleId, sid, message)
      },

      addAssistantMessage(
        rawContent: string,
        emotion?: string,
        sceneId?: string,
        presenceVariant?: PresenceMode,
        replyIsFallback?: boolean,
        preSplit?: RoleplaySplit,
      ): RoleplaySplit {
        const roleStore = useRoleStore()
        const uiStore = useUiStore()
        const sid = sceneId ?? uiStore.sceneId ?? 'default'
        const ts = Date.now()
        const split = preSplit ?? splitRoleplayReply(rawContent)
        const aside = split.aside.trim()
        const dialogue = assistantDialogueFromSplit(rawContent, split)
        const message: ChatMessage = {
          id: `a-${ts}-${Math.random().toString(36).slice(2, 9)}`,
          role: 'assistant',
          content: dialogue,
          timestamp: ts,
          emotion,
          presenceVariant,
          replyIsFallback,
          ...(aside.length > 0 ? { aside } : {}),
        }
        this.addMessage(roleStore.currentRoleId, sid, message)
        return split
      },

      addUserMessage(content: string, sceneId?: string) {
        const roleStore = useRoleStore()
        const uiStore = useUiStore()
        const sid = sceneId ?? uiStore.sceneId ?? 'default'
        const ts = Date.now()
        const message: ChatMessage = {
          id: `u-${ts}-${Math.random().toString(36).slice(2, 9)}`,
          role: 'user',
          content,
          timestamp: ts,
        }
        this.addMessage(roleStore.currentRoleId, sid, message)
      },

      addMessage(
        roleId: string,
        sceneId: string,
        msg: ChatMessage,
        options?: { persistIdbCache?: boolean },
      ) {
        const sid = sceneId || 'default'
        const current = roleSceneBucket(this.messageMap, roleId, sid)
        const next = [...current, msg]
        const trimmed
          = next.length > MAX_MESSAGES_PER_CONVERSATION
            ? next.slice(-MAX_MESSAGES_PER_CONVERSATION)
            : next
        const removedFromHead = next.length - trimmed.length
        if (removedFromHead > 0) {
          adjustSplitAfterTrim(
            this.sceneHistorySplitIndex,
            roleId,
            sid,
            removedFromHead,
          )
        }
        this.messageMap[roleId]![sid] = trimmed
        clampSceneHistorySplitForBucket(
          this.sceneHistorySplitIndex,
          roleId,
          sid,
          trimmed.length,
        )
        syncLastAssistantAside(this.lastAssistantAside, roleId, sid, trimmed)
        if (options?.persistIdbCache !== false)
          schedulePersistMessages(this.messageMap, roleId, sid)
      },

      patchMessageById(
        roleId: string,
        sceneId: string,
        localId: string,
        patch: Partial<Pick<ChatMessage, 'id' | 'timestamp'>>,
      ) {
        const sid = sceneId || 'default'
        const bucket = roleSceneBucket(this.messageMap, roleId, sid)
        const idx = bucket.findIndex(m => m.id === localId)
        if (idx === -1)
          return
        bucket[idx] = { ...bucket[idx], ...patch }
      },

      editMessage(
        roleId: string,
        sceneId: string,
        messageId: string,
        patch: Partial<Pick<ChatMessage, 'content' | 'aside'>>,
      ) {
        const sid = sceneId || 'default'
        const bucket = roleSceneBucket(this.messageMap, roleId, sid)
        const idx = bucket.findIndex(m => m.id === messageId)
        if (idx === -1)
          return
        bucket[idx] = { ...bucket[idx], ...patch }
        syncLastAssistantAside(this.lastAssistantAside, roleId, sid, bucket)
        schedulePersistMessages(this.messageMap, roleId, sid)
      },

      deleteMessage(roleId: string, sceneId: string, messageId: string) {
        const sid = sceneId || 'default'
        const bucket = roleSceneBucket(this.messageMap, roleId, sid)
        const next = bucket.filter(m => m.id !== messageId)
        this.messageMap[roleId]![sid] = next
        syncLastAssistantAside(this.lastAssistantAside, roleId, sid, next)
        schedulePersistMessages(this.messageMap, roleId, sid)
      },

      clearMessages(roleId: string, sceneId: string) {
        const sid = sceneId || 'default'
        roleSceneBucket(this.messageMap, roleId, sid)
        this.messageMap[roleId]![sid] = []
        if (!this.sceneHistorySplitIndex[roleId])
          this.sceneHistorySplitIndex[roleId] = {}
        this.sceneHistorySplitIndex[roleId][sid] = 0
        this.lastAssistantAside[roleSceneAsideKey(roleId, sid)] = ''
        schedulePersistMessages(this.messageMap, roleId, sid)
      },

      async sendMessage(content: string, sceneId: string): Promise<SendMessageResponse> {
        const roleStore = useRoleStore()
        const roleId = roleStore.currentRoleId
        const sid = sceneId || 'default'
        const countBeforeTurn = this.getMessageCountForRoleScene(roleId, sid)
        const userLocalId = `u-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
        this.addMessage(roleId, sid, {
          id: userLocalId,
          role: 'user',
          content,
          timestamp: Date.now(),
        }, { persistIdbCache: false })
        this.isLoading = true
        const relationBefore = roleStore.roleInfo.relationState
        try {
          const res = await sendMessage({
            role_id: roleId,
            user_message: content,
            scene_id: sid || null,
          })
          if (res.user_message_id) {
            this.patchMessageById(roleId, sid, userLocalId, {
              id: res.user_message_id,
              timestamp: parseMessageTimestamp(res.user_message_timestamp),
            })
          }
          const pres = presentationFromSendResponse(res)
          const preSplit = splitRoleplayReply(pres.replyText)
          const aside = preSplit.aside.trim()
          const dialogue = assistantDialogueFromSplit(pres.replyText, preSplit)
          const assistantMsg: ChatMessage = {
            id: res.assistant_message_id
              ?? `a-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
            role: 'assistant',
            content: dialogue,
            timestamp: parseMessageTimestamp(res.assistant_message_timestamp),
            emotion: pres.assistantEmotionLabel,
            presenceVariant: pres.presenceVariant,
            replyIsFallback: pres.replyIsFallback,
            ...(aside.length > 0 ? { aside } : {}),
          }
          this.addMessage(roleId, sid, assistantMsg, { persistIdbCache: false })
          const split = preSplit
          useDebugStore().recordKnowledgeFromSend(res)
          roleStore.updateLocalAfterMessage(
            pres.assistantEmotionLabel,
            res.favorability_current,
          )
          if (res.relation_state) {
            const tip = getRelationUpgradeMessage(
              res.relation_state,
              relationBefore,
            )
            if (tip)
              this.addSystemMessage(tip, sid)
            roleStore.updateRelationState(res.relation_state)
          }
          hostEventBus.emitBuiltin('message:sent', {
            message: content,
            reply: assistantDialogueFromSplit(pres.replyText, split),
            reply_aside: split.aside,
          })
          const countAfterTurn = this.getMessageCountForRoleScene(roleId, sid)
          clampSceneHistorySplitForBucket(
            this.sceneHistorySplitIndex,
            roleId,
            sid,
            countAfterTurn,
            countBeforeTurn,
          )
          return res
        }
        finally {
          this.isLoading = false
        }
      },
    },
    persist: {
      pick: ['sceneHistorySplitIndex'],
    },
  },
)
