import { MAX_BEAT_TEXT_LEN } from "../constants.mjs";
import { dramaGuardrailsBlock } from "../drama_guardrails.mjs";
import { pairRelationBlock, sceneContextBlock } from "../scene_context.mjs";

export function buildOutlineRewrite(input) {
  const outline = (input.script_outline || "").trim();
  const maxBeats = input.max_beats || 12;
  const targetBeats =
    input.cast_rewrite_target_beats ||
    Math.min(maxBeats, Math.max(6, Math.floor((input.cast_rewrite_min_beats || 6 + maxBeats) / 2)));
  const strictTail = input.strict ? "\n【严格】只输出 JSON 数组，无 Markdown、无解释。" : "";
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  const personaBlock =
    !pa && !pb
      ? ""
      : `\n人设摘要：\n- A(${input.cast_a_name}): ${pa || "（无）"}\n- B(${input.cast_b_name}): ${pb || "（无）"}\n`;

  return `【剧场大纲 · 用户剧本】围绕以下大纲，为两位角色撰写双人短剧对白。大纲是剧情骨架，须完整覆盖关键节拍，但台词须原创、符合人设。

cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，角色包场景=${input.scene_id}。仅 a/b 两人发言。

用户剧本大纲：
${outline}
${personaBlock}${pairRelationBlock(input)}${sceneContextBlock(input)}${dramaGuardrailsBlock(input, "compact")}
撰写要求：
1. 恰好 ${targetBeats} 条对白（id 依次为 b1,b2,b3…）；cast 只能是 a 或 b；text 非空≤${MAX_BEAT_TEXT_LEN}字。
2. 须落实大纲中的事件与转折；开场 2 拍建立场景感与性格对照；交替发言、口语自然中文。
3. 不得照搬大纲原文当台词；须写成对白与小动作感。

输出契约：JSON 数组，每元素 {"id","cast":"a"|"b","text"}。
示例：[{"id":"b1","cast":"b","text":"……"},{"id":"b2","cast":"a","text":"……"}]${strictTail}
`;
}
