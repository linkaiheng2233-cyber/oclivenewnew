import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '@oclive/shared/utils/roleplayReplySplit'

const MODEL_CONTROL_TOKEN = /<\|(?:system|user|assistant|im_start|im_end|endoftext)[^|]*\|>|\[\/?INST\]|<<\/?SYS>>/i
const PROMPT_HEADING_LINE = /^\s*(?:【(?:回复质量锚点|用户身份|世界观设定|复杂情感叙事提示|角色当前状态|真实性约束|上一轮回复约束|生成要求|用户刚发的话)】|核心性格档案（|关于用户的记忆（|用户说\s*[:：]|请以角色身份自然地回复)/m

/**
 * Keep model-control syntax and known prompt sections out of speech. Prompt
 * leakage normally begins on a new line; special model tokens are safe to cut
 * wherever they occur. The same boundary is applied to incremental and final
 * TTS text so late stream tokens cannot reopen the prompt tail.
 */
export function stripPromptLeakForVoice(raw: string): string {
  let end = raw.length
  const control = MODEL_CONTROL_TOKEN.exec(raw)
  if (control?.index !== undefined)
    end = Math.min(end, control.index)
  const promptHeading = PROMPT_HEADING_LINE.exec(raw)
  if (promptHeading?.index !== undefined)
    end = Math.min(end, promptHeading.index)
  return raw.slice(0, end).trimEnd()
}

/** Dialogue-only text for TTS (strips roleplay aside/narration and prompt tails). */
export function voiceDialogueFromRaw(raw: string): string {
  const text = stripPromptLeakForVoice(raw).trim()
  if (!text)
    return ''
  const split = splitRoleplayReply(text)
  return assistantDialogueFromSplit(text, split).trim()
}
