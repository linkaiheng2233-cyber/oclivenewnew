import { describe, expect, it } from 'vitest'

describe('main repo smoke', () => {
  it('vitest pipeline is wired', () => {
    expect(1 + 1).toBe(2)
  })
})
