import { describe, expect, it } from 'vitest'
import {
  extractFirstSpeakableChunk,
  extractSpeakableChunkFrom,
  remainderAfterSpokenPrefix,
} from './extractFirstSpeakableChunk'

describe('extractFirstSpeakableChunk', () => {
  it('emits only at a natural punctuation boundary', () => {
    expect(extractFirstSpeakableChunk('你好呀', { isFirst: true })).toBeNull()
    expect(extractFirstSpeakableChunk('你好呀，', { isFirst: true })).toBe('你好呀，')
    expect(extractFirstSpeakableChunk('一二三四五六', { isFirst: true })).toBeNull()
  })

  it('waits for minimum length', () => {
    expect(extractFirstSpeakableChunk('你好')).toBeNull()
    expect(extractFirstSpeakableChunk('你好呀')).toBeNull()
  })

  it('uses the earliest valid punctuation boundary', () => {
    expect(extractFirstSpeakableChunk('你好，世界！后面还有')).toBe('你好，')
  })

  it('uses weak clause punctuation for later chunks too', () => {
    expect(extractFirstSpeakableChunk('你好呀，今天天气')).toBe('你好呀，')
  })

  it('does not split unpunctuated CJK text at a character cap', () => {
    const long = '这是一段没有任何标点但已经很长的话继续输出'
    expect(extractFirstSpeakableChunk(long)).toBeNull()
  })

  it('does not split an unpunctuated first chunk either', () => {
    const long = '这是一段没有任何标点但已经很长的话继续输出'
    expect(extractFirstSpeakableChunk(long, { isFirst: true })).toBeNull()
  })

  it('waits for distant punctuation instead of cutting between characters', () => {
    expect(
      extractFirstSpeakableChunk('一二三四五六七八九。', { isFirst: true }),
    ).toBe('一二三四五六七八九。')
  })

  it('keeps a near-complete first sentence together', () => {
    expect(
      extractFirstSpeakableChunk('今天买了模型。', { isFirst: true }),
    ).toBe('今天买了模型。')
  })

  it('keeps a long later phrase intact until punctuation', () => {
    const text = '一二三四五六七八九十一二三四五六七八九十二三四。'
    expect(extractFirstSpeakableChunk(text)).toBe(text)
  })

  it('first chunk exits on weak punctuation immediately', () => {
    expect(extractFirstSpeakableChunk('你好呀，后面还有', { isFirst: true })).toBe('你好呀，')
  })

  it('skips a too-short leading clause before the first voice chunk', () => {
    expect(
      extractFirstSpeakableChunk('喂，爸，晚', { isFirst: true }),
    ).toBe('喂，爸，')
  })

  it('supports punctuation from multiple writing systems', () => {
    expect(extractFirstSpeakableChunk('Hello, world.')).toBe('Hello,')
    expect(extractFirstSpeakableChunk('مرحبا، كيف حالك؟')).toBe('مرحبا،')
    expect(extractFirstSpeakableChunk('नमस्ते। आप कैसे हैं?')).toBe('नमस्ते।')
  })

  it('does not split a streamed decimal number', () => {
    expect(extractFirstSpeakableChunk('价格是 3.')).toBeNull()
    expect(extractFirstSpeakableChunk('价格是 3.14，继续')).toBe('价格是 3.14，')
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
