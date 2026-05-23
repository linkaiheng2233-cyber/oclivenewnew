import { get, set } from 'idb-keyval'
import type { ChatMessage } from '../stores/chatStore'

export type RoleSceneMessageMap = Record<string, Record<string, ChatMessage[]>>

const IDB_KEY = 'oclive-chat-message-map'
const LEGACY_PINIA_KEY = 'chat'

/** 将旧版 messageMap[roleId] 为数组的结构迁入分桶 map。 */
export function migrateMessageMapShape(raw: unknown): RoleSceneMessageMap {
  if (!raw || typeof raw !== 'object')
    return {}
  const out: RoleSceneMessageMap = {}
  for (const [roleId, bucket] of Object.entries(raw as Record<string, unknown>)) {
    if (Array.isArray(bucket)) {
      out[roleId] = { default: bucket as ChatMessage[] }
    }
    else if (bucket && typeof bucket === 'object') {
      out[roleId] = bucket as Record<string, ChatMessage[]>
    }
  }
  return out
}

export async function loadMessageMapFromIdb(): Promise<RoleSceneMessageMap | null> {
  const v = await get<RoleSceneMessageMap>(IDB_KEY)
  if (!v || typeof v !== 'object')
    return null
  return migrateMessageMapShape(v)
}

export async function saveMessageMapToIdb(map: RoleSceneMessageMap): Promise<void> {
  await set(IDB_KEY, map)
}

/**
 * 从 pinia-plugin-persistedstate 遗留的 localStorage 迁移消息；成功后剥离 messageMap 避免重复体积。
 */
export function migrateMessageMapFromLocalStorage(): RoleSceneMessageMap | null {
  try {
    const raw = localStorage.getItem(LEGACY_PINIA_KEY)
    if (!raw)
      return null
    const parsed = JSON.parse(raw) as { messageMap?: unknown }
    const migrated = migrateMessageMapShape(parsed.messageMap)
    if (Object.keys(migrated).length === 0)
      return null
    delete parsed.messageMap
    const rest = Object.keys(parsed).filter(k => k !== 'messageMap')
    if (rest.length === 0)
      localStorage.removeItem(LEGACY_PINIA_KEY)
    else
      localStorage.setItem(LEGACY_PINIA_KEY, JSON.stringify(parsed))
    return migrated
  }
  catch {
    return null
  }
}
