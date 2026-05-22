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
  /** v3 可选：`stable` / `experimental` 或数组 */
  zone?: string | string[] | null;
}

/** 将蓝图 `zone` 规范为展示用短标签。 */
export function formatSlotZoneLabel(zone: SlotRegistryEntry["zone"]): string | null {
  if (zone == null) return null;
  if (typeof zone === "string") {
    const z = zone.trim().toLowerCase();
    if (!z) return null;
    return z;
  }
  if (Array.isArray(zone)) {
    const parts = zone
      .map((z) => String(z).trim().toLowerCase())
      .filter(Boolean);
    return parts.length ? parts.join("+") : null;
  }
  return null;
}

export type SlotRegistryMap = Record<string, SlotRegistryEntry>;

/** `pipeline.ocblueprint` v2 `groups` entry. */
export interface SlotGroupEntry {
  label: string;
  description?: string | null;
  type: string;
  members: string[];
}

export type SlotGroupsMap = Record<string, SlotGroupEntry>;

/** Group border accent by slot `type` (architecture graph). */
export const SLOT_TYPE_GROUP_COLORS: Record<string, string> = {
  memory: "#6d9a7d",
  emotion: "#c9a86c",
  event: "#d4846a",
  prompt: "#7a92b0",
  llm: "#9a88a6",
  agent: "#8b9db8",
};

/** 校验/列表排序；架构图环上顺序见 `archGraphTopology.ARCHITECTURE_RING_TYPE_ORDER`。 */
export const SLOT_TYPE_ORDER = [
  "memory",
  "emotion",
  "event",
  "complex_emotion",
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

/** 新实例键：优先用 type 名，冲突时 `type_2`、`type_3`… */
export function nextUniqueSlotKey(registry: SlotRegistryMap, slotType: string): string {
  if (!registry[slotType]) return slotType;
  let i = 2;
  while (registry[`${slotType}_${i}`]) i += 1;
  return `${slotType}_${i}`;
}

export function nextPositionForType(registry: SlotRegistryMap, slotType: string): number {
  let max = -1;
  for (const e of Object.values(registry)) {
    if (e.type === slotType && e.position > max) max = e.position;
  }
  return max + 1;
}

const DEFAULT_BACKEND: Record<string, string> = {
  memory: "builtin",
  emotion: "builtin",
  event: "builtin",
  prompt: "builtin",
  llm: "ollama",
  agent: "builtin",
  complex_emotion: "builtin",
};

export function defaultBackendForSlotType(slotType: string): string {
  return DEFAULT_BACKEND[slotType] ?? "builtin";
}

export function countSlotsOfType(registry: SlotRegistryMap, slotType: string): number {
  return Object.values(registry).filter((e) => e.type === slotType).length;
}

/** 最后一个 `llm` 实例不可删。 */
export function canRemoveSlotKey(registry: SlotRegistryMap, key: string): boolean {
  const entry = registry[key];
  if (!entry) return false;
  if (entry.type !== "llm") return true;
  return countSlotsOfType(registry, "llm") > 1;
}

export function addSlotToRegistry(
  registry: SlotRegistryMap,
  slotType: string,
  label: string,
): { registry: SlotRegistryMap; key: string } {
  const key = nextUniqueSlotKey(registry, slotType);
  const trimmed = label.trim() || key;
  return {
    key,
    registry: {
      ...registry,
      [key]: {
        type: slotType,
        label: trimmed,
        backend: defaultBackendForSlotType(slotType),
        position: nextPositionForType(registry, slotType),
      },
    },
  };
}

export const SLOT_REGISTRY_LAST_LLM = "CANNOT_REMOVE_LAST_LLM";

export function removeSlotFromRegistry(
  registry: SlotRegistryMap,
  key: string,
): SlotRegistryMap {
  if (!canRemoveSlotKey(registry, key)) {
    throw new Error(SLOT_REGISTRY_LAST_LLM);
  }
  const next = { ...registry };
  delete next[key];
  return next;
}

export function primaryPluginId(entry: SlotRegistryEntry): string {
  const p = entry.plugin?.trim();
  if (p) return p;
  const first = entry.plugins?.find((x) => x.trim().length > 0);
  return first?.trim() ?? "";
}
