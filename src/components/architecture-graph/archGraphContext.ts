import type { InjectionKey } from 'vue'
import type { CoreModule } from '../../composables/useArchitectureGraphModel'

export interface ArchGraphActions {
  busy: () => boolean
  usesBlueprint: () => boolean
  /** legacy 六槽折叠：下拉即会话覆盖 */
  onBackendChange: (targetKey: string, value: string) => void
  /** v2 蓝图：仅本次会话 */
  onApplySessionOverride: (slotKey: string, backend: string) => void
  /** v2 蓝图：写入 pipeline.ocblueprint */
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
