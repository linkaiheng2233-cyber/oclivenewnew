import type { SendMessageResponse } from '@oclive/shared/api'
import { describe, expect, it } from 'vitest'
import { presentationFromSendResponse } from './replyPresentation'

function response(
  overrides: Partial<SendMessageResponse> = {},
): SendMessageResponse {
  return {
    api_version: 2,
    schema: 2,
    presence_mode: 'co_present',
    relation_state: 'friend',
    reply: '回复',
    emotion: {} as SendMessageResponse['emotion'],
    bot_emotion: 'happy',
    portrait_emotion: 'neutral',
    favorability_delta: 0,
    favorability_current: 50,
    events: [],
    scene_id: 'default',
    offer_destination_picker: false,
    offer_together_travel: false,
    timestamp: 0,
    ...overrides,
  }
}

describe('presentationFromSendResponse', () => {
  it('keeps bubble and role portrait emotions independent', () => {
    const presentation = presentationFromSendResponse(response({
      bot_emotion: 'happy',
      portrait_emotion: 'angry',
    }))

    expect(presentation.assistantEmotionLabel).toBe('happy')
    expect(presentation.portraitEmotionLabel).toBe('angry')
  })

  it('falls back to the assistant emotion for legacy empty portrait values', () => {
    const presentation = presentationFromSendResponse(response({
      bot_emotion: 'sad',
      portrait_emotion: '',
    }))

    expect(presentation.portraitEmotionLabel).toBe('sad')
  })
})
