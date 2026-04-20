import TemplateEndpointConfig from "./TemplateEndpointConfig.vue";
import TemplateProviderSelector from "./TemplateProviderSelector.vue";
import TemplateSlotSelector from "./TemplateSlotSelector.vue";
import TemplateSwitchToggle from "./TemplateSwitchToggle.vue";

export type PluginUiTemplateName =
  | "endpoint-config"
  | "provider-selector"
  | "slot-selector"
  | "switch-toggle";

export const pluginUiTemplateMap = {
  "endpoint-config": TemplateEndpointConfig,
  "provider-selector": TemplateProviderSelector,
  "slot-selector": TemplateSlotSelector,
  "switch-toggle": TemplateSwitchToggle,
} as const;
