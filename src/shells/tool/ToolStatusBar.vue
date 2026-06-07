<script setup lang="ts">
import { computed, inject } from 'vue'
import { useI18n } from 'vue-i18n'
import KernelStatusBar from '../../components/KernelStatusBar.vue'
import { MAIN_SHELL_KEY } from '../../composables/mainShellKey'
import { useUserIdentityState } from '../../composables/useUserIdentityState'
import { useRoleStore } from '../../stores/roleStore'
import { useUiStore } from '../../stores/uiStore'

const props = defineProps<{
  statusHeart: string
  sceneLabelForId: (id: string) => string
}>()

const { t } = useI18n()
const roleStore = useRoleStore()
const uiStore = useUiStore()
const shell = inject(MAIN_SHELL_KEY)
const { currentIdentityLabel, hasCatalog } = useUserIdentityState()

const sceneLabel = computed(() => {
  const id = uiStore.sceneId || 'default'
  return props.sceneLabelForId(id)
})

const favorabilityText = computed(() =>
  `${t('app.sidebar.favorability')} ${Math.round(roleStore.roleInfo.favorability)} ${props.statusHeart}`,
)

function onIdentityClick() {
  shell?.openSettingsToGeneral()
}
</script>

<template>
  <footer class="tool-status-bar" role="status">
    <KernelStatusBar class="tool-status-bar__kernel" />
    <span v-if="roleStore.interactionImmersive" class="tool-status-bar__sep" aria-hidden="true">·</span>
    <span v-if="roleStore.interactionImmersive" class="tool-status-bar__segment">
      {{ sceneLabel }}
    </span>
    <span class="tool-status-bar__sep" aria-hidden="true">·</span>
    <span class="tool-status-bar__segment">
      {{ favorabilityText }}
    </span>
    <template v-if="hasCatalog && currentIdentityLabel">
      <span class="tool-status-bar__sep" aria-hidden="true">·</span>
      <button
        type="button"
        class="tool-status-bar__identity"
        :title="t('settings.userIdentitySectionTitle')"
        @click="onIdentityClick"
      >
        {{ t('roleRuntime.currentIdentity', { name: currentIdentityLabel }) }}
      </button>
    </template>
  </footer>
</template>

<style scoped>
.tool-status-bar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--tool-space-2, 8px);
  min-height: var(--tool-statusbar-h, 24px);
  padding: 0 var(--tool-space-3, 12px);
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  background: var(--tool-chrome-status, var(--tool-bg, var(--bg-secondary)));
  border-top: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
}

.tool-status-bar__sep {
  opacity: 0.55;
}

.tool-status-bar__segment {
  white-space: nowrap;
}

.tool-status-bar__identity {
  padding: 0;
  border: none;
  background: none;
  font: inherit;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  cursor: pointer;
  white-space: nowrap;
  text-decoration: underline;
  text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
  text-underline-offset: 2px;
}

.tool-status-bar__identity:hover {
  color: var(--tool-text, var(--text-primary));
}

.tool-status-bar__kernel :deep(.kernel-status) {
  border-radius: var(--tool-radius, 4px);
  border-color: var(--tool-border, var(--border-light));
  background: transparent;
  font-size: var(--tool-fs-sm, 12px);
  padding: 0 var(--tool-space-2, 8px);
  min-height: 20px;
}
</style>
