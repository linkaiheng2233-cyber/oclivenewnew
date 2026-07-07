import { describe, expect, it } from 'vitest'
import {
  StreamingVoiceChunker,
  stableStreamingPrefix,
} from './streamingVoiceChunker'

describe('stableStreamingPrefix', () => {
  it('waits for closing paren across tokens', () => {
    expect(stableStreamingPrefix('你好（心里默默')).toBe('你好')
    expect(stableStreamingPrefix('你好（心里默默）呀')).toBe('你好（心里默默）呀')
  })

  it('waits for incomplete tag line', () => {
    expect(stableStreamingPrefix('你好\n【动作')).toBe('你好\n')
    expect(stableStreamingPrefix('你好\n【动作】举手')).toBe('你好\n【动作】举手')
    expect(stableStreamingPrefix('你好\n【动作】举手\n再见')).toBe('你好\n【动作】举手\n再见')
  })
})

describe('StreamingVoiceChunker', () => {
  it('skips inner monologue in parentheses', () => {
    const chunker = new StreamingVoiceChunker()
    const chunks = chunker.push('你好呀（心里默默）今天很好！')
    expect(chunks.join('')).not.toMatch(/心里/)
    expect(chunks.join('')).toMatch(/你好/)
  })

  it('skips standalone action tag lines', () => {
    const chunker = new StreamingVoiceChunker()
    const chunks = chunker.push('【动作】转身\n嗯，我在呢。')
    expect(chunks.join('')).not.toMatch(/转身/)
    expect(chunks.some(c => c.includes('我在'))).toBe(true)
  })

  it('incrementally emits across token boundaries', () => {
    const chunker = new StreamingVoiceChunker()
    const a = chunker.push('你好')
    expect(a).toEqual([])
    const b = chunker.push('你好呀，')
    expect(b.length).toBeGreaterThan(0)
    const c = chunker.push('你好呀，今天很好！')
    expect(c.join('')).toMatch(/今天/)
  })

  it('flush speaks dialogue tail only', () => {
    const chunker = new StreamingVoiceChunker()
    chunker.push('你好呀，今天很好！')
    const tail = chunker.flush('你好呀，今天很好！哦')
    expect(tail).toEqual(['哦'])
  })
})
