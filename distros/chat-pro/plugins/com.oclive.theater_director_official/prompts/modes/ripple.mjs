import { MAX_BEAT_TEXT_LEN } from "../constants.mjs";
import { dramaGuardrailsBlock } from "../drama_guardrails.mjs";
import {
  defaultSceneSettingHint,
  pairRelationBlock,
  resolveTheaterScene,
  sceneContextBlock,
} from "../scene_context.mjs";

export function buildRipple(input) {
  const prefix = input.ripple_prefix_beats || [];
  const skeleton = input.ripple_skeleton || [];
  const fullRewrite = input.ripple_full_rewrite || false;
  const maxBeats = input.max_beats || 12;
  const prefixJson = JSON.stringify(prefix);
  const rippleJson = JSON.stringify(skeleton);
  const tweaksJson = JSON.stringify(input.applied_tweaks || []);
  const theaterScene = resolveTheaterScene(input);

  const scopeBlock = fullRewrite
    ? `无微调：重写整场（≤${maxBeats} 拍）。开场骨架：\n${rippleJson}`
    : `前缀（只读，禁止改写或重复输出）：\n${prefixJson}\n\n涟漪区骨架（须重写，体现 drama_seed）：\n${rippleJson}`;

  const tweakBlock =
    (input.applied_tweaks || []).length === 0
      ? "（无微调）"
      : `微调意图：${tweaksJson}\n微调纪律：drama_seed 是剧情变数/事件，须融入涟漪区大纲；不得机械复制罐头 fork 的 cast 顺序或台词；由谁开口、谁主导反应由 A/B 人设摘要与前缀上下文决定，两人都要有戏；罐头 patchLines 仅表事件方向与接锚点，不是强制台词模板；须自然接回后续节拍走向。涟漪区须比前缀更有张力，禁止平淡续写。`;

  const strictTail = input.strict ? "\n【严格】只输出 JSON 数组，无 Markdown、无解释。" : "";
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  const personaBlock =
    !pa && !pb
      ? ""
      : `\n人设摘要（语气/性格约束，不得改变大纲事件）：\n- A(${input.cast_a_name}): ${pa || "（无）"}\n- B(${input.cast_b_name}): ${pb || "（无）"}\n`;

  const settingTail = (input.scene_setting_hint || "").trim() || defaultSceneSettingHint(theaterScene);
  const outputScope = fullRewrite ? "整场" : "涟漪区（不含前缀）";

  return `场景导演：双人剧场。cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，角色包场景=${input.scene_id}。仅 a/b 发言。
${personaBlock}${pairRelationBlock(input)}${sceneContextBlock(input)}${dramaGuardrailsBlock(input, "full")}
${scopeBlock}

${tweakBlock}

输出契约：JSON 数组，每元素 {"id","cast":"a"|"b","name","text","stage_hint?","emotion?"}。
规则：只输出${outputScope}；总拍数≤${maxBeats}；text 非空≤${MAX_BEAT_TEXT_LEN}字；name 与 cast 一致；台词须符合各人设；微调时 cast 分配须随人设与上下文决定，勿照搬罐头 fork 的说话顺序；${settingTail}；不得新增第三人。

示例：[{"id":"r1","cast":"b","name":"枫侵月","text":"……","stage_hint":"推碗","emotion":"happy"},{"id":"r2","cast":"a","name":"木木","text":"……","emotion":"shy"}]${strictTail}
`;
}
