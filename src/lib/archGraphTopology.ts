/**
 * 插件管理架构图：三层自动连线 + 环上初始排列顺序。
 * 与 KERNEL_AND_MODULES / PLUGIN_V1 编排示意一致（非 co_present 逐步时序，而是拓扑层级）。
 */

import type { Edge } from '@vue-flow/core'
import type { BackendKind } from './graphEditorTheme'
import type { SlotRegistryEntry, SlotRegistryMap } from './slotRegistry'
import {
  ARCH_GRAPH_BUS_ID,
  ARCH_GRAPH_COMPLEX_ID,
  ARCH_GRAPH_KERNEL_ID,
} from '../composables/useArchitectureGraphModel'
import { normalizeBackendKind } from './graphEditorTheme'
import {

  sortedSlotRegistryEntries,
} from './slotRegistry'

/** 环上从顶部起顺时针：上弧 memory/emotion/event，下弧 prompt/llm/agent，complex 靠上。 */
export const ARCHITECTURE_RING_TYPE_ORDER = [
  'memory',
  'emotion',
  'event',
  'complex_emotion',
  'prompt',
  'llm',
  'agent',
] as const

export type ArchitectureWireLayer = 'kernel_bus' | 'bus_slot' | 'slot_plugin'

export function ringTypeIndex(slotType: string): number {
  const i = ARCHITECTURE_RING_TYPE_ORDER.indexOf(
    slotType as (typeof ARCHITECTURE_RING_TYPE_ORDER)[number],
  )
  return i >= 0 ? i : ARCHITECTURE_RING_TYPE_ORDER.length
}

/** 架构图环上槽位顺序：先按模块类型（编排拓扑），再按 position。 */
export function sortSlotsForArchitectureRing(
  registry: SlotRegistryMap,
): Array<[string, SlotRegistryEntry]> {
  return sortedSlotRegistryEntries(registry).sort((a, b) => {
    const ta = ringTypeIndex(a[1].type)
    const tb = ringTypeIndex(b[1].type)
    if (ta !== tb)
      return ta - tb
    return a[1].position - b[1].position
  })
}

/** 设施总线右侧输出口 id（complex_emotion 固定 fac-complex）。 */
export function busFacHandleForType(slotType: string): string {
  return slotType === 'complex_emotion' ? 'fac-complex' : `fac-${slotType}`
}

/** 总线节点上要绘制的 fac 口顺序（与环上一致）。 */
export function orderedBusFacTypes(registry: SlotRegistryMap): string[] {
  const present = new Set<string>()
  for (const e of Object.values(registry)) present.add(e.type)
  return ARCHITECTURE_RING_TYPE_ORDER.filter(t => present.has(t))
}

export function registryHasComplexEmotionSlot(registry: SlotRegistryMap): boolean {
  return Object.values(registry).some(e => e.type === 'complex_emotion')
}

type VisiblePluginsFn = (slotKey: string, entry: SlotRegistryEntry) => string[]

/**
 * 三层自动连线（只读系统边）：
 * 1. 编排中心 → 设施总线（pipeline）
 * 2. 设施总线 → 各 slot_registry 实例（fac-{type}）
 * 3. directory 槽位 → 目录插件子节点（plugin-out → plugin-in）
 */
export function buildBlueprintArchitectureEdges(
  registry: SlotRegistryMap,
  visiblePluginIds: VisiblePluginsFn,
): Edge[] {
  const out: Edge[] = []

  out.push({
    id: 'kernel-bus',
    source: ARCH_GRAPH_KERNEL_ID,
    target: ARCH_GRAPH_BUS_ID,
    sourceHandle: 'pipeline',
    targetHandle: 'pipeline-in',
    type: 'archBackend',
    deletable: false,
    updatable: false,
    data: { kind: 'builtin', system: true, layer: 'kernel_bus' satisfies ArchitectureWireLayer },
  })

  const entries = sortSlotsForArchitectureRing(registry)
  for (const [slotKey, entry] of entries) {
    const kind = normalizeBackendKind(entry.backend) as BackendKind
    out.push({
      id: `bus-${slotKey}`,
      source: ARCH_GRAPH_BUS_ID,
      target: slotKey,
      sourceHandle: busFacHandleForType(entry.type),
      targetHandle: 'backend-in',
      type: 'archBackend',
      deletable: false,
      updatable: false,
      data: {
        kind,
        slotKey,
        slotType: entry.type,
        system: true,
        layer: 'bus_slot' satisfies ArchitectureWireLayer,
      },
      animated: kind === 'remote',
    })

    for (const pid of visiblePluginIds(slotKey, entry)) {
      out.push({
        id: `slot-${slotKey}-${pid}`,
        source: slotKey,
        target: `plugin:${pid}`,
        sourceHandle: 'plugin-out',
        targetHandle: 'plugin-in',
        type: 'archBackend',
        deletable: false,
        updatable: false,
        data: {
          kind: 'directory',
          slotKey,
          slotType: entry.type,
          system: true,
          layer: 'slot_plugin' satisfies ArchitectureWireLayer,
        },
      })
    }
  }

  if (!registryHasComplexEmotionSlot(registry)) {
    out.push({
      id: 'bus-complex',
      source: ARCH_GRAPH_BUS_ID,
      target: ARCH_GRAPH_COMPLEX_ID,
      sourceHandle: 'fac-complex',
      targetHandle: 'backend-in',
      type: 'archBackend',
      deletable: false,
      updatable: false,
      data: { kind: 'builtin', system: true, layer: 'bus_slot' satisfies ArchitectureWireLayer },
    })
  }

  return out
}
