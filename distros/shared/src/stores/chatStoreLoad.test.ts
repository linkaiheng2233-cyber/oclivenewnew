import { describe, expect, it } from 'vitest'
import {
  buildSceneLoadCandidates,
  splitAssistantMessages,
  storedMessagesToChatMessages,
} from './chatStoreLoad'

describe('buildSceneLoadCandidates', () => {
  it('prioritizes backend session scenes, then narrative primary, then pack scenes', () => {
    expect(
      buildSceneLoadCandidates('company', ['home', 'school', 'company'], ['school']),
    ).toEqual(['school', 'company', 'home', 'default'])
  })

  it('deduplicates and always includes home/default fallbacks', () => {
    expect(buildSceneLoadCandidates('home', ['home'], [])).toEqual(['home', 'default'])
  })
})

describe('splitAssistantMessages', () => {
  it('preserves narration bubbles for assistant history older than the former tail window', () => {
    const messages = Array.from({ length: 100 }, (_, index) => ({
      id: String(index),
      role: 'assistant' as const,
      content: `dialogue ${index}\n\n【旁白】narration ${index}`,
      timestamp: index,
    }))

    const split = splitAssistantMessages(messages)

    expect(split[0]).toMatchObject({
      content: 'dialogue 0',
      aside: '【旁白】narration 0',
    })
    expect(split[99]).toMatchObject({
      content: 'dialogue 99',
      aside: '【旁白】narration 99',
    })
  })
})

describe('storedMessagesToChatMessages', () => {
  it('restores every persisted reply-mode segment as a sibling bubble', () => {
    const messages = storedMessagesToChatMessages([{
      id: 'assistant-1',
      session_id: 'role',
      turn_index: 1,
      sender: 'assistant',
      content: 'first burst\nsecond burst\nthird burst',
      metadata: JSON.stringify({
        bot_emotion: 'neutral',
        reply_segments: ['first burst', 'second burst', 'third burst'],
        reply_segment_delays_ms: [0, 100, 100],
      }),
      created_at: '2026-08-17T12:00:00.000Z',
    }])

    expect(messages.map(message => ({
      id: message.id,
      content: message.content,
    }))).toEqual([
      { id: 'assistant-1#s0', content: 'first burst' },
      { id: 'assistant-1#s1', content: 'second burst' },
      { id: 'assistant-1#s2', content: 'third burst' },
    ])
    expect(messages.every(message => !message.content.includes('+++'))).toBe(true)
  })
})
