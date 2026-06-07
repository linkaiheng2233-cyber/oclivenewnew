import { describe, expect, it } from 'vitest'
import {
  isImmersiveMode,
  normalizeInteractionMode,
  packDefaultFromApi,
} from '../utils/interactionMode'

describe('interactionMode', () => {
  it('normalizeInteractionMode maps pure_chat and defaults unknown to immersive', () => {
    expect(normalizeInteractionMode('pure_chat')).toBe('pure_chat')
    expect(normalizeInteractionMode('immersive')).toBe('immersive')
    expect(normalizeInteractionMode('other')).toBe('immersive')
    expect(normalizeInteractionMode(null)).toBe('immersive')
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
