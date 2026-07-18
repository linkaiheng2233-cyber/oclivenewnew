import type { RoleSceneMessageMap } from './chatMessageDb'

import { get, setMany } from 'idb-keyval'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  bucketMapKey,
  IDB_BUCKET_INDEX_KEY,
  IDB_MONOLITH_KEY,
  loadMessageMapFromIdb,
  migrateMonolithBlobToBuckets,

  saveDirtyBucketsToIdb,
} from './chatMessageDb'

const idbStore = new Map<string, unknown>()

vi.mock('idb-keyval', () => ({
  get: vi.fn((key: string) => Promise.resolve(idbStore.get(key))),
  set: vi.fn((key: string, value: unknown) => {
    idbStore.set(key, value)
    return Promise.resolve()
  }),
  setMany: vi.fn((entries: [string, unknown][]) => {
    for (const [key, value] of entries)
      idbStore.set(key, value)
    return Promise.resolve()
  }),
  del: vi.fn((key: string) => {
    idbStore.delete(key)
    return Promise.resolve()
  }),
}))

describe('chatMessageDb bucket persistence', () => {
  beforeEach(() => {
    idbStore.clear()
    vi.mocked(setMany).mockClear()
    vi.mocked(get).mockClear()
  })

  it('migrates legacy monolithic blob into per-bucket keys on load', async () => {
    const legacy: RoleSceneMessageMap = {
      roleA: {
        default: [{ id: '1', role: 'user', content: 'hi', timestamp: 1 }],
      },
    }
    idbStore.set(IDB_MONOLITH_KEY, legacy)
    const loaded = await loadMessageMapFromIdb()
    expect(loaded?.roleA?.default).toHaveLength(1)
    expect(idbStore.has(IDB_MONOLITH_KEY)).toBe(false)
    expect(idbStore.get(IDB_BUCKET_INDEX_KEY)).toEqual(['roleA:default'])
  })

  it('issues one IDB write per message when persisting dirty buckets (100 messages)', async () => {
    const map: RoleSceneMessageMap = { role1: { default: [] } }
    const bucketKey = bucketMapKey('role1', 'default')
    for (let i = 0; i < 100; i++) {
      map.role1!.default!.push({
        id: `m${i}`,
        role: 'user',
        content: `c${i}`,
        timestamp: i,
      })
      await saveDirtyBucketsToIdb(map, new Set([bucketKey]))
    }
    expect(setMany).toHaveBeenCalledTimes(100)
  })

  it('migrateMonolithBlobToBuckets slices in-memory map without monolith key', async () => {
    const map: RoleSceneMessageMap = {
      roleX: { scene1: [{ id: 'a', role: 'assistant', content: 'x', timestamp: 0 }] },
    }
    await migrateMonolithBlobToBuckets(map)
    expect(idbStore.get(IDB_BUCKET_INDEX_KEY)).toEqual(['roleX:scene1'])
  })
})
