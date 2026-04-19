/**
 * 将模型整段回复拆成「对白」与「旁白/内心/动作」，供主聊天区与左侧叙事条分流展示。
 *
 * 约定（可在角色包 system / few-shot 中引导模型）：
 * - 独立行以 `【内心】` `【动作】` `【场景】` `【旁白】` `【独白】` 开头 → 归入旁白；
 * - 括号短句 `（…）` 且内含 心里/内心/默默/暗想/小声/嘀咕 等 → 从对白移出，归入旁白；
 * - 其余括号（如「笑」「点头」）保留在对白中。
 */
const INNER_IN_PAREN = /心里|内心|默默|暗想|小声|嘀咕/;
const TAG_LINE = /^\s*【(?:内心|动作|场景|旁白|独白)】/;
const PAREN_CHUNK = /（[^）]{1,500}）/g;

export interface RoleplaySplit {
  dialogue: string;
  aside: string;
}

/** 与 `chatStore` 中助手气泡正文规则一致（仅旁白时占位为「…」） */
export function assistantDialogueFromSplit(raw: string, split: RoleplaySplit): string {
  const d = split.dialogue.trim();
  if (d.length > 0) return d;
  if (split.aside.trim().length > 0) return "…";
  return raw.trim();
}

export function splitRoleplayReply(raw: string): RoleplaySplit {
  const asideChunks: string[] = [];
  const lines = raw.replace(/\r\n/g, "\n").split("\n");
  const keptLines: string[] = [];
  for (const line of lines) {
    const tr = line.trim();
    if (tr.length > 0 && TAG_LINE.test(tr)) {
      asideChunks.push(tr);
      continue;
    }
    keptLines.push(line);
  }
  let t = keptLines.join("\n");
  let changed = true;
  while (changed) {
    changed = false;
    t = t.replace(PAREN_CHUNK, (full) => {
      const inner = full.slice(1, -1);
      if (INNER_IN_PAREN.test(inner) || /^心里|内心|默默|暗想/.test(inner)) {
        asideChunks.push(full.trim());
        changed = true;
        return "";
      }
      return full;
    });
  }
  const dialogue = t
    .replace(/\n{3,}/g, "\n\n")
    .replace(/[ \t\u3000]+$/gm, "")
    .trim();
  const aside = asideChunks.join("\n\n").trim();
  return { dialogue, aside };
}
