/** Mirrors `EventTriggerMatchScope` in `oclive_kernel_runtime::models::expert_models`. */
export type EventTriggerMatchScope = "any" | "user_only" | "bot_only";

export type EventTriggerEvalNode = {
  type: "event_trigger";
  matchSubstring: string;
  memoryContent: string;
  enabled: boolean;
  importance?: number;
  matchScope?: EventTriggerMatchScope | null;
};

export function applyEventMemoryTemplate(template: string, needle: string): string {
  return template.split("{match}").join(needle).split("{keyword}").join(needle);
}

export function eventTriggerFires(
  scope: EventTriggerMatchScope,
  needle: string,
  userMessage: string,
  botReply: string,
): { fires: boolean; hitUser: boolean; hitBot: boolean } {
  const hitUser = userMessage.includes(needle);
  const hitBot = botReply.includes(needle);
  const s = scope ?? "any";
  const fires =
    s === "any"
      ? hitUser || hitBot
      : s === "user_only"
        ? hitUser
        : s === "bot_only"
          ? hitBot
          : hitUser || hitBot;
  return { fires, hitUser, hitBot };
}

export type EventTriggerPreviewResult =
  | {
      kind: "ok";
      fires: boolean;
      hitUser: boolean;
      hitBot: boolean;
      resolvedMemory: string;
    }
  | { kind: "skip"; reason: "disabled" | "empty_keyword" | "empty_memory" | "no_match" };

/** Dry-run evaluation for the expert workbench (aligned with kernel `expert_graph_events`). */
export function previewEventTrigger(
  node: EventTriggerEvalNode,
  userMessage: string,
  botReply: string,
): EventTriggerPreviewResult {
  if (node.enabled === false) {
    return { kind: "skip", reason: "disabled" };
  }
  const needle = (node.matchSubstring ?? "").trim();
  if (!needle) {
    return { kind: "skip", reason: "empty_keyword" };
  }
  const rawMem = (node.memoryContent ?? "").trim();
  if (!rawMem) {
    return { kind: "skip", reason: "empty_memory" };
  }
  const scope = (node.matchScope ?? "any") as EventTriggerMatchScope;
  const { fires, hitUser, hitBot } = eventTriggerFires(scope, needle, userMessage, botReply);
  if (!fires) {
    return { kind: "skip", reason: "no_match" };
  }
  return {
    kind: "ok",
    fires,
    hitUser,
    hitBot,
    resolvedMemory: applyEventMemoryTemplate(node.memoryContent ?? "", needle).trim(),
  };
}
