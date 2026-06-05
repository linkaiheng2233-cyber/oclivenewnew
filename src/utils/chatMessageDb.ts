import type { ChatMessage } from '../stores/chatStore'
import { del, get, set, setMany } from 'idb-keyval'

const CHAT_STORAGE_MIGRATED_KEY = 'chat_storage_migrated'

function idbHotPathWritesEnabled(): boolean {
  try {
    return localStorage.getItem(CHAT_STORAGE_MIGRATED_KEY) !== 'true'
  }
  catch {
    return true
  }
}

export type RoleSceneMessageMap = Record<string, Record<string, ChatMessage[]>>

/** Legacy monolithic blob key (pre bucketed persistence). */
export const IDB_MONOLITH_KEY = 'oclive-chat-message-map'
export const IDB_BUCKET_INDEX_KEY = 'oclive-chat-bucket-index'
const LEGACY_PINIA_KEY = 'chat'

export function bucketMapKey(roleId: string, sceneId: string): string {
  return `${roleId}:${sceneId || 'default'}`
}

function bucketStorageKey(roleId: string, sceneId: string): string {
  return `oclive-chat-bucket:${bucketMapKey(roleId, sceneId)}`
}

function parseBucketMapKey(key: string): { roleId: string, sceneId: string } {
  const idx = key.indexOf(':')
  if (idx <= 0)
    throw new Error(`invalid bucket map key: ${key}`)
  return { roleId: key.slice(0, idx), sceneId: key.slice(idx + 1) }
}

function messagesForBucket(
  map: RoleSceneMessageMap,
  roleId: string,
  sceneId: string,
): ChatMessage[] {
  const roleBucket = map[roleId]
  if (!roleBucket)
    return []
  if (Array.isArray(roleBucket))
    return sceneId === 'default' ? roleBucket : []
  return roleBucket[sceneId] ?? []
}

/** Migrate legacy messageMap[roleId] array shape into bucketed map. */
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

export async function migrateMonolithBlobToBuckets(map: RoleSceneMessageMap): Promise<void> {
  const indexKeys: string[] = []
  const entries: [string, ChatMessage[]][] = []
  for (const [roleId, roleBucket] of Object.entries(map)) {
    if (Array.isArray(roleBucket)) {
      const mapKey = bucketMapKey(roleId, 'default')
      indexKeys.push(mapKey)
      entries.push([bucketStorageKey(roleId, 'default'), roleBucket])
    }
    else {
      for (const [sceneId, messages] of Object.entries(roleBucket)) {
        const mapKey = bucketMapKey(roleId, sceneId)
        indexKeys.push(mapKey)
        entries.push([bucketStorageKey(roleId, sceneId), messages])
      }
    }
  }
  if (entries.length > 0)
    await setMany(entries)
  await set(IDB_BUCKET_INDEX_KEY, indexKeys)
}

export async function loadMessageMapFromIdb(): Promise<RoleSceneMessageMap | null> {
  const monolith = await get<unknown>(IDB_MONOLITH_KEY)
  if (monolith && typeof monolith === 'object') {
    const migrated = migrateMessageMapShape(monolith)
    await migrateMonolithBlobToBuckets(migrated)
    await del(IDB_MONOLITH_KEY)
    return migrated
  }
  const index = await get<string[]>(IDB_BUCKET_INDEX_KEY)
  if (!index || index.length === 0)
    return null
  const out: RoleSceneMessageMap = {}
  for (const mapKey of index) {
    const { roleId, sceneId } = parseBucketMapKey(mapKey)
    const messages = await get<ChatMessage[]>(bucketStorageKey(roleId, sceneId))
    if (!messages)
      continue
    if (!out[roleId])
      out[roleId] = {}
    out[roleId]![sceneId] = messages
  }
  return Object.keys(out).length > 0 ? out : null
}

/** Persist only dirty role×scene buckets; returns number of bucket writes (setMany calls). */
export async function saveDirtyBucketsToIdb(
  map: RoleSceneMessageMap,
  dirtyKeys: ReadonlySet<string>,
): Promise<number> {
  if (!idbHotPathWritesEnabled())
    return 0
  if (dirtyKeys.size === 0)
    return 0
  const existing = new Set(await get<string[]>(IDB_BUCKET_INDEX_KEY) ?? [])
  const entries: [string, ChatMessage[]][] = []
  for (const mapKey of dirtyKeys) {
    const { roleId, sceneId } = parseBucketMapKey(mapKey)
    entries.push([
      bucketStorageKey(roleId, sceneId),
      messagesForBucket(map, roleId, sceneId),
    ])
    existing.add(mapKey)
  }
  await setMany(entries)
  await set(IDB_BUCKET_INDEX_KEY, [...existing])
  return entries.length
}

/**
 * Migrate messages from pinia-plugin-persistedstate legacy localStorage; strip messageMap on success to avoid duplicate size.
 */
export async function loadBucketFromIdb(
  roleId: string,
  sceneId: string,
): Promise<ChatMessage[] | null> {
  const messages = await get<ChatMessage[]>(bucketStorageKey(roleId, sceneId || 'default'))
  return messages?.length ? messages : null
}

export async function saveBucketToIdb(
  roleId: string,
  sceneId: string,
  messages: ChatMessage[],
): Promise<void> {
  if (!idbHotPathWritesEnabled())
    return
  const mapKey = bucketMapKey(roleId, sceneId || 'default')
  const map: RoleSceneMessageMap = { [roleId]: { [sceneId || 'default']: messages } }
  await saveDirtyBucketsToIdb(map, new Set([mapKey]))
}

/** Build migration payloads from in-memory / IDB message map. */
export function messageMapToImportBuckets(map: RoleSceneMessageMap): import('../api/chatStorage').ImportChatBucket[] {
  const buckets: import('../api/chatStorage').ImportChatBucket[] = []
  for (const [roleId, roleBucket] of Object.entries(map)) {
    if (Array.isArray(roleBucket)) {
      buckets.push({
        role_id: roleId,
        scene_id: 'default',
        messages: roleBucket.map(m => ({
          role: m.role,
          content: m.content,
          timestamp: m.timestamp,
          id: m.id,
        })),
      })
    }
    else {
      for (const [sceneId, messages] of Object.entries(roleBucket)) {
        buckets.push({
          role_id: roleId,
          scene_id: sceneId,
          messages: messages.map(m => ({
            role: m.role,
            content: m.content,
            timestamp: m.timestamp,
            id: m.id,
          })),
        })
      }
    }
  }
  return buckets
}

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
