import { usePluginStore } from "../stores/pluginStore";

export type ExpertWorkbenchDraftMode = "effective" | "role_default";

/**
 * 全局唯一入口：请求打开专家模型工作台（V2 浮窗）。
 * 由 `App.vue` 消费 `pluginStore.expertModelsWorkbenchRequestEpoch` 并 `refresh` + `applyWorkbenchNavigationDraft`。
 */
export function openExpertWorkbenchEdit(opts?: { draftMode?: ExpertWorkbenchDraftMode }): void {
  usePluginStore().requestOpenExpertModelsWorkbench({
    draftMode: opts?.draftMode ?? "effective",
  });
}
