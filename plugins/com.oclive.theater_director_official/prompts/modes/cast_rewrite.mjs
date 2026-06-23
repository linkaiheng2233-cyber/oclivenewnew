import { MAX_BEAT_TEXT_LEN } from "../constants.mjs";
import { dramaGuardrailsBlock } from "../drama_guardrails.mjs";
import {
  castRewriteRequiresForks,
  castRewriteTargetBeats,
  pairRelationBlock,
  resolveTheaterScene,
  sceneContextBlock,
} from "../scene_context.mjs";

export function buildCastRewrite(input) {
  const strictTail = input.strict
    ? "\n【严格】只输出 JSON 数组，无 Markdown、无解释。每条仅 id、cast、text；cast 只能是 a 或 b。"
    : "";
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  const personaBlock = `- A(${input.cast_a_name}): ${pa || "（无额外人设，按角色名推断高中生语气）"}\n- B(${input.cast_b_name}): ${pb || "（无额外人设，按角色名推断高中生语气）"}`;
  const theaterScene = resolveTheaterScene(input);
  const min = input.cast_rewrite_min_beats || 6;
  const max = input.cast_rewrite_max_beats || input.max_beats || 12;
  const targetBeats = input.cast_rewrite_target_beats || castRewriteTargetBeats(min, max);
  const pokeLine = castRewriteRequiresForks(input)
    ? "\n5. 主剧本须为戳点 chip 可能触发的事件留出合理插入空间（中段附近可接小插曲），不要预写 forks 正文。"
    : "";

  return `卡司重写：为以下两位角色**从零**撰写「${theaterScene}」双人短剧。不要沿用任何现成台词或剧情模板，须完全贴合人设关系与说话方式。

cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，角色包场景=${input.scene_id}。仅 a/b 两人发言，禁止第三人。

${sceneContextBlock(input)}${dramaGuardrailsBlock(input, "compact")}人设摘要：
${personaBlock}${pairRelationBlock(input)}
撰写要求：
1. 恰好 ${targetBeats} 条对白（id 依次为 b1,b2,b3…）；cast 只能是 a 或 b；text 非空≤${MAX_BEAT_TEXT_LEN}字；写**全新**对白与小事件，须落在上述场景约束内。
2. 开场 2 拍须建立场景物件感与两人性格对照；交替发言、有戏感、口语自然中文。
3. 中段留 poke 插入空间，勿把戳点事件写死进主剧本。${pokeLine}
4. 戳点分支由系统另行挂载，不要输出 forks 字段。

输出格式（仅 JSON 数组，不要其它文字）：
- 只输出一个 JSON 数组；不要 Markdown、不要代码块围栏、不要前后说明。
- 每条仅含 id、cast、text 三个字段；不要 name、stage_hint、emotion 等字段。
- 示例：
[{"id":"b1","cast":"b","text":"……"},{"id":"b2","cast":"a","text":"……"}]
${strictTail}`;
}
