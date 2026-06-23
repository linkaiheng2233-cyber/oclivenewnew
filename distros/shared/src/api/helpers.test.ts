import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it, vi } from 'vitest'

const { showToastMock } = vi.hoisted(() => ({
  showToastMock: vi.fn(),
}))

vi.mock('../composables/useAppToast', () => ({
  useAppToast: () => ({
    showToast: showToastMock,
    toast: { value: { show: false, type: 'info', message: '' } },
  }),
}))

vi.stubGlobal('window', {
  setTimeout: (fn: () => void) => {
    fn()
  },
})

import { ApiInvokeError, snakeToCamelKey, toastAsyncError, toCamelPayload } from './helpers'

const API_DIR = join(import.meta.dirname)

/** Top-level invoke payload keys must be camelCase (Tauri v1 IPC). */
function collectInvokeTopLevelKeys(filePath: string): string[] {
  const src = readFileSync(filePath, 'utf8')
  const keys: string[] = []
  const re = /invokeWithFriendlyError(?:<[^>]*>)?\(\s*['"`][^'"`]+['"`]\s*,\s*\{([^}]*)\}/gs
  for (const m of src.matchAll(re)) {
    const body = m[1] ?? ''
    for (const km of body.matchAll(/(?:^|[,{]\s*)(['"`])([a-zA-Z_][\w]*)\1\s*:/g)) {
      keys.push(km[2]!)
    }
  }
  const bareRe = /invoke(?:WithFriendlyError)?(?:<[^>]*>)?\(\s*['"`][^'"`]+['"`]\s*,\s*\{([^}]*)\}/gs
  for (const m of src.matchAll(bareRe)) {
    const body = m[1] ?? ''
    for (const km of body.matchAll(/(?:^|[,{]\s*)(['"`])([a-zA-Z_][\w]*)\1\s*:/g)) {
      keys.push(km[2]!)
    }
  }
  return keys
}

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

  it('src/api invoke top-level keys are camelCase', () => {
    const files = readdirSync(API_DIR).filter(f => f.endsWith('.ts') && !f.endsWith('.test.ts'))
    const offenders: string[] = []
    for (const file of files) {
      for (const key of collectInvokeTopLevelKeys(join(API_DIR, file))) {
        if (key.includes('_')) {
          offenders.push(`${file}: ${key}`)
        }
      }
    }
    expect(offenders).toEqual([])
  })

  it('toastAsyncError surfaces ApiInvokeError message via showToast', () => {
    showToastMock.mockClear()
    toastAsyncError(new ApiInvokeError({
      message: 'role not found',
      raw: '[ROLE_NOT_FOUND] Role not found: x',
      code: 'ROLE_NOT_FOUND',
    }))
    expect(showToastMock).toHaveBeenCalledWith('error', 'role not found')
  })
})
