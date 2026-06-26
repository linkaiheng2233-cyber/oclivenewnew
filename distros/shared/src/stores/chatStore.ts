import type { PresenceMode, SendMessageResponse } from '@oclive/shared/api'
import type { RoleSceneMessageMap } from '@oclive/shared/utils/chatMessageDb'
import type { RoleplaySplit } from '@oclive/shared/utils/roleplayReplySplit'
import { defineStore } from 'pinia'
import {
  getChatStorageCapabilities,
} from '@oclive/shared/api/chatStorage'
import {
  bucketMapKey,
  migrateMessageMapShape,

  saveDirtyBucketsToIdb,
} from '@oclive/shared/utils/chatMessageDb'

import { isChatStorageMigrated, runChatStorageMigrationIfNeeded } from '@oclive/shared/utils/chatStorageMigration'
import {
  assistantDialogueFromSplit,

  splitRoleplayReply,
} from '@oclive/shared/utils/roleplayReplySplit'
import { loadRoleSceneMessages } from './chatStoreLoad'
import { sendChatStoreMessage } from './chatStoreSend'
import { useRoleStore } from './roleStore'
import { useUiStore } from './uiStore'
import { effectiveChatSceneId } from '../utils/pureChatScene'

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
  /** True while SSE tokens are still arriving */
  streaming?: boolean
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
        ?? splitRoleplayReply(m.content).aside.trim()
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

/** After restart, fold all loaded messages into history so the main chat starts blank. */
function beginNewChatSessionOnRestart(
  splitIndex: SceneHistorySplitIndex,
  roleId: string,
  sceneId: string,
  messageCount: number,
): void {
  if (messageCount <= 0)
    return
  if (!splitIndex[roleId])
    splitIndex[roleId] = {}
  splitIndex[roleId][sceneId || 'default'] = messageCount
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
        const sceneId = effectiveChatSceneId(
          roleStore.roleInfo.interactionMode,
          uiStore.sceneId || 'default',
        )
        await this.loadMessagesForRoleScene(
          roleStore.currentRoleId,
          sceneId,
        )
        beginNewChatSessionOnRestart(
          this.sceneHistorySplitIndex,
          roleStore.currentRoleId,
          sceneId,
          this.getMessageCountForRoleScene(
            roleStore.currentRoleId,
            sceneId,
          ),
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
        const bucket = roleSceneBucket(this.messageMap, roleId, sid)
        syncLastAssistantAside(this.lastAssistantAside, roleId, sid, bucket)
        sanitizeAllSceneHistorySplits(this.sceneHistorySplitIndex, this.messageMap)
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
        const bucket = roleSceneBucket(this.messageMap, roleId, sid)
        bucket.push(msg)
        let removedFromHead = 0
        if (bucket.length > this.messageCapPerSession) {
          removedFromHead = bucket.length - this.messageCapPerSession
          bucket.splice(0, removedFromHead)
        }
        if (removedFromHead > 0) {
          adjustSplitAfterTrim(
            this.sceneHistorySplitIndex,
            roleId,
            sid,
            removedFromHead,
          )
        }
        clampSceneHistorySplitForBucket(
          this.sceneHistorySplitIndex,
          roleId,
          sid,
          bucket.length,
        )
        syncLastAssistantAside(this.lastAssistantAside, roleId, sid, bucket)
        if (options?.persistIdbCache !== false)
          schedulePersistMessages(this.messageMap, roleId, sid)
      },

      patchMessageById(
        roleId: string,
        sceneId: string,
        localId: string,
        patch: Partial<Pick<ChatMessage, 'id' | 'timestamp' | 'content' | 'streaming' | 'emotion' | 'aside'>>,
      ) {
        const sid = sceneId || 'default'
        const bucket = roleSceneBucket(this.messageMap, roleId, sid)
        const idx = bucket.findIndex(m => m.id === localId)
        if (idx === -1)
          return
        Object.assign(bucket[idx]!, patch)
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

      async sendMessage(content: string, sceneId: string): Promise<SendMessageResponse | undefined> {
        return sendChatStoreMessage(
          {
            sceneHistorySplitIndex: this.sceneHistorySplitIndex,
            setLoading: loading => (this.isLoading = loading),
            getMessageCountForRoleScene: (roleId, sid) => this.getMessageCountForRoleScene(roleId, sid),
            addMessage: (roleId, sid, msg, options) => this.addMessage(roleId, sid, msg, options),
            patchMessageById: (roleId, sid, localId, patch) =>
              this.patchMessageById(roleId, sid, localId, patch),
            deleteMessage: (roleId, sid, messageId) => this.deleteMessage(roleId, sid, messageId),
            addSystemMessage: (message, sid) => this.addSystemMessage(message, sid),
            clampSceneHistorySplitForBucket,
          },
          content,
          sceneId,
        )
      },
    },
  },
)
