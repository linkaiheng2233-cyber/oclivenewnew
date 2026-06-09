/**
 * Rule gate: only call LLM when raw reply likely needs polish.
 */

const MARKDOWN_OR_CODE = /```|(^|\n)\s*#{1,6}\s|\*\*[^*]+\*\*|(^|\n)\s*[-*]\s+/m;

/**
 * @param {string} raw
 * @param {string} userMessage
 * @returns {boolean}
 */
export function shouldPolish(raw, userMessage) {
  const text = typeof raw === "string" ? raw.trim() : "";
  if (!text) {
    return false;
  }

  const user = typeof userMessage === "string" ? userMessage.trim() : "";
  if (user.length >= 4 && echoesUserOpening(text, user)) {
    return true;
  }

  if (MARKDOWN_OR_CODE.test(text)) {
    return true;
  }

  if (text.length > 1200) {
    return true;
  }

  return false;
}

/**
 * Detect when the reply repeats the user's opening phrase (common LLM echo).
 *
 * @param {string} raw
 * @param {string} user
 */
function echoesUserOpening(raw, user) {
  const snippet = user.slice(0, Math.min(24, user.length)).trim();
  if (snippet.length < 4) {
    return false;
  }
  const lowerRaw = raw.toLowerCase();
  const lowerSnippet = snippet.toLowerCase();
  if (lowerRaw.startsWith(lowerSnippet)) {
    return true;
  }
  const firstLine = raw.split(/\r?\n/, 1)[0]?.trim().toLowerCase() ?? "";
  return firstLine.startsWith(lowerSnippet);
}
