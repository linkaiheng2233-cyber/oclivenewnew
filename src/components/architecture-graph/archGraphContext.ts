import type { InjectionKey } from 'vue'
import type { CoreModule } from '../../composables/useArchitectureGraphModel'

export interface ArchGraphActions {
  busy: () => boolean
  usesBlueprint: () => boolean
  /** Legacy six-slot fold: dropdown applies session override */
  onBackendChange: (targetKey: string, value: string) => void
  /** v2 blueprint: session-only override */
  onApplySessionOverride: (slotKey: string, backend: string) => void
  /** v2 blueprint: persist to pipeline.ocblueprint */
  onApplyPackDefault: (slotKey: string, backend: string) => void
  onClearSlotOverride: (slotKey: string) => void
  onFocusPlugin: (id: string) => void
  onToggleExpand: (targetKey: string) => void
  onToggleGroupCollapse: (groupId: string) => void
  onTogglePluginDisabled: (id: string) => void
  onUninstallPlugin: (id: string) => void
  onExpertConfigure: (slotKey: string, slotType: string) => void
}

export const archGraphActionsKey: InjectionKey<ArchGraphActions> = Symbol('archGraphActions')
