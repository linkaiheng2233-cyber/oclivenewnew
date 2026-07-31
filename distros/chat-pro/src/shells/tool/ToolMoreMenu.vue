<script setup lang="ts">
import type { JumpTimeResponse } from '@oclive/shared/api'
import VirtualTimeBar from '@oclive/shared/components/scene/VirtualTimeBar.vue'
import HelpHint from '@oclive/shared/components/shared/HelpHint.vue'
import UiButton from '@oclive/shared/components/ui/UiButton.vue'
import UiListRow from '@oclive/shared/components/ui/UiListRow.vue'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useSceneDestination } from '@oclive/shared/composables/useSceneDestination'
import { useDebugStore } from '@oclive/shared/stores/debugStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

defineProps<{
  allSceneOptions: Array<{ id: string, label: string }>
}>()

const emit = defineEmits<{
  openShortcutHelp: []
  sceneChange: [ev: Event]
  notify: [payload: { type: 'success' | 'error' | 'info', message: string }]
  virtualTimeJumpComplete: [res: JumpTimeResponse]
}>()

const open = defineModel<boolean>({ required: true })

const { t } = useI18n()
const { showToast } = useAppToast()
const { characterSceneLabel } = useSceneDestination(showToast)
const roleStore = useRoleStore()
const debugStore = useDebugStore()
const uiStore = useUiStore()

const menuRef = ref<HTMLElement | null>(null)
let clickListenTimer: ReturnType<typeof setTimeout> | null = null

function onDocumentClickClose(e: MouseEvent): void {
  if (!open.value)
    return
  const el = menuRef.value
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
  <div ref="menuRef" class="tool-more-menu">
    <UiButton variant="ghost" @click.stop="open = !open">
      {{ open ? t("app.more.collapse") : t("app.more.more") }}
    </UiButton>

    <div
      v-show="open"
      class="tool-more-menu__panel"
      role="region"
      :aria-label="t('toolShell.moreMenu')"
      @click.stop
    >
      <section class="tool-more-menu__section">
        <UiButton variant="secondary" @click="emit('openShortcutHelp'); open = false">
          {{ t("app.more.shortcutHelp") }}
        </UiButton>
        <UiButton
          v-if="roleStore.interactionImmersive"
          variant="secondary"
          @click="debugStore.toggle(); open = false"
        >
          {{ t("app.more.openDebugPanel") }}
        </UiButton>
      </section>

      <template v-if="roleStore.interactionImmersive">
        <section class="tool-more-menu__section">
          <div class="tool-more-menu__section-head">
            <span>{{ t("app.more.virtualTime") }}</span>
            <HelpHint
              :paragraphs="[t('app.more.virtualTimeHint1'), t('app.more.virtualTimeHint2')]"
            />
          </div>
          <VirtualTimeBar
            compact
            :role-id="roleStore.currentRoleId"
            @notify="(p) => emit('notify', p)"
            @refreshed="roleStore.refreshRoleInfo"
            @jump-complete="(res) => emit('virtualTimeJumpComplete', res)"
          />
        </section>

        <section v-if="allSceneOptions.length > 0" class="tool-more-menu__section">
          <UiListRow :label="t('app.more.narrativeScene')">
            <template #control>
              <select
                class="tool-more-menu__select"
                :value="uiStore.sceneId"
                @change="emit('sceneChange', $event)"
              >
                <option v-for="s in allSceneOptions" :key="s.id" :value="s.id">
                  {{ s.label }}
                </option>
              </select>
            </template>
          </UiListRow>
          <p class="tool-more-menu__hint">
            {{ t('app.more.characterAt', { label: characterSceneLabel() }) }}
          </p>
        </section>
      </template>
    </div>
  </div>
</template>

<style scoped>
.tool-more-menu {
  position: relative;
}

.tool-more-menu__panel {
  position: absolute;
  top: calc(100% + var(--tool-space-1, 4px));
  right: 0;
  z-index: 30;
  width: min(320px, calc(100vw - 96px));
  max-height: min(70vh, 560px);
  overflow: auto;
  padding: var(--tool-space-3, 12px);
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius-lg, 6px);
  background: var(--tool-elevated, var(--bg-primary));
  box-shadow: none;
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-3, 12px);
}

.tool-more-menu__section {
  display: flex;
  flex-direction: column;
  gap: var(--tool-space-2, 8px);
}

.tool-more-menu__section + .tool-more-menu__section {
  padding-top: var(--tool-space-2, 8px);
  border-top: 1px solid var(--tool-border, var(--border-light));
}

.tool-more-menu__section-head {
  display: flex;
  align-items: center;
  gap: var(--tool-space-2, 8px);
  font-size: var(--tool-fs-sm, 12px);
  font-weight: 600;
  color: var(--tool-text-muted, var(--text-secondary));
}

.tool-more-menu__select {
  min-width: 8rem;
  max-width: 100%;
  padding: 4px 8px;
  border: 1px solid var(--tool-border, var(--border-light));
  border-radius: var(--tool-radius, 4px);
  font-size: var(--tool-fs-md, 13px);
  background: var(--tool-elevated, var(--bg-elevated));
  color: var(--tool-text, var(--text-primary));
}

.tool-more-menu__hint {
  margin: 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
}
</style>
