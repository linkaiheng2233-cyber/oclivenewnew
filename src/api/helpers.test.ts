import { describe, expect, it } from 'vitest'
import { snakeToCamelKey, toCamelPayload } from './helpers'

describe('api/helpers', () => {
  it('snakeToCamelKey converts ipc keys', () => {
    expect(snakeToCamelKey('role_id')).toBe('roleId')
    expect(snakeToCamelKey('plugin_id')).toBe('pluginId')
  })

  it('toCamelPayload recurses shallow objects', () => {
    expect(toCamelPayload({ role_id: 'r1', nested: { session_id: 's1' } })).toEqual({
      roleId: 'r1',
      nested: { sessionId: 's1' },
    })
  })
})
