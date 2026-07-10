import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '@oclive/shared/utils/roleplayReplySplit'

/** Dialogue-only text for TTS (strips roleplay aside/narration lines). */
export function voiceDialogueFromRaw(raw: string): string {
  const text = raw.trim()
  if (!text)
    return ''
  const split = splitRoleplayReply(text)
  return assistantDialogueFromSplit(text, split).trim()
}
