import fs from "node:fs";
import path from "node:path";

const PRESET_HEADER = `【角色润色器 · 非扮演】
任务：保持人设与事实，修正复读/OOC/格式；只输出角色台词。

【人设摘要】{excerpt}
【不可违背】{anchor_excerpt}
【输出规则】只输出修正文本；已合格则原样返回；禁止复述用户起笔。`;

/**
 * @param {string} roleDir
 * @param {{ maxExcerpt?: number }} [opts]
 * @returns {string}
 */
export function buildPresetFromRolePack(roleDir, opts = {}) {
  const maxExcerpt = opts.maxExcerpt ?? readMaxExcerptEnv();

  const customPath = path.join(roleDir, "polish_prompt.md");
  if (fs.existsSync(customPath)) {
    return fs.readFileSync(customPath, "utf8").trim();
  }

  const personalityPath = path.join(roleDir, "core_personality.txt");
  const blueprintPath = path.join(roleDir, "pipeline.ocblueprint");

  const excerpt = fs.existsSync(personalityPath)
    ? truncateExcerpt(fs.readFileSync(personalityPath, "utf8"), maxExcerpt)
    : "（无人设摘要）";

  const anchor = readReplyQualityAnchor(blueprintPath) || "（无质量锚点）";

  return PRESET_HEADER.replace("{excerpt}", excerpt).replace(
    "{anchor_excerpt}",
    truncateExcerpt(anchor, Math.min(maxExcerpt, 600)),
  );
}

/**
 * @param {string} blueprintPath
 * @returns {string | null}
 */
export function readReplyQualityAnchor(blueprintPath) {
  if (!fs.existsSync(blueprintPath)) {
    return null;
  }
  try {
    const doc = JSON.parse(fs.readFileSync(blueprintPath, "utf8"));
    const anchor = doc?.meta?.reply_quality_anchor;
    return typeof anchor === "string" && anchor.trim() ? anchor.trim() : null;
  } catch {
    return null;
  }
}

/**
 * @param {string} text
 * @param {number} maxChars
 */
export function truncateExcerpt(text, maxChars) {
  const t = text.trim();
  if (t.length <= maxChars) {
    return t;
  }
  return `${t.slice(0, maxChars).trim()}…`;
}

function readMaxExcerptEnv() {
  const raw = process.env.OCLIVE_POLISH_MAX_EXCERPT?.trim();
  const n = raw ? Number.parseInt(raw, 10) : 800;
  return Number.isFinite(n) && n > 0 ? n : 800;
}
