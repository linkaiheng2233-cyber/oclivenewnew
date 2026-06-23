import { MAX_BEAT_TEXT_LEN } from "../constants.mjs";
import { dramaGuardrailsBlock } from "../drama_guardrails.mjs";
import { sceneContextBlock } from "../scene_context.mjs";

function castAdaptPassInstructions(pass) {
  const p = (pass || "").trim();
  let focus;
  if (p === "voice") {
    focus =
      "【本轮·语气人设】第一轮：在保持事件顺序与 beat id 不变的前提下，把每位角色的台词改成其人设口吻；同步调整 emotion/stage_hint 以贴合性格（毒舌/温柔/别扭等）。禁止只改姓名。";
  } else if (p === "depth") {
    focus =
      "【本轮·角色化大纲】第二轮：在当前场景时间框架内，进一步改写台词内容与 stage_hint，使互动、拌嘴方式、关心/抵触的表达方式更符合两位角色的关系与性格；可调整具体物件与情绪转折，但 beat id/cast 不可变，仍须落在同一 scene_brief 场景。";
  } else if (p === "polish") {
    focus =
      "【本轮·戳点收束】第三轮：重点改写 forks 戳点罐头台词，每条须是可分享的一击（有反差/动作/情绪），勿平述；beats 做最终通顺与人设一致性润色，确保全剧台词风格统一、角色区分度明显。";
  } else {
    focus = "【综合适配】语气、角色化互动与戳点一并改写；beat id/cast 不可变。";
  }
  return `\n${focus}\n`;
}

export function buildCastAdapt(input) {
  const beatsJson = JSON.stringify(input.base_beats || []);
  const forksJson = JSON.stringify(input.fork_templates || []);
  const strictTail = input.strict
    ? "\n【严格】只输出 JSON 对象，无 Markdown、无解释。每个 beat/fork patch 的 id 与 cast 必须与骨架完全一致。"
    : "";
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  const personaBlock =
    !pa && !pb
      ? ""
      : `\n人设摘要（语气/性格约束）：\n- A(${input.cast_a_name}): ${pa || "（无）"}\n- B(${input.cast_b_name}): ${pb || "（无）"}\n`;

  return `卡司适配：双人剧场。cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，场景=${input.scene_id}。仅 a/b 发言。
${castAdaptPassInstructions(input.adapt_pass)}${dramaGuardrailsBlock(input, "compact")}${sceneContextBlock(input)}${personaBlock}
开场 beats 骨架（id/cast 只读，可改 name/text/stage_hint/emotion）：
${beatsJson}

戳点 fork 罐头（每项 chip_id 只读；patch_lines 的 id/cast 只读，可改 name/text/stage_hint/emotion；勿输出 insert_after_beat_id）：
${forksJson}

输出契约：JSON 对象 {"beats":[...],"forks":[{"chip_id","patch_lines":[...]}]}（forks 可省略，有则改写戳点罐头）。
规则：beats 每项 id/cast 与骨架一致；forks 每项 chip_id 与骨架一致、patch_lines 的 id/cast 一致；总 beats≤${input.max_beats}；text 非空≤${MAX_BEAT_TEXT_LEN}字；台词须明显贴合各角色人设；禁止仅替换姓名；不得新增第三人或自造 id。

示例：{"beats":[{"id":"b1","cast":"b","name":"${input.cast_b_name}","text":"……","emotion":"happy"}],"forks":[{"chip_id":"tea","patch_lines":[{"id":"tea-1","cast":"b","name":"${input.cast_b_name}","text":"……"}]}]}${strictTail}
`;
}
