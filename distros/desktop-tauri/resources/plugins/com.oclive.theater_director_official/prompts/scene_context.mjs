/** Scene brief / setting defaults keyed by theater_scene (mirrors theaterSceneCatalog). */

const SCENE_DEFAULTS = {
  breakfast: {
    brief: "早餐 · 上学前：厨房餐桌、温粥、收拾书包、出门前的日常照应与拌嘴。",
    setting: "地点限于家中厨房/餐桌/玄关；时间早晨上学前；禁止脱离居家早饭场景或引入第三人。",
  },
  supermarket: {
    brief: "超市采购：推购物车、抢特价、试吃拌嘴、结账忘带东西的小插曲。",
    setting:
      "地点限于超市卖场/货架/试吃台/收银台；时间白天采购；禁止脱离超市或引入店员以外的第三人对话。",
  },
  way_home: {
    brief: "回家路上：采购或放学后同行，路灯/公交、拌嘴谁拿重物、随口关心。",
    setting: "地点限于街道/路灯下/公交站附近；时间傍晚或采购归途；禁止跳转到室内长戏或引入第三人。",
  },
  bedtime: {
    brief: "洗澡睡觉：洗漱顺序、吹头发/抢浴室、睡前一句软话的收束。",
    setting: "地点限于家中浴室/卧室门口/睡前片刻；时间夜晚就寝前；禁止脱离居家就寝场景。",
  },
};

export function resolveTheaterScene(input) {
  const raw = (input.theater_scene || "").trim();
  if (raw && SCENE_DEFAULTS[raw]) return raw;
  return "breakfast";
}

export function defaultSceneBrief(theaterScene) {
  return SCENE_DEFAULTS[theaterScene]?.brief ?? SCENE_DEFAULTS.breakfast.brief;
}

export function defaultSceneSettingHint(theaterScene) {
  return SCENE_DEFAULTS[theaterScene]?.setting ?? SCENE_DEFAULTS.breakfast.setting;
}

export function pairRelationBlock(input) {
  const hint = (input.pair_relation_hint || "").trim();
  if (!hint) return "";
  const id = (input.pair_relation_id || "").trim() || "custom";
  return `\n双角色关系（${id}）：${hint}\n`;
}

export function sceneContextBlock(input) {
  const theaterScene = resolveTheaterScene(input);
  const brief = (input.scene_brief || "").trim() || defaultSceneBrief(theaterScene);
  const setting = (input.scene_setting_hint || "").trim() || defaultSceneSettingHint(theaterScene);
  return `场景：${brief}\n场景约束：${setting}\n`;
}

export function personaBlock(input, { prefix = "", allowEmpty = true, emptyLabel = "（无）" } = {}) {
  const pa = (input.persona_a || "").trim();
  const pb = (input.persona_b || "").trim();
  if (!pa && !pb) return allowEmpty ? "" : "";
  const lines = [`${prefix}人设摘要${prefix ? "" : "（语气/性格约束）"}：`];
  lines.push(`- A(${input.cast_a_name}): ${pa || emptyLabel}`);
  lines.push(`- B(${input.cast_b_name}): ${pb || emptyLabel}`);
  return `\n${lines.join("\n")}\n`;
}

export function normalizeLeadCast(lead) {
  return (lead || "a").trim().toLowerCase() === "b" ? "b" : "a";
}

export function leadSpeaker(input, leadCast) {
  const nameA = (input.cast_a_name || "").trim();
  const nameB = (input.cast_b_name || "").trim();
  if (leadCast === "b") return [nameB, nameA];
  return [nameA, nameB];
}

export function castRewriteTargetBeats(min, max) {
  return Math.min(Math.max(Math.floor((min + max) / 2), min), 8);
}

export function castRewriteRequiresForks(input) {
  return Array.isArray(input.poke_chips) && input.poke_chips.length > 0;
}
