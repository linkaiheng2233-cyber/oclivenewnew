<script setup lang="ts">
import type { LocalePreference } from '../i18n'
import type { JumpTimeResponse } from '../api'
import type { RelationOptionRow } from '../utils/relationOptions'
import { useAppToast } from '../composables/useAppToast'
import { setLocalePreference } from '../i18n'
import { useSceneDestination } from '../composables/useSceneDestination'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import HelpHint from './shared/HelpHint.vue'
import RoleSelector from './role/RoleSelector.vue'
import VirtualTimeBar from './scene/VirtualTimeBar.vue'
import { useOcliveAppearance } from '../composables/useOcliveAppearance'
import { useChatStore } from '../stores/chatStore'
import { useDebugStore } from '../stores/debugStore'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'
const open = defineModel<boolean>({ required: true })
const localePreference = defineModel<LocalePreference>('localePreference', { required: true })

defineProps<{
  relationOptions: RelationOptionRow[]
  allSceneOptions: Array<{ id: string, label: string }>
  settingsEntryMoreHelp: string
  pluginManagerMoreBtnLabel: string
}>()

const emit = defineEmits<{
  openSettings: []
  openShortcutHelp: []
  openPluginManager: []
  openPluginMarket: []
  sceneChange: [ev: Event]
  interactionModeChange: [ev: Event]
  changeRole: [roleId: string]
  changeRelation: [relation: string]
  notify: [payload: { type: 'success' | 'error' | 'info', message: string }]
  virtualTimeJumpComplete: [res: JumpTimeResponse]
}>()

const { t } = useI18n()
const { showToast } = useAppToast()
const { characterSceneLabel } = useSceneDestination(showToast)
const roleStore = useRoleStore()
const chatStore = useChatStore()
const debugStore = useDebugStore()
const uiStore = useUiStore()
const { themePreference, themeCycleLabel, cycleTheme, bumpScale, scaleLabel }
  = useOcliveAppearance()

const panelRootRef = ref<HTMLElement | null>(null)
let clickListenTimer: ReturnType<typeof setTimeout> | null = null

function onLocalePreferenceChange(ev: Event): void {
  const v = (ev.target as HTMLSelectElement).value as LocalePreference
  setLocalePreference(v)
  localePreference.value = v
}

function toggleOpen(e: Event): void {
  e.stopPropagation()
  open.value = !open.value
}

function onDocumentClickClose(e: MouseEvent): void {
  if (!open.value)
    return
  const el = panelRootRef.value
  if (el && !el.contains(e.target as Node))
    open.value = false
}

watch(open, (isOpen) => {
  if (clickListenTimer != null) {
    clearTimeout(clickListenTimer)
    clickListenTimer = null
  }
  document.removeEventListener('click', onDocumentClickClose)
  if (isOpen) {
    void nextTick(() => {
      clickListenTimer = setTimeout(() => {
        clickListenTimer = null
        document.addEventListener('click', onDocumentClickClose)
      }, 0)
    })
  }
})

onBeforeUnmount(() => {
  if (clickListenTimer != null)
    clearTimeout(clickListenTimer)
  document.removeEventListener('click', onDocumentClickClose)
})
</script>

<template>
  <div ref="panelRootRef" class="top-bar-more-root">
    <div class="top-bar-row">
      <slot name="leading" />
      <button
        type="button"
        class="more-toggle"
        :aria-expanded="open"
        aria-controls="top-more-panel"
        @click="toggleOpen"
      >
        {{ open ? t("app.more.collapse") : t("app.more.more") }}
      </button>
    </div>

    <div
      v-show="open"
      id="top-more-panel"
      class="top-more-panel"
      role="region"
      :aria-label="t('app.more.ariaMoreFeatures')"
      @click.stop
    >
      <div class="more-grid">
        <div class="more-tile more-tile--xs">
          <div class="more-tile-head">
            <span class="more-label">{{ t("app.locale.label") }}</span>
          </div>
          <div class="more-tile-body">
            <select
              class="interaction-mode-select more-select more-select--fill"
              :value="localePreference"
              @change="onLocalePreferenceChange"
            >
              <option value="system">
                {{ t("app.locale.system") }}
              </option>
              <option value="zh-CN">
                {{ t("app.locale.zhCN") }}
              </option>
              <option value="en-US">
                {{ t("app.locale.enUS") }}
              </option>
            </select>
          </div>
        </div>

        <div class="more-tile more-tile--xs">
          <div class="more-tile-head">
            <span class="more-label">{{ t("app.more.interactionMode") }}</span>
            <HelpHint
              :paragraphs="[
                t('app.more.interactionImmersiveHint'),
                t('app.more.interactionPureChatHint'),
              ]"
            />
          </div>
          <div class="more-tile-body">
            <select
              id="interaction-mode"
              class="interaction-mode-select more-select more-select--fill"
              :value="roleStore.roleInfo.interactionMode"
              @change="emit('interactionModeChange', $event)"
            >
              <option value="immersive">
                {{ t("app.more.interactionImmersive") }}
              </option>
              <option value="pure_chat">
                {{ t("app.more.interactionPureChat") }}
              </option>
            </select>
          </div>
        </div>

        <div class="more-tile more-tile--sm">
          <div class="more-tile-head">
            <span class="more-label">{{ t("app.more.identity") }}</span>
            <HelpHint :text="t('app.more.identityHelp')" />
          </div>
          <div class="more-tile-body more-tile-body--selector">
            <RoleSelector
              variant="topbar"
              :sections="['relation']"
              :current-role-id="roleStore.currentRoleId"
              :current-relation="roleStore.relationSelectValue"
              :roles="roleStore.roles"
              :relations="relationOptions"
              :loading="chatStore.isLoading"
              @change-role="emit('changeRole', $event)"
              @change-relation="emit('changeRelation', $event)"
            />
          </div>
        </div>

        <div class="more-tile more-tile--lg">
          <div class="more-tile-head">
            <span class="more-label">{{ t("app.more.ui") }}</span>
            <HelpHint
              :paragraphs="[t('app.more.uiHint1'), t('app.more.uiHint2')]"
            />
          </div>
          <div class="more-tile-body">
            <div class="top-bar-appearance" role="toolbar" :aria-label="t('app.more.appearanceToolbar')">
              <div class="appearance-scale" :aria-label="t('app.more.scaleGroup')">
                <button
                  type="button"
                  class="appearance-icon-btn"
                  :title="t('app.more.shrinkTitle')"
                  :aria-label="t('app.more.shrinkAria')"
                  @click="bumpScale(-1)"
                >
                  A−
                </button>
                <span
                  class="appearance-scale-value"
                  :title="t('app.more.scaleRelativeTitle', { label: scaleLabel })"
                >{{ scaleLabel }}</span>
                <button
                  type="button"
                  class="appearance-icon-btn"
                  :title="t('app.more.enlargeTitle')"
                  :aria-label="t('app.more.enlargeAria')"
                  @click="bumpScale(1)"
                >
                  A+
                </button>
              </div>
              <button
                type="button"
                class="appearance-theme-btn"
                :title="t('app.more.themeTitle', { label: themeCycleLabel })"
                @click="cycleTheme"
              >
                {{
                  themePreference === "system"
                    ? "◐"
                    : themePreference === "dark"
                      ? "🌙"
                      : "☀️"
                }}
                {{ themeCycleLabel }}
              </button>
            </div>
          </div>
        </div>

        <div class="more-tile more-tile--action settings-entry-tile">
          <div class="more-tile-head">
            <span class="more-label">{{ t("app.more.settingsEntry") }}</span>
            <HelpHint :text="settingsEntryMoreHelp" />
          </div>
          <div class="more-tile-body settings-entry-actions" role="group" :aria-label="t('app.more.settingsEntry')">
            <button type="button" class="more-debug-btn more-debug-btn--fill settings-entry-btn" @click="emit('openShortcutHelp')">
              {{ t("app.more.shortcutHelp") }}
            </button>
            <button
              type="button"
              class="more-debug-btn more-debug-btn--fill settings-entry-btn settings-entry-btn--primary settings-gear-btn"
              @click="emit('openSettings')"
            >
              {{ t("app.more.openSettings") }}
            </button>
            <button
              type="button"
              class="more-debug-btn more-debug-btn--fill settings-entry-btn"
              @click="emit('openPluginManager')"
            >
              {{ pluginManagerMoreBtnLabel }}
            </button>
            <button
              type="button"
              class="more-debug-btn more-debug-btn--fill settings-entry-btn"
              @click="emit('openPluginMarket')"
            >
              {{ t("app.more.pluginMarket") }}
            </button>
          </div>
        </div>

        <div class="more-tile more-tile--action">
          <div class="more-tile-head">
            <span class="more-label">{{ t("app.more.debug") }}</span>
            <HelpHint :text="t('app.more.debugHelp')" />
          </div>
          <div class="more-tile-body">
            <button type="button" class="more-debug-btn more-debug-btn--fill" @click="debugStore.toggle">
              {{ t("app.more.openDebugPanel") }}
            </button>
          </div>
        </div>

        <template v-if="roleStore.interactionImmersive">
          <div class="more-tile more-tile--third">
            <div class="more-tile-head more-tile-head--tight">
              <span class="more-label">{{ t("app.more.virtualTime") }}</span>
              <HelpHint
                :paragraphs="[t('app.more.virtualTimeHint1'), t('app.more.virtualTimeHint2')]"
              />
            </div>
            <div class="more-tile-body more-tile-body--row">
              <VirtualTimeBar
                compact
                class="more-vtime"
                :role-id="roleStore.currentRoleId"
                @notify="(p) => emit('notify', p)"
                @refreshed="roleStore.refreshRoleInfo"
                @jump-complete="(res) => emit('virtualTimeJumpComplete', res)"
              />
            </div>
          </div>

          <div v-if="allSceneOptions.length > 0" class="more-tile more-tile--third">
            <div class="more-tile-head more-tile-head--tight">
              <span class="more-label">{{ t("app.more.narrativeScene") }}</span>
              <HelpHint :text="t('app.more.narrativeSceneHelp')" />
            </div>
            <div class="more-tile-body more-tile-body--scene more-tile-body--scene-inline">
              <select
                id="top-scene-select"
                class="scene-select more-select more-select--fill"
                :value="uiStore.sceneId"
                @change="emit('sceneChange', $event)"
              >
                <option v-for="s in allSceneOptions" :key="s.id" :value="s.id">
                  {{ s.label }}
                </option>
              </select>
              <span class="scene-row-hint scene-row-hint--tile">{{ t('app.more.characterAt', { label: characterSceneLabel() }) }}</span>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.top-bar-more-root {
  display: contents;
}
.top-bar-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.more-toggle {
  flex-shrink: 0;
  padding: 6px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: var(--control-transition);
}
.more-toggle:hover {
  border-color: color-mix(in srgb, var(--border-light) 70%, var(--text-secondary) 30%);
  color: var(--text-accent);
}
.more-toggle:focus {
  outline: none;
}
.more-toggle:focus-visible {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
.top-more-panel {
  margin-top: 10px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
}
.top-more-panel .interaction-mode-select,
.top-more-panel .scene-select {
  font-size: 13px;
  padding: 6px 10px;
  line-height: 1.4;
}
.top-more-panel .appearance-icon-btn,
.top-more-panel .appearance-theme-btn {
  font-size: 13px;
  min-height: 30px;
}
.top-more-panel .more-debug-btn {
  font-size: 13px;
  padding: 8px 12px;
}
.settings-entry-tile {
  min-width: min(24rem, 100%);
}
.settings-entry-actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
.settings-entry-btn {
  min-height: 34px;
  font-size: 12px;
  font-weight: 600;
}
.settings-entry-btn--primary {
  border-color: color-mix(in srgb, var(--accent) 48%, var(--border-light) 52%);
  color: var(--text-accent);
  background: color-mix(in srgb, var(--bg-elevated) 75%, var(--accent-soft) 25%);
}
.settings-gear-btn {
  justify-content: center;
}
@media (max-width: 680px) {
  .settings-entry-actions {
    grid-template-columns: 1fr;
  }
}
.more-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-start;
  align-items: flex-start;
  align-content: flex-start;
  gap: 12px 16px;
}
.more-tile {
  box-sizing: border-box;
  min-width: 0;
  padding: 12px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: var(--shadow-sm);
}
.more-tile--xs {
  flex: 0 0 auto;
  width: min(12rem, 100%);
}
.more-tile--sm {
  flex: 0 0 auto;
  width: min(17rem, 100%);
}
.more-tile--lg {
  flex: 0 0 auto;
  width: min(22rem, 100%);
}
.more-tile--action {
  flex: 0 0 auto;
  width: min(13rem, 100%);
}
.more-tile--third {
  flex: 0 0 calc((100% - 32px) / 3);
  width: calc((100% - 32px) / 3);
  max-width: calc((100% - 32px) / 3);
  min-width: 0;
  padding: 12px 14px;
  gap: 10px;
  box-sizing: border-box;
}
.more-tile-head--tight {
  justify-content: flex-start;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 8px;
}
.more-tile-head--tight .more-label {
  padding-top: 0;
}
@media (max-width: 560px) {
  .more-tile--xs,
  .more-tile--sm,
  .more-tile--lg,
  .more-tile--action {
    width: 100%;
  }
  .more-tile--third {
    flex: 1 1 100%;
    width: 100%;
    max-width: 100%;
  }
}
.more-tile-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}
.more-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  line-height: 1.45;
  padding-top: 2px;
}
.more-tile-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.more-tile-body--row {
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
}
.more-tile-body--scene {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
  gap: 8px 12px;
  align-items: center;
}
.more-tile-body--scene-inline {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 8px 12px;
}
.more-tile-body--scene-inline .more-select--fill,
.more-tile-body--scene-inline .scene-select {
  flex: 0 1 14rem;
  min-width: min(12rem, 100%);
  max-width: 100%;
}
@media (max-width: 520px) {
  .more-tile-body--scene {
    grid-template-columns: 1fr;
  }
}
.more-tile-body--selector :deep(.selector-row--topbar) {
  width: 100%;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.more-tile-body--selector :deep(.select) {
  min-width: 0;
  flex: 1 1 8rem;
  max-width: 100%;
}
.more-select--fill {
  width: 100%;
  max-width: none;
  box-sizing: border-box;
}
.more-vtime {
  flex: 1 1 12rem;
  min-width: 0;
  width: 100%;
}
.scene-row-hint--tile {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  min-width: min(12rem, 100%);
  flex: 1 1 12rem;
  max-width: 100%;
}
.more-tile--third :deep(.vtime--compact) {
  gap: 6px;
  flex-wrap: wrap;
}
.more-tile--third :deep(.vtime--compact .time-display) {
  max-width: 100%;
  padding: 5px 8px;
  font-size: 12px;
}
.more-tile--third :deep(.vtime--compact .label-icon) {
  font-size: 14px;
}
.more-debug-btn {
  padding: 8px 12px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 12px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: var(--control-transition);
}
.more-debug-btn--fill {
  width: 100%;
  box-sizing: border-box;
}
.more-debug-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-focus);
}
.top-bar-appearance {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  width: 100%;
}
.top-more-panel .top-bar-appearance {
  margin-left: 0;
}
.appearance-scale {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 88%, transparent);
  box-shadow: var(--shadow-sm), var(--frame-inset-highlight);
}
.appearance-scale-value {
  min-width: 2.6rem;
  text-align: center;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}
.appearance-icon-btn,
.appearance-theme-btn {
  padding: 4px 8px;
  min-height: 28px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  font-family: var(--font-ui);
  transition: var(--control-transition);
}
.appearance-icon-btn:hover,
.appearance-theme-btn:hover {
  border-color: var(--accent);
  color: var(--text-accent);
}
.appearance-icon-btn:focus,
.appearance-theme-btn:focus {
  outline: none;
}
.appearance-icon-btn:focus-visible,
.appearance-theme-btn:focus-visible {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
.appearance-theme-btn {
  white-space: nowrap;
}
.interaction-mode-select {
  min-width: 88px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--bg-elevated);
}
.interaction-mode-select:focus {
  outline: none;
}
.interaction-mode-select:focus-visible {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
</style>
