/**
 * Split a full model reply into dialogue vs aside/narration/action for main chat vs left narrative strip.
 *
 * Conventions (guide model via role pack system / few-shot):
 * - Standalone lines starting with `【内心】` `【动作】` `【场景】` `【旁白】` `【独白】` → aside;
 * - Parenthetical `（…）` containing 心里/内心/默默/暗想/小声/嘀咕 etc. → move from dialogue to aside;
 * - Other parentheses (e.g. 「笑」「点头」) stay in dialogue.
 */
import { rt } from '@oclive/shared/i18n/runtimeT'

const INNER_IN_PAREN = /心里|内心|默默|暗想|小声|嘀咕/
const TAG_LINE = /^\s*【(?:内心|动作|场景|旁白|独白)】/
const PAREN_CHUNK = /（[^）]{1,500}）/g

export interface RoleplaySplit {
  dialogue: string
  aside: string
}

/** Matches `chatStore` assistant bubble body rule (aside-only placeholder is 「…」). */
export function assistantDialogueFromSplit(raw: string, split: RoleplaySplit): string {
  const d = split.dialogue.trim()
  if (d.length > 0)
    return d
  if (split.aside.trim().length > 0)
    return rt('chat.assistReplyAsideOnly')
  return raw.trim()
}

export function splitRoleplayReply(raw: string): RoleplaySplit {
  const asideChunks: string[] = []
  const lines = raw.replace(/\r\n/g, '\n').split('\n')
  const keptLines: string[] = []
  for (const line of lines) {
    const tr = line.trim()
    if (tr.length > 0 && TAG_LINE.test(tr)) {
      asideChunks.push(tr)
      continue
    }
    keptLines.push(line)
  }
  let t = keptLines.join('\n')
  let changed = true
  while (changed) {
    changed = false
    t = t.replace(PAREN_CHUNK, (full) => {
      const inner = full.slice(1, -1)
      if (INNER_IN_PAREN.test(inner) || /^心里|内心|默默|暗想/.test(inner)) {
        asideChunks.push(full.trim())
        changed = true
        return ''
      }
      return full
    })
  }
  const dialogue = t
    .replace(/\n{3,}/g, '\n\n')
    .replace(/[ \t\u3000]+$/gm, '')
    .trim()
  const aside = asideChunks.join('\n\n').trim()
  return { dialogue, aside }
}

/** Apply roleplay split to an assistant message (idempotent when `aside` already set). */
export function applyAssistantSplit<
  T extends { role: string, content: string, aside?: string },
>(msg: T): T {
  if (msg.role !== 'assistant' || msg.aside?.trim())
    return msg
  const split = splitRoleplayReply(msg.content)
  const aside = split.aside.trim()
  return {
    ...msg,
    content: assistantDialogueFromSplit(msg.content, split),
    ...(aside ? { aside } : {}),
  }
}
