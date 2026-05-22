import type { SlotRegistryMap } from "./slotRegistry";

/** 写盘：更新某实例键的 `backend`（其余字段保留）。 */
export function patchSlotRegistryBackend(
  pack: SlotRegistryMap,
  slotKey: string,
  backend: string,
): SlotRegistryMap {
  const entry = pack[slotKey];
  if (!entry) {
    throw new Error(`unknown slot key: ${slotKey}`);
  }
  return {
    ...pack,
    [slotKey]: { ...entry, backend },
  };
}
