<script setup lang="ts">
import type { CastTier } from '../../composables/theater/theaterCastConfig'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import UiButton from '../../components/ui/UiButton.vue'

defineProps<{
  castLabel: string
  castTier?: CastTier
  sceneLabelKey: string
}>()

const emit = defineEmits<{
  openSettings: []
  restart: []
}>()

const { t } = useI18n()
const moreOpen = ref(false)

function onMore(action: 'settings' | 'restart') {
  moreOpen.value = false
  if (action === 'settings')
    emit('openSettings')
  else
    emit('restart')
}
</script>

<template>
  <header class="theater-header">
    <div class="theater-header__titles">
      <h1 class="theater-header__scene">
        {{ t(sceneLabelKey) }}
      </h1>
      <p v-if="castLabel" class="theater-header__cast">
        {{ castLabel }}
        <span v-if="castTier === 'applied'" class="theater-header__cast-badge">
          {{ t('theater.cast.tierAppliedBadge') }}
        </span>
      </p>
    </div>

    <div class="theater-header__more">
      <UiButton variant="ghost" size="sm" @click="moreOpen = !moreOpen">
        {{ moreOpen ? t('app.more.collapse') : t('app.more.more') }}
      </UiButton>
      <div v-show="moreOpen" class="theater-header__menu" role="menu" @click.stop>
        <button type="button" class="theater-header__menu-item" role="menuitem" @click="onMore('settings')">
          {{ t('theater.header.settings') }}
        </button>
        <button type="button" class="theater-header__menu-item" role="menuitem" @click="onMore('restart')">
          {{ t('theater.header.restart') }}
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.theater-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--tool-space-3, 12px);
  flex-shrink: 0;
  min-height: var(--tool-topbar-h, 36px);
  padding: 0 var(--tool-space-4, 16px);
  border-bottom: 1px solid var(--tool-divider, var(--border-light));
  background: var(--tool-chrome-editor, var(--bg-primary));
}

.theater-header__titles {
  min-width: 0;
}

.theater-header__scene {
  margin: 0;
  font-size: var(--tool-fs-md, 13px);
  font-weight: 600;
  line-height: var(--tool-line, 1.5);
}

.theater-header__cast {
  margin: 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.theater-header__cast-badge {
  display: inline-block;
  padding: 0 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--tool-accent, #6b8cff) 12%, transparent);
  border: 1px solid var(--border-light);
}

.theater-header__more {
  position: relative;
  flex-shrink: 0;
}

.theater-header__menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  z-index: 20;
  min-width: 140px;
  padding: var(--tool-space-1, 4px);
  border: 1px solid var(--tool-divider, var(--border-light));
  border-radius: var(--tool-radius, 8px);
  background: var(--tool-elevated, var(--bg-elevated));
}

.theater-header__menu-item {
  display: block;
  width: 100%;
  padding: var(--tool-space-2, 8px) var(--tool-space-3, 12px);
  border: none;
  border-radius: var(--tool-radius, 8px);
  background: transparent;
  color: var(--text-primary);
  font-size: var(--tool-fs-md, 13px);
  text-align: left;
  cursor: pointer;
}

.theater-header__menu-item:hover {
  background: color-mix(in srgb, var(--tool-accent) 10%, transparent);
}
</style>
