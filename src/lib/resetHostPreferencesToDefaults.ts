/**
 * 宿主侧偏好「恢复默认」：写入本机数据库 / 配置文件，不涉及角色包磁盘或插件卸载。
 * 调用方负责 Pinia 刷新与 `notifyHostModelsInventoryChanged`。
 *
 * 覆盖项与 `settings.globalReset.scope` 文案应对齐；默认 Ollama 模型 id 与内核在 DB 为空时的回退一致
 * （`crates/oclive_kernel_runtime/src/state/app_state.rs`）。
 */
import {
  getPluginState,
  saveGlobalPluginState,
  saveHotkeyBindings,
  setHostChatModel,
  setHostCloudLlm,
  setPluginIndexSources,
  setPluginMarketDeveloperMode,
  type HotkeyBindingsFile,
} from "../utils/tauri-api";

export const SETTINGS_DEFAULT_HOST_CHAT_MODEL_ID = "qwen2.5:7b";

export async function resetHostPreferencesToDefaults(roleId: string): Promise<void> {
  await setHostCloudLlm({ baseUrl: "", apiKey: "" });
  await setHostChatModel(SETTINGS_DEFAULT_HOST_CHAT_MODEL_ID);
  const emptyHotkeys: HotkeyBindingsFile = { schemaVersion: 1, bindings: [] };
  await saveHotkeyBindings(emptyHotkeys);
  await setPluginMarketDeveloperMode(false);
  await setPluginIndexSources([]);
  const bundle = await getPluginState(roleId);
  await saveGlobalPluginState({
    ...bundle.globalDefaults,
    force_iframe_mode: false,
  });
}
