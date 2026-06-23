/**
 * Theater prompt builders — mirrors kernel builtin templates (official plugin).
 */
const MAX_BEAT_TEXT_LEN = 500;
const MAX_STAGE_HINT_LEN = 120;

function defaultSceneBrief() {
  return "早餐 · 上学前：厨房餐桌、温粥、收拾书包、出门前的日常照应与拌嘴。";
}

function defaultSceneSettingHint() {
  return "地点限于家中厨房/餐桌/玄关；时间早晨上学前；禁止脱离居家早饭场景或引入第三人。";
}

function pairRelationBlock(input) {
  const hint = (input.pair_relation_hint || "").trim();
  if (!hint) return "";
  const id = (input.pair_relation_id || "").trim() || "custom";
  return `\n双角色关系（${id}）：${hint}\n`;
}

function sceneContextBlock(input) {
  const brief = (input.scene_brief || "").trim() || defaultSceneBrief();
  const setting = (input.scene_setting_hint || "").trim() || defaultSceneSettingHint();
  return `场景：${brief}\n场景约束：${setting}\n`;
}

function castAdaptPassInstructions(pass) {
  const p = (pass || "").trim();
  let focus;
  if (p === "voice") {
    focus =
      "【本轮·语气人设】第一轮：在保持事件顺序与 beat id 不变的前提下，把每位角色的台词改成其人设口吻；同步调整 emotion/stage_hint 以贴合性格（毒舌/温柔/别扭等）。禁止只改姓名。";
  } else if (p === "depth") {
    focus =
      "【本轮·角色化大纲】第二轮：在早饭→上学前的时间框架内，进一步改写台词内容与 stage_hint，使互动、拌嘴方式、关心/抵触的表达方式更符合两位角色的关系与性格；可调整具体物件与情绪转折，但 beat id/cast 不可变，仍须落在同一早餐场景。";
  } else if (p === "polish") {
    focus =
      "【本轮·戳点收束】第三轮：重点改写 forks 戳点罐头台词；beats 做最终通顺与人设一致性润色，确保全剧台词风格统一、角色区分度明显。";
  } else {
    focus = "【综合适配】语气、角色化互动与戳点一并改写；beat id/cast 不可变。";
  }
  return `\n${focus}\n`;
}

function castRewriteTargetBeats(min, max) {
  return Math.min(Math.max(Math.floor((min + max) / 2), min), 8);
}

function castRewriteRequiresForks(input) {
  return Array.isArray(input.poke_chips) && input.poke_chips.length > 0;
}

function normalizeLeadCast(lead) {
  return (lead || "a").trim().toLowerCase() === "b" ? "b" : "a";
}

function leadSpeaker(input, leadCast) {
  const nameA = (input.cast_a_name || "").trim();
  const nameB = (input.cast_b_name || "").trim();
  if (leadCast === "b") return [nameB, nameA];
  return [nameA, nameB];
}

function buildRipple(input) {
  const prefix = input.ripple_prefix_beats || [];
  const skeleton = input.ripple_skeleton || [];
  const fullRewrite = input.ripple_full_rewrite || false;
  const maxBeats = input.max_beats || 12;
  const prefixJson = JSON.stringify(prefix);
  const rippleJson = JSON.stringify(skeleton);
  const tweaksJson = JSON.stringify(input.applied_tweaks || []);

  const scopeBlock = fullRewrite
    ? `无微调：重写整场（≤${maxBeats} 拍）。开场骨架：\n${rippleJson}`
    : `前缀（只读，禁止改写或重复输出）：\n${prefixJson}\n\n涟漪区骨架（须重写，体现 drama_seed）：\n${rippleJson}`;

  const tweakBlock =
    (input.applied_tweaks || []).length === 0
      ? "（无微调）"
      : `微调意图：${tweaksJson}\n微调纪律：drama_seed 是剧情变数/事件，须融入涟漪区大纲；不得机械复制罐头 fork 的 cast 顺序或台词；由谁开口、谁主导反应由 A/B 人设摘要与前缀上下文决定，两人都要有戏；罐头 patchLines 仅表事件方向与接锚点，不是强制台词模板；须自然接回后续节拍走向。`;

  const strictTail = input.strict ? "\n【严格】只输出 JSON 数组，无 Markdown、无解释。" : "";
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  const personaBlock =
    !pa && !pb
      ? ""
      : `\n人设摘要（语气/性格约束，不得改变大纲事件）：\n- A(${input.cast_a_name}): ${pa || "（无）"}\n- B(${input.cast_b_name}): ${pb || "（无）"}\n`;

  const settingTail = (input.scene_setting_hint || "").trim() || defaultSceneSettingHint();
  const outputScope = fullRewrite ? "整场" : "涟漪区（不含前缀）";

  return `场景导演：双人剧场。cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，角色包场景=${input.scene_id}。仅 a/b 发言。
${personaBlock}${pairRelationBlock(input)}${sceneContextBlock(input)}
${scopeBlock}

${tweakBlock}

输出契约：JSON 数组，每元素 {"id","cast":"a"|"b","name","text","stage_hint?","emotion?"}。
规则：只输出${outputScope}；总拍数≤${maxBeats}；text 非空≤${MAX_BEAT_TEXT_LEN}字；name 与 cast 一致；台词须符合各人设；微调时 cast 分配须随人设与上下文决定，勿照搬罐头 fork 的说话顺序；${settingTail}；不得新增第三人。

示例：[{"id":"r1","cast":"b","name":"枫侵月","text":"……","stage_hint":"推碗","emotion":"happy"},{"id":"r2","cast":"a","name":"木木","text":"……","emotion":"shy"}]${strictTail}
`;
}

function buildCastAdapt(input) {
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

  return `卡司适配：双人早饭上学前剧场。cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，场景=${input.scene_id}。仅 a/b 发言。
${castAdaptPassInstructions(input.adapt_pass)}
${personaBlock}
开场 beats 骨架（id/cast 只读，可改 name/text/stage_hint/emotion）：
${beatsJson}

戳点 fork 罐头（每项 chip_id 只读；patch_lines 的 id/cast 只读，可改 name/text/stage_hint/emotion；勿输出 insert_after_beat_id）：
${forksJson}

输出契约：JSON 对象 {"beats":[...],"forks":[{"chip_id","patch_lines":[...]}]}（forks 可省略，有则改写戳点罐头）。
规则：beats 每项 id/cast 与骨架一致；forks 每项 chip_id 与骨架一致、patch_lines 的 id/cast 一致；总 beats≤${input.max_beats}；text 非空≤${MAX_BEAT_TEXT_LEN}字；台词须明显贴合各角色人设；禁止仅替换姓名；不得新增第三人或自造 id。

示例：{"beats":[{"id":"b1","cast":"b","name":"${input.cast_b_name}","text":"……","emotion":"happy"}],"forks":[{"chip_id":"tea","patch_lines":[{"id":"tea-1","cast":"b","name":"${input.cast_b_name}","text":"……"}]}]}${strictTail}
`;
}

function buildCastRewrite(input) {
  const strictTail = input.strict
    ? "\n【严格】只输出 JSON 数组，无 Markdown、无解释。每条仅 id、cast、text；cast 只能是 a 或 b。"
    : "";
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  const personaBlock = `- A(${input.cast_a_name}): ${pa || "（无额外人设，按角色名推断高中生语气）"}\n- B(${input.cast_b_name}): ${pb || "（无额外人设，按角色名推断高中生语气）"}`;
  const theaterScene = (input.theater_scene || "").trim() || "breakfast";
  const min = input.cast_rewrite_min_beats || 6;
  const max = input.cast_rewrite_max_beats || input.max_beats || 12;
  const targetBeats = input.cast_rewrite_target_beats || castRewriteTargetBeats(min, max);
  const pokeLine = castRewriteRequiresForks(input)
    ? "\n4. 主剧本须为戳点 chip 可能触发的事件留出合理插入空间（中段附近可接小插曲），不要预写 forks 正文。"
    : "";

  return `卡司重写：为以下两位角色**从零**撰写「${theaterScene}」双人短剧。不要沿用任何现成台词或剧情模板，须完全贴合人设关系与说话方式。

cast a=${input.cast_a_name}，cast b=${input.cast_b_name}，角色包场景=${input.scene_id}。仅 a/b 两人发言，禁止第三人。

${sceneContextBlock(input)}人设摘要：
${personaBlock}${pairRelationBlock(input)}
撰写要求：
1. 恰好 ${targetBeats} 条对白（id 依次为 b1,b2,b3…）；cast 只能是 a 或 b；text 非空≤${MAX_BEAT_TEXT_LEN}字；写**全新**对白与小事件，须落在上述场景约束内。
2. 交替发言、有戏感、口语自然中文。
3. 戳点分支由系统另行挂载，不要输出 forks 字段。${pokeLine}

输出格式（仅 JSON 数组，不要其它文字）：
- 只输出一个 JSON 数组；不要 Markdown、不要代码块围栏、不要前后说明。
- 每条仅含 id、cast、text 三个字段；不要 name、stage_hint、emotion 等字段。
- 示例：
[{"id":"b1","cast":"b","text":"……"},{"id":"b2","cast":"a","text":"……"}]
${strictTail}`;
}

function buildCastRewriteMinimal(input) {
  const target = input.cast_rewrite_target_beats || input.max_beats || 8;
  const brief = (input.scene_brief || "").trim() || defaultSceneBrief();
  const pa = (input.persona_a || "").trim() || "按角色名推断语气";
  const pb = (input.persona_b || "").trim() || "按角色名推断语气";
  return `只输出 JSON 数组，恰好 ${target} 条对白。从 [ 开始到 ] 结束，不要 Markdown、不要解释。
cast 只能是 a 或 b；每条仅 id、cast、text 三个字段。
A(${input.cast_a_name})=${pa}
B(${input.cast_b_name})=${pb}
场景：${brief}
示例：[{"id":"b1","cast":"b","text":"……"},{"id":"b2","cast":"a","text":"……"},{"id":"b3","cast":"a","text":"……"}]`;
}

function buildPatch(input) {
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
  const revPrefix = prefix.slice(-2);
  for (const b of revPrefix) {
    styleExamples += `${(b.name || "").trim()}：${(b.text || "").trim()}\n`;
  }
  if (canned[0]) {
    styleExamples += `${(canned[0].name || "").trim()}：${(canned[0].text || "").trim()}\n`;
  }

  const parts = [
    "【剧场即兴 · 导演指令】",
    `这是一幕双人日常戏。本事件主角是「${speakerName}」（cast ${lead}），对手戏是「${partnerName}」。`,
    `观众刚刚按下了剧情转折「${chipLabel}」。`,
  ];
  if (dramaSeed) parts.push(`本场戏剧目标：${dramaSeed}`);
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  if (pa || pb) {
    parts.push("", "【人设摘要 · 必须贴合】");
    if (pa) parts.push(`${input.cast_a_name}（cast a）：${pa}`);
    if (pb) parts.push(`${input.cast_b_name}（cast b）：${pb}`);
  }
  parts.push("", "【演出要求】");
  parts.push(`· 写出「${speakerName}」接下来的 1–${maxLines} 句台词，每句一行，格式：角色名：台词`);
  parts.push(`· 可选：「${partnerName}」最多回一句，仍算在上述句数内`);
  parts.push("· 至少一句带上动作或神态，单独成行用括号包住，例：(耳朵红了)");
  parts.push("· 紧接上文语气，口语化、贴合人设；不要旁白、解说、JSON 或引号");
  parts.push("· 总字数 100 字以内");
  parts.push(`· 禁止把本事件安到「${partnerName}」身上；主角必须是「${speakerName}」`);
  if (variant === 1) {
    parts.push("· 这是第二版候选：给出另一种合理走向，仍贴合人设，勿重复第一版的措辞与情节");
  }
  if (input.strict) {
    parts.push("· 【严格模式】只输出对白行与括号动作行，不要任何前缀说明");
  }
  parts.push("", "【刚刚发生的对白】", contextLines || "（无）", "", "【可参考的情绪走向（仅作灵感，请改写出新意，禁止照抄）】", styleExamples.trim() || "（自由发挥）");
  return parts.join("\n");
}

export function buildTheaterPrompt(input) {
  const mode = (input.mode || "ripple").trim();
  switch (mode) {
    case "patch":
      return buildPatch(input);
    case "cast_adapt":
      return buildCastAdapt(input);
    case "cast_rewrite":
      return buildCastRewrite(input);
    case "cast_rewrite_minimal":
      return buildCastRewriteMinimal(input);
    case "ripple":
    default:
      return buildRipple(input);
  }
}
