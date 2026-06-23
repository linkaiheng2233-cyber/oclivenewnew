// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  getCloudModelHistory,
  mergeCloudModelOptions,
  rememberCloudModel,
} from '@oclive/shared/composables/useCloudModelHistory'

describe('useCloudModelHistory', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('rememberCloudModel keeps newest first without duplicates', () => {
    rememberCloudModel('gpt-4o-mini')
    rememberCloudModel('deepseek-chat')
    rememberCloudModel('gpt-4o-mini')
    expect(getCloudModelHistory()).toEqual(['gpt-4o-mini', 'deepseek-chat'])
  })

  it('mergeCloudModelOptions dedupes current, history, and provider list', () => {
    const merged = mergeCloudModelOptions(
      ['gpt-4o', 'gpt-4o-mini'],
      ['deepseek-chat', 'gpt-4o-mini'],
      'custom-model',
    )
    expect(merged).toEqual(['custom-model', 'deepseek-chat', 'gpt-4o-mini', 'gpt-4o'])
  })
})
