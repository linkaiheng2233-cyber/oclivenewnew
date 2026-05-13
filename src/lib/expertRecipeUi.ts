import type { ExpertGraph, ExpertNode, PromptStyleOverride } from "../utils/tauri-api";

function basenameGguf(path: string): string {
  const p = (path ?? "").trim().replace(/\\/g, "/");
  const seg = p.split("/").pop() ?? p;
  return seg || path;
}

export function isExpertGraphEmpty(graph: ExpertGraph | null | undefined): boolean {
  const n = graph?.nodes;
  return !Array.isArray(n) || n.length === 0;
}

export type ExpertRecipeParts = {
  cloudModels: string[];
  loraBits: string[];
  baseModels: string[];
  eventTriggerCount: number;
};

export function collectExpertRecipeParts(graph: ExpertGraph): ExpertRecipeParts {
  const nodes = graph?.nodes ?? [];
  const clouds = nodes.filter(
    (x): x is Extract<ExpertNode, { type: "cloud_model" }> =>
      x.type === "cloud_model" && x.enabled,
  );
  const loras = nodes.filter(
    (x): x is Extract<ExpertNode, { type: "lora_adapter" }> =>
      x.type === "lora_adapter" && x.enabled,
  );
  const bases = nodes.filter((x): x is Extract<ExpertNode, { type: "base_model" }> => x.type === "base_model");
  const triggers = nodes.filter(
    (x): x is Extract<ExpertNode, { type: "event_trigger" }> =>
      x.type === "event_trigger" && x.enabled,
  );
  return {
    cloudModels: clouds.map((c) => {
      const m = (c.model ?? "").trim();
      return m || "cloud";
    }),
    loraBits: loras.map((l) => {
      const name = basenameGguf(l.ggufPath ?? "");
      const s = typeof l.strength === "number" && Number.isFinite(l.strength) ? l.strength : 0;
      return `${name}×${s.toFixed(2)}`;
    }),
    baseModels: bases.map((b) => basenameGguf(b.ggufPath ?? "")),
    eventTriggerCount: triggers.length,
  };
}

/** One-line fallback (locale-neutral), e.g. for logs. */
export function buildExpertRecipeSummaryLine(graph: ExpertGraph): string {
  const p = collectExpertRecipeParts(graph);
  const parts: string[] = [];
  if (p.cloudModels.length) parts.push(p.cloudModels.join(" · "));
  if (p.loraBits.length) parts.push(p.loraBits.join("，"));
  if (p.baseModels.length) parts.push(p.baseModels.join(" · "));
  if (p.eventTriggerCount) parts.push(`events×${p.eventTriggerCount}`);
  return parts.join(" · ") || "—";
}

export function formatExpertConfigDetailJson(
  graph: ExpertGraph,
  promptStyle: PromptStyleOverride | null | undefined,
): string {
  const body = {
    graph,
    promptStyle: promptStyle ?? null,
  };
  return `${JSON.stringify(body, null, 2)}\n`;
}

export type ExpertRecipeUiMode = "pure" | "role_default" | "session_override";

export function resolveExpertRecipeUiMode(
  graphSource: "pack_default" | "role_default" | "session_override",
  graph: ExpertGraph,
): ExpertRecipeUiMode {
  if (graphSource === "session_override") return "session_override";
  if (graphSource === "role_default" && !isExpertGraphEmpty(graph)) return "role_default";
  return "pure";
}
