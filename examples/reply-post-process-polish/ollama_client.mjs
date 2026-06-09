/**
 * Minimal Ollama chat client for polish pass (system + user, non-streaming).
 */

/**
 * @param {{ system: string, user: string, baseUrl?: string, model?: string }} opts
 * @returns {Promise<string>}
 */
export async function polishWithOllama(opts) {
  const baseUrl = (opts.baseUrl ?? readOllamaUrl()).replace(/\/+$/, "");
  const model = opts.model ?? readPolishModel();
  if (!model) {
    throw new Error("OCLIVE_POLISH_MODEL not set");
  }

  const res = await fetch(`${baseUrl}/api/chat`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      model,
      stream: false,
      messages: [
        { role: "system", content: opts.system },
        { role: "user", content: opts.user },
      ],
    }),
  });

  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`ollama ${res.status}: ${body.slice(0, 200)}`);
  }

  const data = await res.json();
  const content = data?.message?.content;
  if (typeof content !== "string" || !content.trim()) {
    throw new Error("ollama empty response");
  }
  return content.trim();
}

export function readOllamaUrl() {
  return process.env.OCLIVE_POLISH_OLLAMA_URL?.trim() || "http://127.0.0.1:11434";
}

export function readPolishModel() {
  return process.env.OCLIVE_POLISH_MODEL?.trim() || "";
}

/**
 * @param {string} userMessage
 * @param {string} rawReply
 * @param {string} locale
 */
export function buildPolishUserBlock(userMessage, rawReply, locale) {
  const loc = locale?.trim() || "zh";
  return [
    `语言：${loc}`,
    `用户说：${userMessage || "（空）"}`,
    `角色初稿：${rawReply}`,
    "请按 system 规则润色，只输出修正后的角色台词。",
  ].join("\n");
}
