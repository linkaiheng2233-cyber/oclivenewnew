// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest'
import {
  isChatStreamEnabled,
  setChatStreamEnabled,
} from '@oclive/shared/utils/chatStreamSettings'

describe('chatStreamSettings', () => {
  afterEach(() => {
    localStorage.removeItem('oclive.chat.streamEnabled')
  })

  it('defaults stream enabled when unset', () => {
    expect(isChatStreamEnabled()).toBe(true)
  })

  it('persists disabled stream preference', () => {
    setChatStreamEnabled(false)
    expect(isChatStreamEnabled()).toBe(false)
    setChatStreamEnabled(true)
    expect(isChatStreamEnabled()).toBe(true)
  })
})
