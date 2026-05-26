import { describe, expect, it } from 'vitest'
import {
  applyLlmNodeConfig,
  ensureExpertRouteForLlmSlot,
  expertReferencedSlotKeys,
  slotHasExpertGenerateStep,
} from './expertNodeRouting'

describe('expertNodeRouting', () => {
  it('collects referenced slot keys from generate steps', () => {
    const doc = ensureExpertRouteForLlmSlot(null, 'main_llm')
    expect(expertReferencedSlotKeys(doc)).toEqual(new Set(['main_llm']))
    expect(slotHasExpertGenerateStep(doc, 'main_llm')).toBe(true)
  })

  it('merges llm params into routing doc', () => {
    const base = ensureExpertRouteForLlmSlot(null, 'a')
    const next = applyLlmNodeConfig(base, 'a', {
      llmSlotKey: 'a',
      temperature: 0.2,
      maxTokens: 1024,
    })
    const step = next.routes[0]!.steps.find(s => s.action === 'slot.a.generate')
    expect(step?.params).toMatchObject({
      temperature: 0.2,
      max_tokens: 1024,
    })
  })
})
