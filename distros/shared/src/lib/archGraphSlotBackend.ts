import type { SlotRegistryMap } from './slotRegistry'

/** Persist: update `backend` for one instance key (other fields preserved). */
export function patchSlotRegistryBackend(
  pack: SlotRegistryMap,
  slotKey: string,
  backend: string,
): SlotRegistryMap {
  const entry = pack[slotKey]
  if (!entry) {
    throw new Error(`unknown slot key: ${slotKey}`)
  }
  return {
    ...pack,
    [slotKey]: { ...entry, backend },
  }
}
