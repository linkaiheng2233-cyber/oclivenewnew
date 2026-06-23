<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { HotkeyAction } from '@oclive/shared/api'
import { listen } from '@tauri-apps/api/event'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'

const { t } = useI18n()
const pluginStore = usePluginStore()

const launcherOpen = ref(false)
const hotkeyTarget = ref<{
  pluginId: string
  slot: string
  appearanceId: string
} | null>(null)

const activeSlot = computed(() => {
  const target = hotkeyTarget.value
  if (!target) {
    return null
  }
  return pluginStore.bootstrapUiSlots.find(
    s =>
      s.pluginId === target.pluginId
      && s.slot === target.slot
      && (s.appearanceId ?? '') === (target.appearanceId ?? ''),
  )
})

let unlisten: UnlistenFn | undefined

onMounted(async () => {
  unlisten = await listen<{ bindingId: string, action: HotkeyAction }>(
    'hotkey-action',
    (e) => {
      const a = e.payload.action
      if (a.type === 'openLauncherList') {
        launcherOpen.value = true
        return
      }
      if (a.type === 'openPluginSlot') {
        hotkeyTarget.value = {
          pluginId: a.pluginId,
          slot: a.slot,
          appearanceId: (a.appearanceId ?? '').trim(),
        }
      }
    },
  )
})

onBeforeUnmount(() => {
  unlisten?.()
})

function closeHotkeyPlugin(): void {
  hotkeyTarget.value = null
}

function closeLauncher(): void {
  launcherOpen.value = false
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="hotkeyTarget && activeSlot"
      class="hk-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('app.hotkeyHost.pluginDialogAria')"
      @click.self="closeHotkeyPlugin"
    >
      <div class="hk-dialog">
        <header class="hk-head">
          <span class="hk-title">{{ activeSlot.pluginId }} · {{ activeSlot.slot }}</span>
          <button type="button" class="hk-close" :aria-label="t('settings.closeAria')" @click="closeHotkeyPlugin">
            ×
          </button>
        </header>
        <iframe
          class="hk-frame"
          :src="activeSlot.url"
          :title="`plugin ${activeSlot.pluginId}`"
          referrerpolicy="no-referrer"
        />
      </div>
    </div>
    <div
      v-else-if="hotkeyTarget"
      class="hk-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('app.hotkeyHost.notFoundDialogAria')"
      @click.self="closeHotkeyPlugin"
    >
      <div class="hk-dialog hk-dialog--narrow">
        <header class="hk-head">
          <span class="hk-title">{{ t("app.hotkeyHost.cannotOpenTitle") }}</span>
          <button type="button" class="hk-close" :aria-label="t('settings.closeAria')" @click="closeHotkeyPlugin">
            ×
          </button>
        </header>
        <p class="hk-muted">
          {{
            t("app.hotkeyHost.notFoundBody", {
              plugin: hotkeyTarget.pluginId,
              slot: hotkeyTarget.slot,
            })
          }}
        </p>
      </div>
    </div>
    <div
      v-if="launcherOpen"
      class="hk-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('app.hotkeyHost.launcherDialogAria')"
      @click.self="closeLauncher"
    >
      <div class="hk-dialog hk-dialog--narrow">
        <header class="hk-head">
          <span class="hk-title">{{ t("app.hotkeyHost.launcherTitle") }}</span>
          <button type="button" class="hk-close" :aria-label="t('settings.closeAria')" @click="closeLauncher">
            ×
          </button>
        </header>
        <ul class="hk-launch-list">
          <li v-for="p in pluginStore.catalog" :key="p.id">
            <span class="hk-launch-id">{{ p.id }}</span>
            <span v-if="p.uiSlotNames?.length" class="hk-launch-slots">{{
              p.uiSlotNames.join(", ")
            }}</span>
          </li>
        </ul>
        <p v-if="!pluginStore.catalog.length" class="hk-muted">
          {{ t("app.hotkeyHost.noPlugins") }}
        </p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.hk-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10070;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.hk-dialog {
  width: min(720px, 100%);
  max-height: min(88vh, 640px);
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  overflow: hidden;
}
.hk-dialog--narrow {
  width: min(420px, 100%);
  max-height: min(80vh, 520px);
  padding: 0 0 12px;
}
.hk-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
}
.hk-title {
  font-size: 14px;
  font-weight: 600;
}
.hk-close {
  border: none;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.hk-frame {
  width: 100%;
  min-height: 360px;
  flex: 1;
  border: none;
  background: var(--bg-elevated);
}
.hk-launch-list {
  margin: 0;
  padding: 8px 14px;
  list-style: none;
  max-height: 420px;
  overflow: auto;
  font-size: 13px;
}
.hk-launch-list li {
  padding: 8px 0;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.hk-launch-id {
  font-weight: 600;
}
.hk-launch-slots {
  font-size: 12px;
  color: var(--text-secondary);
}
.hk-muted {
  margin: 8px 14px;
  font-size: 13px;
  color: var(--text-secondary);
}
</style>
