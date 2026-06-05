import type { PresenceMode, SendMessageResponse } from '../api'
import type { RoleSceneMessageMap } from '../utils/chatMessageDb'
import type { RoleplaySplit } from '../utils/roleplayReplySplit'
import { defineStore } from 'pinia'
import {
  sendMessage,
} from '../api'
import {
  getChatStorageCapabilities,
} from '../api/chatStorage'
import { hostEventBus } from '../lib/hostEventBus'
import {
  bucketMapKey,
  migrateMessageMapShape,

  saveDirtyBucketsToIdb,
} from '../utils/chatMessageDb'

import { isChatStorageMigrated, runChatStorageMigrationIfNeeded } from '../utils/chatStorageMigration'
import { getRelationUpgradeMessage } from '../utils/relation'
import { presentationFromSendResponse } from '../utils/replyPresentation'
import {
  assistantDialogueFromSplit,

  splitRoleplayReply,
} from '../utils/roleplayReplySplit'
import { loadRoleSceneMessages, parseMessageTimestamp } from './chatStoreLoad'
import { useDebugStore } from './debugStore'
import { useRoleStore } from './roleStore'
import { useUiStore } from './uiStore'

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  /** assistant: bot emotion this turn (lowercase); usually omitted for user */
  emotion?: string
  /** assistant: remote presence mode (for styling) */
  presenceVariant?: PresenceMode
  /** Fallback short reply when primary LLM fails (matches backend `reply_is_fallback`) */
  replyIsFallback?: boolean
  /** Narration/inner thought/action split from main reply (assistant only; main content is dialogue) */
  aside?: string
}

/** Populated from `get_chat_storage_capabilities` on hydrate; backend SSOT is `DEFAULT_MAX_MESSAGES`. */
const FALLBACK_MAX_MESSAGES_PER_CONVERSATION = 500

/** Message count already in bucket when entering a scene; indices below this are folded "history" (per role × scene) */
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

/** Prevent split ≥ count from blanking main chat (all new messages folded into history) */
function clampSceneHistorySplitForBucket(
  splitIndex: SceneHistorySplitIndex,
  roleId: string,
  sceneId: string,
  messageCount: number,
  /** Count before this turn's send; fallback if split hides just-sent messages */
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

/** After restart, if split folds all messages into history, main chat is blank; restore direct display. */
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
  if (isChatStorageMigrated())
    return
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
      /** Bumped on each scene load; stale async results are ignored. */
      messageLoadGeneration: 0,
      /** Role×scene bucket currently loading (avoids showing another scene's messages). */
      messagesLoadingKey: null as string | null,
      /** Per-session UI cap; synced from backend capabilities on hydrate. */
      messageCapPerSession: FALLBACK_MAX_MESSAGES_PER_CONVERSATION,
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
      isMessagesLoadingFor: (state) => {
        return (roleId: string, sceneId: string): boolean =>
          state.messagesLoadingKey === bucketMapKey(roleId, sceneId || 'default')
      },
    },
    actions: {
      /** Startup: migrate IndexedDB → backend, then load current role×scene from API (IDB cache fallback on failure). */
      async hydrateFromStorage() {
        if (this.messagesHydrated)
          return
        await this.runIdbMigrationIfNeeded()
        try {
          const caps = await getChatStorageCapabilities()
          if (caps.default_max_messages_per_session > 0)
            this.messageCapPerSession = caps.default_max_messages_per_session
        }
        catch {
          // keep fallback cap
        }
        const roleStore = useRoleStore()
        const uiStore = useUiStore()
        await this.loadMessagesForRoleScene(
          roleStore.currentRoleId,
          uiStore.sceneId || 'default',
        )
        this.messagesHydrated = true
      },

      async runIdbMigrationIfNeeded() {
        await runChatStorageMigrationIfNeeded()
      },

      async loadMessagesForRoleScene(roleId: string, sceneId: string) {
        const sid = sceneId || 'default'
        const loadKey = bucketMapKey(roleId, sid)
        const gen = ++this.messageLoadGeneration
        this.messagesLoadingKey = loadKey
        this.ensureLegacyMigrated(roleId)
        try {
          await loadRoleSceneMessages(this.messageMap, roleId, sid)
        }
        finally {
          if (this.messagesLoadingKey === loadKey)
            this.messagesLoadingKey = null
        }
        if (gen !== this.messageLoadGeneration)
          return
        this.lastAssistantAside = rebuildLastAssistantAsideMap(this.messageMap)
        sanitizeAllSceneHistorySplits(this.sceneHistorySplitIndex, this.messageMap)
        repairSplitsSoCurrentSessionVisible(
          this.sceneHistorySplitIndex,
          this.messageMap,
        )
      },

      /** Flush IndexedDB before exit (avoid 300ms debounce not yet written). */
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

      /** Migrate legacy messageMap[roleId] array shape into buckets (full scan; init / single-role path only). */
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
        if (prev !== next) {
          this.messagesLoadingKey = bucketMapKey(roleStore.currentRoleId, next)
          uiStore.setScene(next)
          void this.loadMessagesForRoleScene(roleStore.currentRoleId, next)
        }
        else {
          uiStore.setScene(next)
        }
      },

      messagesForDisplay(roleId: string, sceneId: string): ChatMessage[] {
        return this.messagesForRoleScene(roleId, sceneId)
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
          = next.length > this.messageCapPerSession
            ? next.slice(-this.messageCapPerSession)
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
        const next = [...bucket]
        next[idx] = { ...next[idx], ...patch }
        this.messageMap[roleId]![sid] = next
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
        catch (err) {
          this.deleteMessage(roleId, sid, userLocalId)
          throw err
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
