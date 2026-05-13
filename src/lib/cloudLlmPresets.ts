/** OpenAI-compatible：内核会拼 `{base}/v1/chat/completions`，故 base 不要含 `/v1`。 */

export type CloudLlmPresetId =
  | "openai"
  | "deepseek"
  | "siliconflow"
  | "openrouter"
  | "moonshot"
  | "groq"
  | "together"
  | "custom";

export const CLOUD_LLM_PRESET_ORDER: CloudLlmPresetId[] = [
  "openai",
  "deepseek",
  "siliconflow",
  "openrouter",
  "moonshot",
  "groq",
  "together",
  "custom",
];

type PresetDefaults = { baseUrl: string; model: string };

export const CLOUD_LLM_PRESET_DEFAULTS: Record<Exclude<CloudLlmPresetId, "custom">, PresetDefaults> = {
  openai: { baseUrl: "https://api.openai.com", model: "gpt-4o-mini" },
  deepseek: { baseUrl: "https://api.deepseek.com", model: "deepseek-chat" },
  siliconflow: { baseUrl: "https://api.siliconflow.cn", model: "Qwen/Qwen2.5-7B-Instruct" },
  openrouter: { baseUrl: "https://openrouter.ai/api", model: "openai/gpt-4o-mini" },
  moonshot: { baseUrl: "https://api.moonshot.cn", model: "moonshot-v1-8k" },
  groq: { baseUrl: "https://api.groq.com/openai", model: "llama-3.3-70b-versatile" },
  together: { baseUrl: "https://api.together.xyz", model: "meta-llama/Llama-3.3-70B-Instruct-Turbo" },
};
