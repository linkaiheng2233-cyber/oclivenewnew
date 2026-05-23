import TemplateEndpointConfig from './TemplateEndpointConfig.vue'
import TemplateProviderSelector from './TemplateProviderSelector.vue'
import TemplateSlotRegistry from './TemplateSlotRegistry.vue'
import TemplateSlotSelector from './TemplateSlotSelector.vue'
import TemplateSwitchToggle from './TemplateSwitchToggle.vue'

export type PluginUiTemplateName
  = | 'endpoint-config'
    | 'provider-selector'
    | 'slot-selector'
    | 'slot-registry'
    | 'switch-toggle'

export const pluginUiTemplateMap = {
  'endpoint-config': TemplateEndpointConfig,
  'provider-selector': TemplateProviderSelector,
  'slot-selector': TemplateSlotSelector,
  'slot-registry': TemplateSlotRegistry,
  'switch-toggle': TemplateSwitchToggle,
} as const
