import { describe, expect, it } from 'vitest'
import type { ChatMessage } from './chatStore'
import { mergeMessagesFromServer } from './chatStoreLoad'

function msg(id: string, ts: number): ChatMessage {
  return { id, role: 'user', content: id, timestamp: ts }
}

describe('mergeMessagesFromServer', () => {
  it('keeps optimistic local messages when load resolves after send', () => {
    const server = [msg('srv-1', 100), msg('srv-2', 200)]
    const local = [...server, msg('u-999-local', 250)]
    const merged = mergeMessagesFromServer(server, local)
    expect(merged).toHaveLength(3)
    expect(merged.map(m => m.id)).toEqual(['srv-1', 'srv-2', 'u-999-local'])
  })

  it('returns server list when local has no extra rows', () => {
    const server = [msg('a', 1), msg('b', 2)]
    expect(mergeMessagesFromServer(server, server)).toEqual(server)
  })

  it('sorts merged rows by timestamp', () => {
    const server = [msg('s1', 10)]
    const local = [msg('opt', 5), msg('s1', 10)]
    const merged = mergeMessagesFromServer(server, local)
    expect(merged.map(m => m.id)).toEqual(['opt', 's1'])
  })
})
