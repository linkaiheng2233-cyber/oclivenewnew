/**
 * Plugin manager architecture graph: three-layer auto-wiring + initial ring order.
 * Matches KERNEL_AND_MODULES / PLUGIN_V1 orchestration sketch (topology layers, not co-present step timing).
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

/** Ring clockwise from top: upper arc memory/emotion/event, lower prompt/llm/agent, complex_emotion upper. */
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

/** Architecture ring slot order: module type (orchestration topology) first, then position. */
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

/** Facility bus right-side output handle id (complex_emotion fixed fac-complex). */
export function busFacHandleForType(slotType: string): string {
  return slotType === 'complex_emotion' ? 'fac-complex' : `fac-${slotType}`
}

/** Fac handle draw order on bus node (matches ring). */
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
 * Three-layer auto-wiring (read-only system edges):
 * 1. Orchestration center → facility bus (pipeline)
 * 2. Facility bus → each slot_registry instance (fac-{type})
 * 3. directory slot → directory plugin child nodes (plugin-out → plugin-in)
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
