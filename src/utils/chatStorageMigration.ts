import type { RoleSceneMessageMap } from './chatMessageDb'
/**
 * One-time IndexedDB / localStorage chat migration (runs before backend is SSOT).
 * After `chat_storage_migrated` is set, this module is not used on the hot path.
 */
import { migrateIndexeddbToBackend } from '../api/chatStorage'
import {
  loadMessageMapFromIdb,
  messageMapToImportBuckets,
  migrateMessageMapFromLocalStorage,
  migrateMessageMapShape,
  migrateMonolithBlobToBuckets,

} from './chatMessageDb'

const CHAT_STORAGE_MIGRATED_KEY = 'chat_storage_migrated'

export { CHAT_STORAGE_MIGRATED_KEY }

/** True after one-time IDB/localStorage → backend migration (hot-path IDB writes are gated). */
export function isChatStorageMigrated(): boolean {
  return localStorage.getItem(CHAT_STORAGE_MIGRATED_KEY) === 'true'
}

/** Returns true when legacy local/IDB data was migrated to the backend this session. */
export async function runChatStorageMigrationIfNeeded(): Promise<boolean> {
  if (localStorage.getItem(CHAT_STORAGE_MIGRATED_KEY) === 'true')
    return false
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
      return true
    }
    catch (err) {
      console.warn('[chatStorageMigration] IndexedDB migration failed; will retry', err)
      return false
    }
  }
  localStorage.setItem(CHAT_STORAGE_MIGRATED_KEY, 'true')
  return false
}

export type { RoleSceneMessageMap }
