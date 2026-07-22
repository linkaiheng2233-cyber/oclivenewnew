import { describe, expect, it } from 'vitest'
import {
  extractFirstSpeakableChunk,
  extractSpeakableChunkFrom,
  remainderAfterSpokenPrefix,
} from './extractFirstSpeakableChunk'

describe('extractFirstSpeakableChunk', () => {
  it('waits for punctuation or the first-chunk cap instead of emitting tiny fragments', () => {
    expect(extractFirstSpeakableChunk('你好呀', { isFirst: true })).toBeNull()
    expect(extractFirstSpeakableChunk('你好呀，', { isFirst: true })).toBe('你好呀，')
    expect(extractFirstSpeakableChunk('一二三四五六七八', { isFirst: true })).toBe('一二三四五六七八')
  })

  it('waits for minimum length', () => {
    expect(extractFirstSpeakableChunk('你好')).toBeNull()
    expect(extractFirstSpeakableChunk('你好呀')).toBeNull()
  })

  it('prefers earliest break for lower speak latency', () => {
    expect(extractFirstSpeakableChunk('你好，世界！后面还有')).toBe('你好，')
  })

  it('breaks early on weak clause punctuation', () => {
    expect(extractFirstSpeakableChunk('你好呀，今天天气')).toBe('你好呀，')
  })

  it('falls back to char cap without punctuation', () => {
    const long = '这是一段没有任何标点但已经很长的话继续输出'
    const chunk = extractFirstSpeakableChunk(long)
    expect(chunk).toBe(long.slice(0, 12))
  })

  it('first chunk caps earlier without punctuation', () => {
    const long = '这是一段没有任何标点但已经很长的话继续输出'
    const chunk = extractFirstSpeakableChunk(long, { isFirst: true })
    expect(chunk).toBe(long.slice(0, 8))
  })

  it('first chunk exits on weak punctuation immediately', () => {
    expect(extractFirstSpeakableChunk('你好呀，后面还有', { isFirst: true })).toBe('你好呀，')
  })

  it('extracts sequential chunks from a stream buffer', () => {
    const text = '你好呀，今天很好！明天见。'
    const first = extractSpeakableChunkFrom(text, 0)
    expect(first?.chunk).toBe('你好呀，')
    const second = extractSpeakableChunkFrom(text, first!.endIndex)
    expect(second?.chunk).toBe('今天很好！')
    const third = extractSpeakableChunkFrom(text, second!.endIndex)
    expect(third?.chunk).toBe('明天见。')
  })

  it('computes remainder after streamed prefix', () => {
    const full = '你好呀，今天很好！明天见。'
    expect(remainderAfterSpokenPrefix(full, '你好呀，今天很好！')).toBe('明天见。')
  })

  it('recovers chunk position when indexOf would miss trimmed chunk', () => {
    const text = '  你好呀，今天很好！'
    const first = extractSpeakableChunkFrom(text, 0)
    expect(first?.chunk).toBe('你好呀，')
    expect(first!.endIndex).toBeGreaterThan(0)
    const second = extractSpeakableChunkFrom(text, first!.endIndex)
    expect(second?.chunk).toBe('今天很好！')
  })

  it('flushes trailing text after sequential extraction', () => {
    const text = '你好呀，今天很好！哦'
    let idx = 0
    const chunks: string[] = []
    while (true) {
      const next = extractSpeakableChunkFrom(text, idx)
      if (!next)
        break
      chunks.push(next.chunk)
      idx = next.endIndex
    }
    expect(chunks).toEqual(['你好呀，', '今天很好！'])
    const tail = text.slice(idx).trim()
    expect(tail).toBe('哦')
    expect(remainderAfterSpokenPrefix(text, `${chunks.join('')}${tail}`)).toBe('')
  })
})
