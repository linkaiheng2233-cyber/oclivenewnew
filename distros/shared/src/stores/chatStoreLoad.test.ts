import { describe, expect, it } from 'vitest'
import {
  buildSceneLoadCandidates,
  splitAssistantMessages,
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
