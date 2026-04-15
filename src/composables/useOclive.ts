import { getCurrentInstance, onUnmounted } from "vue";
import { pluginBridgeInvoke } from "../utils/tauri-api";
import { hostEventBus } from "../lib/hostEventBus";

export interface OcliveEvents {
  emit(event: string, data?: unknown): void;
  on(event: string, handler: (data: unknown) => void): void;
  off(event: string, handler: (data: unknown) => void): void;
}

export interface OcliveApi {
  pluginId: string;
  bridgeAssetRel: string;
  invoke(command: string, params?: unknown): Promise<unknown>;
  events: OcliveEvents;
}

function makeEvents(): OcliveEvents {
  const inst = getCurrentInstance();
  return {
    emit(event: string, data?: unknown) {
      hostEventBus.emit(event, data);
    },
    on(event: string, handler: (data: unknown) => void) {
      const fn = handler as (d: unknown) => void;
      hostEventBus.on(event, fn);
      if (inst) {
        onUnmounted(() => hostEventBus.off(event, fn), inst);
      }
    },
    off(event: string, handler: (data: unknown) => void) {
      hostEventBus.off(event, handler as (d: unknown) => void);
    },
  };
}

/** 供 `provide('oclive', …)`；`bridgeAssetRel` 为 manifest 资源路径（插槽 `entry`、或整壳 `shell.vueEntry` 等），与 `plugin_bridge_invoke` 的 `assetRel` 一致。 */
export function createOcliveApi(
  pluginId: string,
  bridgeAssetRel: string,
): OcliveApi {
  return {
    pluginId,
    bridgeAssetRel,
    async invoke(command: string, params?: unknown) {
      return pluginBridgeInvoke({
        pluginId,
        assetRel: bridgeAssetRel,
        command,
        params,
      });
    },
    events: makeEvents(),
  };
}
