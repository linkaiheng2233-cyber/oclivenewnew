import { onMounted, ref } from "vue";

export type NodeLayoutEntry = {
  dx?: number;
  dy?: number;
  w?: number;
  h?: number;
};

const STORAGE_KEY = "oclive-arch-graph-layout-v3";

export const ARCH_NODE_DEFAULT_SIZE: Record<
  string,
  { width: number; height: number; minWidth: number; minHeight: number; maxWidth: number; maxHeight: number }
> = {
  archKernel: { width: 124, height: 124, minWidth: 100, minHeight: 100, maxWidth: 168, maxHeight: 168 },
  archBus: { width: 240, height: 100, minWidth: 200, minHeight: 80, maxWidth: 380, maxHeight: 220 },
  archModule: { width: 220, height: 112, minWidth: 176, minHeight: 88, maxWidth: 400, maxHeight: 340 },
  archPlugin: { width: 180, height: 72, minWidth: 148, minHeight: 56, maxWidth: 300, maxHeight: 180 },
  archComplex: { width: 200, height: 88, minWidth: 160, minHeight: 72, maxWidth: 300, maxHeight: 160 },
};

export function defaultSizeForNode(id: string, type?: string) {
  if (type && ARCH_NODE_DEFAULT_SIZE[type]) return ARCH_NODE_DEFAULT_SIZE[type];
  if (id.startsWith("plugin:")) return ARCH_NODE_DEFAULT_SIZE.archPlugin!;
  if (["memory", "emotion", "event", "prompt", "llm", "agent"].includes(id)) {
    return ARCH_NODE_DEFAULT_SIZE.archModule!;
  }
  return ARCH_NODE_DEFAULT_SIZE.archModule!;
}

export function useArchitectureGraphLayout() {
  const layout = ref<Record<string, NodeLayoutEntry>>({});

  function load(): void {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) layout.value = JSON.parse(raw) as Record<string, NodeLayoutEntry>;
    } catch {
      layout.value = {};
    }
  }

  function save(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(layout.value));
    } catch {
      /* ignore */
    }
  }

  function reset(): void {
    layout.value = {};
    save();
  }

  function get(id: string): NodeLayoutEntry {
    return layout.value[id] ?? {};
  }

  function setPosition(id: string, dx: number, dy: number): void {
    const cur = get(id);
    layout.value = { ...layout.value, [id]: { ...cur, dx, dy } };
  }

  function setSize(id: string, w: number, h: number): void {
    const cur = get(id);
    layout.value = { ...layout.value, [id]: { ...cur, w, h } };
  }

  function applyToNode(
    id: string,
    type: string | undefined,
    baseX: number,
    baseY: number,
  ): { x: number; y: number; width: number; height: number; style: Record<string, string> } {
    const entry = get(id);
    const def = defaultSizeForNode(id, type);
    const width = entry.w ?? def.width;
    const height = entry.h ?? def.height;
    const x = baseX + (entry.dx ?? 0);
    const y = baseY + (entry.dy ?? 0);
    return {
      x,
      y,
      width,
      height,
      style: { width: `${width}px`, minHeight: `${height}px` },
    };
  }

  onMounted(load);

  return { layout, load, save, reset, get, setPosition, setSize, applyToNode };
}
