import type { InjectionKey } from "vue";
import type { CoreModule } from "../../composables/useArchitectureGraphModel";

export type ArchGraphActions = {
  busy: () => boolean;
  onBackendChange: (module: CoreModule, value: string) => void;
  onFocusPlugin: (id: string) => void;
  onToggleExpand: (module: CoreModule) => void;
  onTogglePluginDisabled: (id: string) => void;
  onUninstallPlugin: (id: string) => void;
};

export const archGraphActionsKey: InjectionKey<ArchGraphActions> = Symbol("archGraphActions");
