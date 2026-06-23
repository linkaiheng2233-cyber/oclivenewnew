import { dramaGuardrailsBlock } from "../drama_guardrails.mjs";
import { defaultSceneBrief, resolveTheaterScene } from "../scene_context.mjs";

export function buildCastRewriteMinimal(input) {
  const target = input.cast_rewrite_target_beats || input.max_beats || 8;
  const theaterScene = resolveTheaterScene(input);
  const brief = (input.scene_brief || "").trim() || defaultSceneBrief(theaterScene);
  const pa = (input.persona_a || "").trim() || "按角色名推断语气";
  const pb = (input.persona_b || "").trim() || "按角色名推断语气";
  return `只输出 JSON 数组，恰好 ${target} 条对白。从 [ 开始到 ] 结束，不要 Markdown、不要解释。
cast 只能是 a 或 b；每条仅 id、cast、text 三个字段。
A(${input.cast_a_name})=${pa}
B(${input.cast_b_name})=${pb}
场景：${brief}${dramaGuardrailsBlock(input, "compact")}
示例：[{"id":"b1","cast":"b","text":"……"},{"id":"b2","cast":"a","text":"……"},{"id":"b3","cast":"a","text":"……"}]`;
}
