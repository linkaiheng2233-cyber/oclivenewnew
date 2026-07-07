<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDirectoryPluginSlotEmbed } from '@oclive/shared/composables/useDirectoryPluginSlotEmbed'
import { PURE_CHAT_PLATFORM_PLUGIN_IDS, SLOT_SETTINGS_PANEL } from '@oclive/shared/stores/pluginStore'
import AsyncPluginVue from './AsyncPluginVue.vue'
import PluginErrorPlaceholder from './PluginErrorPlaceholder.vue'

const props = withDefaults(
  defineProps<{
    /** Bump to refetch bootstrap after changes (synced with plugin manager save) */
    bootstrapEpoch?: number
    /** In pure_chat, only show platform side-channel settings plugins (e.g. voice.asr). */
    platformOnly?: boolean
    /** Only embed slots from these plugin ids. */
    pluginIdAllowlist?: readonly string[] | null
    /** Hide slots from these plugin ids. */
    pluginIdDenylist?: readonly string[] | null
    /** Hide tab bar when a single plugin panel is shown. */
    hideTabs?: boolean
    /** Fill parent height and scroll inside plugin panel (voice settings page). */
    fillHeight?: boolean
  }>(),
  {
    bootstrapEpoch: 0,
    platformOnly: false,
    pluginIdAllowlist: null,
    pluginIdDenylist: null,
    hideTabs: false,
    fillHeight: false,
  },
)

const { t } = useI18n()

const pluginIdAllowlist = computed(() => {
  if (props.pluginIdAllowlist?.length)
    return props.pluginIdAllowlist
  if (props.platformOnly)
    return PURE_CHAT_PLATFORM_PLUGIN_IDS
  return null
})

const pluginIdDenylist = computed(() => props.pluginIdDenylist ?? null)

function slotTabLabel(s: { pluginId: string, label?: string | null }): string {
  const label = s.label?.trim()
  return label || s.pluginId
}

const {
  pluginError,
  slots: panelSlots,
  frameErrors,
  frameErrorDetails,
  reloadNonceFor,
  onFrameError,
  onFrameLoad,
  onVueFailed,
  onVueCompileError,
  retrySlot,
  showIframe,
  showVue,
} = useDirectoryPluginSlotEmbed({
  slot: SLOT_SETTINGS_PANEL,
  bootstrapEpoch: () => props.bootstrapEpoch,
  pluginIdAllowlist,
  pluginIdDenylist,
})

const showTabBar = computed(() => !props.hideTabs && panelSlots.value.length > 1)

const activeTab = ref(0)

watch(panelSlots, (list) => {
  if (activeTab.value >= list.length) {
    activeTab.value = 0
  }
})
</script>

<template>
  <div class="psp-root" :class="{ 'psp-root--fill': props.fillHeight }">
    <div v-if="pluginError" class="psp-msg psp-msg--err" role="status">
      {{ pluginError }}
    </div>
    <template v-else-if="panelSlots.length > 0">
      <div v-if="showTabBar" class="psp-tabs" role="tablist" :aria-label="t('pluginManager.slotsAria.settingsPanelTablist')">
        <button
          v-for="(s, i) in panelSlots"
          :key="`${s.pluginId}:${s.appearanceId ?? ''}`"
          type="button"
          role="tab"
          class="psp-tab"
          :aria-selected="activeTab === i"
          @click="activeTab = i"
        >
          {{ slotTabLabel(s) }}
        </button>
      </div>
      <div
        v-for="(s, i) in panelSlots"
        v-show="activeTab === i"
        :key="`frame-${s.pluginId}-${s.appearanceId ?? ''}`"
        class="psp-frame-wrap"
        :class="{ 'psp-frame-wrap--fill': props.fillHeight }"
        role="tabpanel"
      >
        <AsyncPluginVue
          v-if="showVue(s)"
          :key="`vue-${s.pluginId}-${s.appearanceId ?? ''}-${reloadNonceFor(s.pluginId)}`"
          class="psp-vue"
          :class="{ 'psp-vue--fill': props.fillHeight }"
          :plugin-id="s.pluginId"
          :vue-component="s.vueComponent!"
          :bridge-asset-rel="s.entry"
          :reload-nonce="reloadNonceFor(s.pluginId)"
          skeleton-variant="block"
          @failed="onVueFailed(s.pluginId)"
          @compile-error="onVueCompileError(s.pluginId, $event)"
        />
        <iframe
          v-if="showIframe(s)"
          :key="`if-${s.pluginId}-${s.appearanceId ?? ''}-${reloadNonceFor(s.pluginId)}`"
          class="psp-frame"
          :src="s.url"
          :title="`plugin settings ${s.pluginId}`"
          loading="lazy"
          referrerpolicy="no-referrer"
          @load="onFrameLoad(s.pluginId)"
          @error="onFrameError(s.pluginId)"
        />
        <PluginErrorPlaceholder
          v-if="frameErrors[s.pluginId]"
          class="psp-fail"
          :message="frameErrors[s.pluginId]!"
          :detail="frameErrorDetails[s.pluginId] || undefined"
          :show-fallback="false"
          @retry="retrySlot(s)"
        />
      </div>
    </template>
    <p v-else class="psp-msg psp-msg--muted">
      {{ t("pluginManager.slotsAria.settingsPanelEmpty", { slot: SLOT_SETTINGS_PANEL }) }}
    </p>
  </div>
</template>

<style scoped>
.psp-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
}
.psp-root--fill {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  gap: 0;
}
.psp-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.psp-tab {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
  cursor: pointer;
  color: var(--text-primary);
}
.psp-tab[aria-selected="true"] {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 12%, var(--bg-elevated));
}
.psp-frame-wrap {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.psp-frame-wrap--fill {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  gap: 0;
}
.psp-frame {
  width: 100%;
  min-height: 260px;
  flex: 1;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: var(--bg-elevated);
}
.psp-vue {
  width: 100%;
  flex: 0 0 auto;
}
.psp-vue--fill {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.psp-msg {
  margin: 0;
  font-size: 13px;
}
.psp-msg--err {
  color: var(--text-danger, #c33);
}
.psp-msg--warn {
  font-size: 12px;
  color: var(--text-secondary);
}
.psp-msg--muted {
  color: var(--text-secondary);
  font-size: 13px;
}
code {
  font-size: 0.9em;
}
</style>

<style>
@import '@oclive/shared/styles/win98/component-voice-settings.css';
</style>
