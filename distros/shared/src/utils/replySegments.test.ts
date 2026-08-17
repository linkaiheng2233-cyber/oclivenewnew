import { describe, expect, it } from 'vitest'
import {
  draftsFromPresentation,
  segmentMessageIds,
  splitReplyBySeparatorLine,
} from './replySegments'

describe('splitReplyBySeparatorLine', () => {
  it('splits only standalone separator lines', () => {
    expect(splitReplyBySeparatorLine('第一发\n\n+++\n\n第二发', '+++', 2)).toEqual([
      '第一发',
      '第二发',
    ])
    expect(splitReplyBySeparatorLine('C+++ 代码\n\na +++ b\n\n+++\n\n第二发', '+++', 2)).toEqual([
      'C+++ 代码\n\na +++ b',
      '第二发',
    ])
  })

  it('degrades to one segment when separator is missing', () => {
    expect(splitReplyBySeparatorLine('只有一段，第二发没有来。', '+++', 2)).toEqual([
      '只有一段，第二发没有来。',
    ])
  })

  it('caps and merges overflow segments', () => {
    expect(splitReplyBySeparatorLine('一\n+++\n二\n+++\n三\n+++\n四', '+++', 2)).toEqual([
      '一',
      '二\n\n三\n\n四',
    ])
  })

  it('drops empty segments and normalizes CRLF', () => {
    expect(splitReplyBySeparatorLine('第一发\r\n\r\n+++\r\n\r\n第二发', '+++', 3)).toEqual([
      '第一发',
      '第二发',
    ])
  })
})

describe('draftsFromPresentation', () => {
  it('keeps ordinary replies as one draft', () => {
    expect(draftsFromPresentation('你好。', null)).toEqual([{ text: '你好。', delayMs: 0 }])
  })

  it('maps segments and delays from the backend DTO', () => {
    expect(
      draftsFromPresentation('第一发\n第二发', {
        segments: ['第一发', '第二发'],
        delays_ms: [0, 300],
      }),
    ).toEqual([
      { text: '第一发', delayMs: 0 },
      { text: '第二发', delayMs: 300 },
    ])
  })
})

describe('segmentMessageIds', () => {
  it('keeps single segments on the base id', () => {
    expect(segmentMessageIds('a-1', 1)).toEqual(['a-1'])
  })

  it('suffixes extra segments', () => {
    expect(segmentMessageIds('a-1', 3)).toEqual(['a-1#s0', 'a-1#s1', 'a-1#s2'])
  })
})
