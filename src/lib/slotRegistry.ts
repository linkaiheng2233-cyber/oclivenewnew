/** 与 `pipeline.ocblueprint` v2 `slot_registry` 条目一致（serde `type` 字段）。 */
export interface SlotRegistryEntry {
  type: string;
  label: string;
  backend: string;
  position: number;
  plugin?: string | null;
  plugins?: string[] | null;
  model?: string | null;
  url?: string | null;
  local_memory_provider_id?: string | null;
}

export type SlotRegistryMap = Record<string, SlotRegistryEntry>;

const SLOT_TYPE_ORDER = [
  "emotion",
  "complex_emotion",
  "event",
  "memory",
  "prompt",
  "llm",
  "agent",
] as const;

export type SlotType = (typeof SLOT_TYPE_ORDER)[number];

export const SLOT_BACKEND_OPTIONS: Record<string, string[]> = {
  memory: ["builtin", "builtin_v2", "remote", "local", "directory"],
  emotion: ["builtin", "builtin_v2", "remote", "directory"],
  event: ["builtin", "builtin_v2", "remote", "directory"],
  prompt: ["builtin", "builtin_v2", "remote", "directory"],
  llm: ["ollama", "remote", "directory"],
  agent: ["builtin", "remote", "directory"],
  complex_emotion: ["builtin", "remote", "directory"],
};

export const SLOT_TYPE_LABEL_KEYS: Record<string, string> = {
  memory: "pluginWorkbench.graph.memory",
  emotion: "pluginWorkbench.graph.emotion",
  event: "pluginWorkbench.graph.event",
  prompt: "pluginWorkbench.graph.prompt",
  llm: "pluginWorkbench.graph.llm",
  agent: "pluginWorkbench.graph.agent",
  complex_emotion: "pluginWorkbench.graph.complexEmotion",
};

export const SLOT_TYPE_ICONS: Record<string, string> = {
  memory: "🧠",
  emotion: "💭",
  event: "⚡",
  prompt: "📝",
  llm: "🤖",
  agent: "🛠",
  complex_emotion: "✨",
};

export function sortedSlotRegistryEntries(
  registry: SlotRegistryMap,
): Array<[string, SlotRegistryEntry]> {
  return Object.entries(registry).sort((a, b) => {
    const ta = slotTypeOrderIndex(a[1].type);
    const tb = slotTypeOrderIndex(b[1].type);
    if (ta !== tb) return ta - tb;
    return a[1].position - b[1].position;
  });
}

function slotTypeOrderIndex(t: string): number {
  const i = SLOT_TYPE_ORDER.indexOf(t as SlotType);
  return i >= 0 ? i : SLOT_TYPE_ORDER.length;
}

export function uniqueSlotTypes(registry: SlotRegistryMap): string[] {
  const set = new Set<string>();
  for (const e of Object.values(registry)) {
    set.add(e.type);
  }
  return [...set].sort((a, b) => slotTypeOrderIndex(a) - slotTypeOrderIndex(b));
}

export function primaryPluginId(entry: SlotRegistryEntry): string {
  const p = entry.plugin?.trim();
  if (p) return p;
  const first = entry.plugins?.find((x) => x.trim().length > 0);
  return first?.trim() ?? "";
}
