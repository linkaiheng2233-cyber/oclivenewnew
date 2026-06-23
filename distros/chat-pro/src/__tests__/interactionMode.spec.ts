import { describe, expect, it } from 'vitest'
import {
  isImmersiveMode,
  normalizeInteractionMode,
  packDefaultFromApi,
} from '@oclive/shared/utils/interactionMode'

describe('interactionMode', () => {
  it('normalizeInteractionMode maps immersive and defaults unknown to pure_chat', () => {
    expect(normalizeInteractionMode('pure_chat')).toBe('pure_chat')
    expect(normalizeInteractionMode('immersive')).toBe('immersive')
    expect(normalizeInteractionMode('other')).toBe('pure_chat')
    expect(normalizeInteractionMode(null)).toBe('pure_chat')
  })

  it('packDefaultFromApi keeps only canonical values', () => {
    expect(packDefaultFromApi('pure_chat')).toBe('pure_chat')
    expect(packDefaultFromApi('nope')).toBeNull()
  })

  it('isImmersiveMode is false for pure_chat', () => {
    expect(isImmersiveMode('immersive')).toBe(true)
    expect(isImmersiveMode('pure_chat')).toBe(false)
  })
})
