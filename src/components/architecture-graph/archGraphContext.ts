import type { InjectionKey } from "vue";
import type { CoreModule } from "../../composables/useArchitectureGraphModel";

export type ArchGraphActions = {
  busy: () => boolean;
  usesBlueprint: () => boolean;
  onBackendChange: (targetKey: string, value: string) => void;
  onClearSlotOverride: (slotKey: string) => void;
  onFocusPlugin: (id: string) => void;
  onToggleExpand: (targetKey: string) => void;
  onToggleGroupCollapse: (groupId: string) => void;
  onTogglePluginDisabled: (id: string) => void;
  onUninstallPlugin: (id: string) => void;
};

export const archGraphActionsKey: InjectionKey<ArchGraphActions> = Symbol("archGraphActions");
