import { describe, expect, it } from 'vitest'
import { stripPromptLeakForVoice, voiceDialogueFromRaw } from './voiceDialogueFromRaw'

describe('stripPromptLeakForVoice', () => {
  it('cuts a leaked host prompt section after dialogue', () => {
    const raw = '我在这里陪你。\n\n【回复质量锚点】（每轮须遵守）\n- 只写角色台词'
    expect(stripPromptLeakForVoice(raw)).toBe('我在这里陪你。')
  })

  it('cuts model control tokens even without a newline', () => {
    expect(stripPromptLeakForVoice('晚点见。<|system|>你是角色')).toBe('晚点见。')
  })

  it('preserves ordinary dialogue mentioning a user', () => {
    expect(stripPromptLeakForVoice('这个用户说法挺有意思，但我更想听你的想法。'))
      .toBe('这个用户说法挺有意思，但我更想听你的想法。')
  })
})

describe('voiceDialogueFromRaw', () => {
  it('applies the prompt boundary before roleplay splitting', () => {
    expect(voiceDialogueFromRaw('嗯，我知道了。\n用户说: 不要读这一段'))
      .toBe('嗯，我知道了。')
  })
})
