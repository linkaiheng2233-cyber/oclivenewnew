import { describe, expect, it } from 'vitest'
import { messageHintsUserIdentity } from '@oclive/shared/utils/identitySurpriseTriggers'

describe('identitySurpriseTriggers', () => {
  it('detects Chinese identity hints', () => {
    expect(messageHintsUserIdentity('其实我是你的经纪人')).toBe(true)
    expect(messageHintsUserIdentity('你好呀')).toBe(false)
  })

  it('detects English identity hints', () => {
    expect(messageHintsUserIdentity('I am your classmate')).toBe(true)
  })
})
