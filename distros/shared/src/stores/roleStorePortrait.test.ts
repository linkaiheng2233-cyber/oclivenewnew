import type { RoleInfo } from '@oclive/shared/api'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { useRoleStore } from './roleStore'

function roleInfo(roleId: string, name = roleId): RoleInfo {
  return {
    role_id: roleId,
    role_name: name,
  } as RoleInfo
}

describe('role portrait state ownership', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('preserves the latest catalog portrait across same-role info refreshes', () => {
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    role.applyRoleInfo(roleInfo('mumu', '沐沐'))
    role.updateLocalAfterMessage('happy', 12, {
      visualStateId: 'happy_moderate',
      portraitAssetPath: 'assets/images/happy_moderate.png',
    })

    role.applyRoleInfo(roleInfo('mumu', '沐沐（刷新）'))

    expect(role.roleInfo.visualStateId).toBe('happy_moderate')
    expect(role.roleInfo.portraitAssetPath).toBe('assets/images/happy_moderate.png')
    expect(role.roleInfo.portraitEmotion).toBe('happy')
  })

  it('clears transient portrait state when applying another role', () => {
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    role.applyRoleInfo(roleInfo('mumu'))
    role.updateLocalAfterMessage('sad', 3, {
      visualStateId: 'sad_severe',
      portraitAssetPath: 'assets/images/sad_severe.png',
    })

    role.currentRoleId = 'other'
    role.applyRoleInfo(roleInfo('other'))

    expect(role.roleInfo.visualStateId).toBeUndefined()
    expect(role.roleInfo.portraitAssetPath).toBeUndefined()
    expect(role.portraitStateRoleId).toBe('')
    expect(role.roleInfo.portraitEmotion).toBeUndefined()
  })

  it('keeps the latest catalog portrait when a compatibility response omits visual fields', () => {
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    role.applyRoleInfo(roleInfo('mumu'))
    role.updateLocalAfterMessage('happy', 12, {
      visualStateId: 'happy_mild',
      portraitAssetPath: 'assets/images/happy_mild.png',
    })

    role.updateLocalAfterMessage('neutral', 13, {
      visualStateId: undefined,
      portraitAssetPath: undefined,
    })

    expect(role.roleInfo.visualStateId).toBe('happy_mild')
    expect(role.roleInfo.portraitAssetPath).toBe('assets/images/happy_mild.png')
    expect(role.roleInfo.portraitEmotion).toBe('happy')
  })

  it('keeps the display hint independent of the refreshed analysis emotion', () => {
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    role.applyRoleInfo(roleInfo('mumu'))
    role.updateLocalAfterMessage('shy', 12, {
      visualStateId: 'shy_severe',
      portraitAssetPath: 'assets/images/shy_severe.png',
    })
    role.applyRoleInfo({ ...roleInfo('mumu'), current_emotion: 'neutral' })
    // Same operation performed by the periodic role snapshot poll.
    role.roleInfo.currentEmotion = 'neutral'
    expect(role.roleInfo.portraitEmotion).toBe('shy')
    expect(role.roleInfo.portraitAssetPath).toBe('assets/images/shy_severe.png')
    expect(role.roleInfo.currentEmotion).toBe('neutral')
  })

  it('also preserves legacy filename expressions on a same-role refresh', () => {
    const role = useRoleStore()
    role.currentRoleId = 'mumu'
    role.applyRoleInfo(roleInfo('mumu'))
    role.updateLocalAfterMessage('excited', 0)
    role.applyRoleInfo({ ...roleInfo('mumu'), current_emotion: 'neutral' })
    expect(role.roleInfo.portraitEmotion).toBe('excited')
    expect(role.roleInfo.portraitAssetPath).toBeUndefined()
  })
})
