import { onMounted, ref } from 'vue'

export interface NodeOffset { dx: number, dy: number }

const STORAGE_KEY = 'oclive-arch-graph-node-offsets-v2'

export function useGraphNodeLayout() {
  const offsets = ref<Record<string, NodeOffset>>({})

  function load(): void {
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      if (raw)
        offsets.value = JSON.parse(raw) as Record<string, NodeOffset>
    }
    catch {
      offsets.value = {}
    }
  }

  function save(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(offsets.value))
    }
    catch {
      /* ignore quota */
    }
  }

  function reset(): void {
    offsets.value = {}
    save()
  }

  function get(id: string): NodeOffset {
    return offsets.value[id] ?? { dx: 0, dy: 0 }
  }

  function shift(id: string, dx: number, dy: number): void {
    const cur = get(id)
    offsets.value = { ...offsets.value, [id]: { dx: cur.dx + dx, dy: cur.dy + dy } }
  }

  function apply(id: string, x: number, y: number, cx: number, cy: number) {
    const o = get(id)
    return { x: x + o.dx, y: y + o.dy, cx: cx + o.dx, cy: cy + o.dy }
  }

  onMounted(load)

  return { offsets, load, save, reset, get, shift, apply }
}
