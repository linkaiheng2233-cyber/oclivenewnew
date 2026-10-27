<script setup lang="ts">
import UiCloseButton from '@oclive/shared/components/ui/UiCloseButton.vue'
import { VOICE_ASR_PLUGIN_ID } from '@oclive/shared/lib/voiceAsrEvents'
import { useAdultInteractionStore } from '@oclive/shared/stores/adultInteractionStore'
import {
  isPureChatPlatformPlugin,
  SLOT_SETTINGS_PANEL,
  usePluginStore,
} from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, ref, Teleport, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    visible: boolean
    embedded?: boolean
    /** When set, switches the active tab (e.g. open from StatusBar identity link). */
    focusTab?: SettingsTab | null
    storageRoleId?: string
    basicWorkspace?: boolean
  }>(),
  { embedded: false, focusTab: null },
)
const emit = defineEmits<{
  close: []
}>()
const ChatStorageSettingsPanel = defineAsyncComponent(() => import('@oclive/shared/components/settings/ChatStorageSettingsPanel.vue'))
const SettingsGeneralTab = defineAsyncComponent(() => import('@oclive/shared/components/settings/SettingsGeneralTab.vue'))
const SettingsPluginsTab = defineAsyncComponent(() => import('@oclive/shared/components/settings/SettingsPluginsTab.vue'))
const SettingsVoiceTab = defineAsyncComponent(() => import('@oclive/shared/components/settings/SettingsVoiceTab.vue'))
const SettingsAdultTab = defineAsyncComponent(() => import('@oclive/shared/components/settings/SettingsAdultTab.vue'))

export type SettingsTab = 'general' | 'voice' | 'plugins' | 'storage' | 'adult'

const { t } = useI18n()
const pluginStore = usePluginStore()
const roleStore = useRoleStore()
const uiStore = useUiStore()
const adultStore = useAdultInteractionStore()

type GeneralSubTab = 'simple' | 'advanced'

const tab = ref<SettingsTab>('general')
const generalSubTab = ref<GeneralSubTab>('simple')
const generalPage = ref('appearance')
const generalPages = ['appearance', 'chat', 'roles', 'connection', 'advanced']
const showVoiceSettingsTab = computed(() =>
  !props.basicWorkspace && uiStore.advancedFeaturesEnabled && (pluginStore.bootstrapUiSlots ?? []).some(
    s => s.slot === SLOT_SETTINGS_PANEL && s.pluginId === VOICE_ASR_PLUGIN_ID,
  ),
)

const showPluginsSettingsTab = computed(() => {
  if (props.basicWorkspace || !uiStore.advancedFeaturesEnabled)
    return false
  if (roleStore.interactionImmersive) {
    return (pluginStore.bootstrapUiSlots ?? []).some(
      s => s.slot === SLOT_SETTINGS_PANEL && s.pluginId !== VOICE_ASR_PLUGIN_ID,
    )
  }
  return (pluginStore.bootstrapUiSlots ?? []).some(
    s => s.slot === SLOT_SETTINGS_PANEL
      && isPureChatPlatformPlugin(s.pluginId)
      && s.pluginId !== VOICE_ASR_PLUGIN_ID,
  )
})

const showAdultSettingsTab = computed(() => (!props.basicWorkspace && uiStore.advancedFeaturesEnabled) || adultStore.globalEnabled)
const navigation = computed(() => [
  { id: 'appearance', label: t('settings.pages.appearance.title'), icon: 'M12 3a9 9 0 1 0 9 9c0-2-3-1-4-1s-2-1-1-2c2-2 0-6-4-6Z M7 10h.01M10 6h.01M6 14h.01' },
  { id: 'chat', label: t('settings.pages.chat.title'), icon: 'M4 4h16v12H9l-5 4V4Z M8 8h8M8 12h5' },
  { id: 'roles', label: t('settings.pages.roles.title'), icon: 'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z M4 21v-3a8 8 0 0 1 16 0v3' },
  { id: 'storage', label: t('settings.tabStorage'), icon: 'M4 4h16v16H4V4Z M4 9h16M8 13h8M8 16h5' },
  { id: 'connection', label: t('settings.pages.connection.title'), icon: 'M9 3v5M15 3v5M6 8h12v3a6 6 0 0 1-12 0V8Z M12 17v4' },
  { id: 'advanced', label: t('settings.pages.advanced.title'), icon: 'M4 6h16M4 12h16M4 18h16M8 3v6M16 9v6M10 15v6' },
  ...(showVoiceSettingsTab.value ? [{ id: 'voice', label: t('settings.tabVoice'), icon: 'M9 5a3 3 0 0 1 6 0v7a3 3 0 0 1-6 0V5Z M5 11a7 7 0 0 0 14 0M12 18v4' }] : []),
  ...(showPluginsSettingsTab.value ? [{ id: 'plugins', label: t('settings.tabPlugins'), icon: 'M4 4h6v4h4V4h6v6h-4v4h4v6H4V4Z' }] : []),
  ...(showAdultSettingsTab.value ? [{ id: 'adult', label: t('settings.tabAdult'), icon: 'M12 3 3 7v5c0 5 9 9 9 9s9-4 9-9V7l-9-4Z' }] : []),
])
const selectedPage = computed(() => tab.value === 'general' ? generalPage.value : tab.value)
const pageTitle = computed(() => navigation.value.find(item => item.id === selectedPage.value)?.label)
const pageDescription = computed(() => generalPages.includes(selectedPage.value)
  ? t(`settings.pages.${selectedPage.value}.description`)
  : t('settings.pages.optionalDescription'))

function selectPage(next: string) {
  if (generalPages.includes(next)) {
    tab.value = 'general'
    generalPage.value = next
    generalSubTab.value = next === 'advanced' || next === 'connection' ? 'advanced' : 'simple'
  }
  else {
    tab.value = availableTab(next as SettingsTab)
  }
}

function availableTab(next: SettingsTab): SettingsTab {
  if ((next === 'voice' && !showVoiceSettingsTab.value)
    || (next === 'plugins' && !showPluginsSettingsTab.value)
    || (next === 'adult' && !showAdultSettingsTab.value)) {
    return 'general'
  }
  return next
}

watch(
  () => props.focusTab,
  (next) => {
    if (next)
      tab.value = availableTab(next)
  },
  { immediate: true },
)

watch(
  () => props.visible,
  (open) => {
    if (open && props.focusTab)
      tab.value = availableTab(props.focusTab)
  },
)

const settingsDialogRef = ref<HTMLElement | null>(null)
let previousFocus: HTMLElement | null = null

function restoreFocus() {
  if (previousFocus?.isConnected)
    previousFocus.focus({ preventScroll: true })
  previousFocus = null
}

function onDialogKeydown(event: KeyboardEvent) {
  if (props.embedded || event.key !== 'Tab')
    return
  const dialog = settingsDialogRef.value
  const controls = [...(dialog?.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea:not(:disabled),a[href],[tabindex="0"],summary') ?? [])].filter(element => element.getClientRects().length > 0)
  const first = controls[0]
  const last = controls.at(-1)
  if (!first || !last) {
    event.preventDefault()
    return
  }
  if (event.shiftKey && (document.activeElement === first || document.activeElement === dialog)) {
    event.preventDefault()
    last.focus()
  }
  else if (!event.shiftKey && (document.activeElement === last || document.activeElement === dialog)) {
    event.preventDefault()
    first.focus()
  }
}
onBeforeUnmount(restoreFocus)

watch(
  [showVoiceSettingsTab, showPluginsSettingsTab, showAdultSettingsTab],
  () => { tab.value = availableTab(tab.value) },
)

watch(
  () => props.visible,
  (v) => {
    if (v && !props.embedded) {
      previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
      void nextTick(() => {
        settingsDialogRef.value?.focus({ preventScroll: true })
      })
    }
    else if (!v) {
      restoreFocus()
    }
  },
  { immediate: true },
)

function openGeneralAdvancedKeybindings(): void {
  selectPage('advanced')
}
</script>

<template>
  <component
    :is="embedded ? 'div' : Teleport"
    v-bind="embedded ? {} : { to: 'body' }"
  >
    <div
      v-if="visible"
      :class="embedded ? 'sv-embedded-root' : 'sv-backdrop'"
      :role="embedded ? undefined : 'dialog'"
      :aria-modal="embedded ? undefined : 'true'"
      :aria-label="embedded ? undefined : t('settings.ariaDialog')"
      @click.self="!embedded && emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="settingsDialogRef"
        :class="[
          embedded ? 'sv-embedded' : 'sv-dialog',
          !embedded && tab === 'voice' ? 'sv-dialog--voice' : '',
        ]"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
        @keydown="onDialogKeydown"
      >
        <header v-if="!embedded" class="sv-head">
          <h2 class="sv-title">
            {{ t("settings.title") }}
          </h2>
          <UiCloseButton class="sv-close" :label="t('settings.closeAria')" @click="emit('close')" />
        </header>

        <div class="sv-layout">
          <nav class="sv-nav" :aria-label="t('settings.ariaNav')">
            <span class="sv-nav-heading">{{ t('settings.navigationHeading') }}</span>
            <button v-for="item in navigation" :key="item.id" type="button" class="sv-nav-btn" :data-settings-page="item.id" :aria-current="selectedPage === item.id ? 'page' : undefined" @click="selectPage(item.id)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path :d="item.icon" /></svg>
              <span>{{ item.label }}</span>
            </button>
            <p class="sv-nav-note">
              {{ t('settings.navigationNote') }}
            </p>
          </nav>
          <section class="sv-content" aria-labelledby="settings-page-title">
            <header class="sv-page-head">
              <div>
                <h3 id="settings-page-title">
                  {{ pageTitle }}
                </h3><p>{{ pageDescription }}</p>
              </div>
              <span class="sv-local-badge">{{ t('settings.localPreferences') }}</span>
            </header>

            <SettingsGeneralTab
              v-if="visible && tab === 'general'"
              :general-sub-tab="generalSubTab"
              :page="generalPage"
              :visible="visible"
              :embedded="embedded"
              :basic-workspace="basicWorkspace"
            />

            <SettingsVoiceTab
              v-if="visible && tab === 'voice' && showVoiceSettingsTab"
              :bootstrap-epoch="pluginStore.bootstrapEpoch"
            />

            <SettingsPluginsTab
              v-if="visible && tab === 'plugins' && showPluginsSettingsTab"
              :bootstrap-epoch="pluginStore.bootstrapEpoch"
              :platform-only="!roleStore.interactionImmersive"
              @request-general-advanced="openGeneralAdvancedKeybindings"
            />

            <div v-if="visible && tab === 'storage'" class="sv-body">
              <ChatStorageSettingsPanel :initial-role-id="storageRoleId" :allow-memory-replay="!basicWorkspace" />
            </div>

            <SettingsAdultTab v-if="visible && tab === 'adult' && showAdultSettingsTab" />
          </section>
        </div>
      </div>
    </div>
  </component>
</template>

<style scoped>
.sv-embedded-root { flex:1; width:100%; min-height:0; display:flex; flex-direction:column; overflow:hidden; }
.sv-embedded { flex:1; min-height:0; min-width:0; display:flex; flex-direction:column; overflow:hidden; container-type:inline-size; container-name:oclive-settings; }
.sv-backdrop { position:fixed; inset:0; z-index:10040; display:flex; align-items:center; justify-content:center; padding:24px; background:#1c2d2866; backdrop-filter:blur(3px); }
.sv-dialog { position:relative; width:min(1040px,100%); height:min(800px,calc(100dvh - 48px)); min-height:0; max-height:calc(100dvh - 48px); overflow:hidden; border:1px solid color-mix(in srgb,var(--accent) 28%,var(--border-light)); border-radius:16px; background:var(--bg-primary); box-shadow:0 24px 72px #172a2440; display:flex; flex-direction:column; }
.sv-dialog:focus { outline:none; }
.sv-head { display:flex; justify-content:space-between; align-items:center; flex-shrink:0; padding:20px 26px; border-bottom:1px solid var(--border-light); background:var(--bg-elevated,var(--bg-primary)); }
.sv-title { margin:0; font-size:21px; font-weight:650; color:var(--text-primary); }
.sv-layout { flex:1; min-height:0; min-width:0; display:grid; grid-template-columns:194px minmax(0,1fr); overflow:hidden; }
.sv-nav { display:flex; flex-direction:column; gap:5px; padding:20px 12px; overflow:auto; border-right:1px solid var(--border-light); background:var(--bg-secondary); }
.sv-nav-heading { margin:0 12px 12px; font-size:11px; letter-spacing:.1em; color:var(--text-secondary); }
.sv-nav-btn { display:flex; align-items:center; gap:11px; padding:12px; text-align:left; font:inherit; font-size:13px; line-height:1.5; color:var(--text-secondary); border:1px solid transparent; border-radius:9px; background:transparent; cursor:pointer; }
.sv-nav-btn svg { width:18px; height:18px; flex-shrink:0; }
.sv-nav-btn:hover { background:color-mix(in srgb,var(--accent) 7%,transparent); color:var(--text-primary); }
.sv-nav-btn[aria-current=page] { color:var(--text-primary); font-weight:600; border-color:color-mix(in srgb,var(--accent) 24%,var(--border-light)); background:var(--bg-elevated,var(--bg-primary)); box-shadow:inset 3px 0 var(--accent); }
.sv-nav-note { margin:auto 12px 0; padding-top:24px; font-size:11px; line-height:1.8; color:var(--text-secondary); }
.sv-content { display:flex; flex-direction:column; min-height:0; min-width:0; overflow:hidden; }
.sv-page-head { display:flex; align-items:flex-start; gap:20px; justify-content:space-between; padding:24px 28px 20px; border-bottom:1px solid var(--border-light); flex-shrink:0; }
.sv-page-head h3 { margin:0; font-size:20px; font-weight:650; color:var(--text-primary); }
.sv-page-head p { margin:8px 0 0; font-size:12px; line-height:1.8; color:var(--text-secondary); max-width:52ch; }
.sv-local-badge { font-size:10px; border:1px solid var(--border-light); color:var(--text-secondary); border-radius:5px; padding:4px 7px; white-space:nowrap; margin-top:3px; }
.sv-content :deep(.sv-body) { flex:1; min-height:0; min-width:0; overflow:auto; padding:24px 28px 32px; display:flex; flex-direction:column; gap:18px; overscroll-behavior:contain; scrollbar-gutter:stable; }
.sv-content :deep(.ui-section) { flex-shrink:0; min-width:0; margin:0; padding:20px; border:1px solid var(--border-light); border-radius:12px; background:var(--bg-elevated,var(--bg-primary)); }
.sv-content :deep(.ui-section__head) { margin-bottom:16px; }
.sv-content :deep(.ui-section__title-row) { display:flex; align-items:center; justify-content:space-between; gap:12px; }
.sv-content :deep(.ui-section__title) { font-size:14px; font-weight:650; margin:0; }
.sv-content :deep(.ui-section__desc) { margin:7px 0 0; line-height:1.8; font-size:12px; color:var(--text-secondary); }
.sv-content :deep(.ui-section__body) { display:flex; flex-direction:column; gap:14px; }
.sv-content :deep(.ui-field-row) { display:flex; justify-content:space-between; min-height:40px; align-items:center; gap:18px; }
.sv-content :deep(.ui-field-row__label) { min-width:100px; color:var(--text-secondary); font-size:13px; }
.sv-content :deep(.ui-field-row__control) { min-width:0; display:flex; align-items:center; flex-wrap:wrap; justify-content:flex-end; gap:8px; }
.sv-content :deep(.ui-select) { min-height:36px; max-width:100%; padding:7px 30px 7px 12px; border:1px solid var(--border-light); border-radius:7px; background:var(--bg-primary); color:var(--text-primary); font:inherit; font-size:13px; }
.sv-content :deep(.ui-btn) { display:inline-flex; align-items:center; justify-content:center; min-height:34px; padding:7px 12px; border:1px solid var(--border-light); border-radius:7px; background:var(--bg-primary); color:var(--text-primary); font:inherit; font-size:12px; line-height:1.5; cursor:pointer; }
.sv-content :deep(.ui-btn--primary) { background:var(--accent); color:var(--accent-text,#fff); border-color:var(--accent); }
.sv-content :deep(.ui-btn:disabled) { opacity:.5; cursor:not-allowed; }
.sv-content :deep(.ui-btn:focus-visible),.sv-content :deep(.ui-select:focus-visible) { outline:2px solid var(--accent); outline-offset:2px; }
.sv-content :deep(.error-notice) { flex-shrink:0; }
.sv-nav-btn:focus-visible { outline:2px solid var(--accent); outline-offset:2px; }
.sv-nav,.sv-content :deep(.sv-body),.sv-content :deep(pre) { scrollbar-width:thin; scrollbar-color:var(--scrollbar-chat-thumb,#486b6240) transparent; }
.sv-nav::-webkit-scrollbar,.sv-content :deep(.sv-body::-webkit-scrollbar),.sv-content :deep(pre::-webkit-scrollbar) { width:10px; height:10px; }
.sv-nav::-webkit-scrollbar-thumb,.sv-content :deep(.sv-body::-webkit-scrollbar-thumb),.sv-content :deep(pre::-webkit-scrollbar-thumb) { background:var(--scrollbar-chat-thumb,#486b6240); border:3px solid transparent; background-clip:padding-box; border-radius:12px; }
.sv-content :deep(pre) { white-space:pre-wrap; overflow-wrap:anywhere; }
.sv-embedded .sv-layout { grid-template-columns:180px minmax(0,1fr); }
@container oclive-settings (max-width:600px) {
  .sv-layout { display:flex; flex-direction:column; }
  .sv-nav { flex-shrink:0; flex-direction:row; overflow:auto; padding:10px; gap:6px; border-right:0; border-bottom:1px solid var(--border-light); }
  .sv-nav-heading,.sv-nav-note { display:none; }
  .sv-nav-btn { flex-shrink:0; white-space:nowrap; padding:9px 11px; font-size:12px; }
  .sv-nav-btn svg { width:15px; height:15px; }
  .sv-nav-btn[aria-current=page] { box-shadow:inset 0 -2px var(--accent); }
  .sv-content { flex:1; }
  .sv-page-head { padding:17px 16px 14px; }
  .sv-page-head h3 { font-size:18px; }
  .sv-page-head p { max-width:none; font-size:11px; }
  .sv-local-badge { display:none; }
  .sv-content :deep(.sv-body) { padding:16px; gap:14px; }
  .sv-content :deep(.ui-section) { padding:15px; }
  .sv-content :deep(.ui-field-row) { flex-wrap:wrap; gap:8px; }
  .sv-content :deep(.ui-toggle) { gap:14px; }
}
@media(max-width:820px) { .sv-layout,.sv-embedded .sv-layout{grid-template-columns:166px minmax(0,1fr)}.sv-nav{padding:18px 9px}.sv-nav-btn{gap:8px;padding:10px;font-size:12px}.sv-page-head{padding:20px}.sv-content :deep(.sv-body){padding:20px}.sv-local-badge{display:none} }
@media(max-width:600px) { .sv-backdrop{padding:10px}.sv-dialog{height:calc(100dvh - 20px);max-height:calc(100dvh - 20px);border-radius:12px}.sv-head{padding:14px 16px}.sv-title{font-size:18px}.sv-layout,.sv-embedded .sv-layout{display:flex;flex-direction:column}.sv-nav{flex-shrink:0;flex-direction:row;padding:10px;gap:6px;border-right:0;border-bottom:1px solid var(--border-light);overflow:auto}.sv-nav-heading,.sv-nav-note{display:none}.sv-nav-btn{white-space:nowrap;flex-shrink:0;padding:9px 11px}.sv-nav-btn[aria-current=page]{box-shadow:inset 0 -2px var(--accent)}.sv-nav-btn svg{width:15px;height:15px}.sv-content{flex:1}.sv-page-head{padding:17px 16px 14px}.sv-page-head h3{font-size:18px}.sv-page-head p{font-size:11px;margin-top:5px}.sv-content :deep(.sv-body){padding:16px;gap:14px}.sv-content :deep(.ui-section){padding:15px}.sv-content :deep(.ui-field-row){flex-wrap:wrap;gap:8px}.sv-content :deep(.ui-field-row__label){min-width:80px} }
</style>

<style>
@import '@oclive/shared/styles/win98/panel-settings.css';
</style>
