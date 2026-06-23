import { dramaGuardrailsBlock } from "../drama_guardrails.mjs";
import { leadSpeaker, normalizeLeadCast } from "../scene_context.mjs";

export function buildPatch(input) {
  const tweak = input.patch_tweak || {};
  const lead = normalizeLeadCast(tweak.lead_cast);
  const [speakerName, partnerName] = leadSpeaker(input, lead);
  const chipLabel = (tweak.chip_label || "").trim() || "剧情转折";
  const dramaSeed = (tweak.drama_seed || "").trim();
  const prefix = input.patch_prefix_beats || [];
  const canned = input.patch_canned_patch || [];
  const maxLines = input.patch_max_lines || 3;
  const variant = input.patch_variant || 0;

  const contextLines = prefix
    .slice(-4)
    .map((b) => `${(b.name || "").trim()}：${(b.text || "").trim()}`)
    .join("\n");

  let styleExamples = "";
  for (const b of prefix.slice(-2)) {
    styleExamples += `${(b.name || "").trim()}：${(b.text || "").trim()}\n`;
  }
  if (canned[0]) {
    styleExamples += `${(canned[0].name || "").trim()}：${(canned[0].text || "").trim()}\n`;
  }

  const parts = [
    "【剧场即兴 · 戏剧性补丁】",
    `这是一幕双人日常戏。本事件主角是「${speakerName}」（cast ${lead}），须承担事件主体与第一反应；对手戏是「${partnerName}」。`,
    `观众刚刚按下了剧情转折「${chipLabel}」。`,
  ];
  if (dramaSeed) parts.push(`本场戏剧目标：${dramaSeed}`);
  parts.push(dramaGuardrailsBlock(input, "full"));

  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  if (pa || pb) {
    parts.push("", "【人设摘要 · 必须贴合】");
    if (pa) parts.push(`${input.cast_a_name}（cast a）：${pa}`);
    if (pb) parts.push(`${input.cast_b_name}（cast b）：${pb}`);
  }

  parts.push("", "【演出要求】");
  parts.push(`· 写出「${speakerName}」接下来的 1–${maxLines} 句台词，每句一行，格式：角色名：台词`);
  parts.push(
    `· 「${partnerName}」若回句，须带与主角形成性格反差的反应（吐槽/关心/嘴硬/害羞等），禁止礼貌式「好的」「没事吧」敷衍接话；最多回一句，仍算在上述句数内`,
  );
  parts.push("· 至少一句带上动作或神态，单独成行用括号包住，例：(耳朵红了)");
  parts.push("· 紧接上文语气，口语化、贴合人设；不要旁白、解说、JSON 或引号");
  parts.push("· 总字数 100 字以内");
  parts.push(`· 禁止把本事件安到「${partnerName}」身上；主角必须是「${speakerName}」`);
  if (variant === 1) {
    parts.push(
      "· 这是第二版候选：同一事件的不同性格演绎——换情绪走向与措辞，勿换词复述第一版，勿重复同一情节节拍",
    );
  }
  if (input.strict) {
    parts.push("· 【严格模式】只输出对白行与括号动作行，不要任何前缀说明");
  }
  parts.push(
    "",
    "【刚刚发生的对白】",
    contextLines || "（无）",
    "",
    "【可参考的情绪走向（仅作灵感，请改写出新意，禁止照抄）】",
    styleExamples.trim() || "（自由发挥）",
  );
  return parts.join("\n");
}
