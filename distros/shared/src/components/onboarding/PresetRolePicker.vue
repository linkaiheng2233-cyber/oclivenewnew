<script setup lang="ts">
import type { PresetRoleOption } from '@oclive/shared/utils/presetRolePicker'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { presetGalleryRoles } from '@oclive/shared/utils/presetRolePicker'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  visible: boolean
  roles: PresetRoleOption[]
  picking?: boolean
}>()

const emit = defineEmits<{
  pick: [roleId: string]
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)

const galleryRoles = computed(() => presetGalleryRoles(props.roles))

useModalFocusRestore(computed(() => props.visible), dialogRef)

function modeLabel(suggestion: string | null | undefined): string | null {
  if (suggestion === 'pure_chat')
    return t('onboarding.presetPicker.modePureChat')
  if (suggestion === 'immersive')
    return t('onboarding.presetPicker.modeImmersive')
  return null
}

function onPick(roleId: string): void {
  if (props.picking)
    return
  emit('pick', roleId)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="preset-picker-overlay"
      role="dialog"
      aria-modal="true"
      :aria-label="t('onboarding.presetPicker.title')"
    >
      <div
        ref="dialogRef"
        class="preset-picker-card"
        tabindex="-1"
      >
        <header class="preset-picker-card__header">
          <h2 class="preset-picker-card__title">
            {{ t('onboarding.presetPicker.title') }}
          </h2>
          <p class="preset-picker-card__lead">
            {{ t('onboarding.presetPicker.lead') }}
          </p>
        </header>

        <ul class="preset-picker-grid" role="list">
          <li
            v-for="role in galleryRoles"
            :key="role.id"
            class="preset-picker-grid__item"
          >
            <button
              type="button"
              class="preset-picker-role"
              :disabled="picking"
              @click="onPick(role.id)"
            >
              <span class="preset-picker-role__name">{{ role.name }}</span>
              <span
                v-if="modeLabel(role.interaction_mode_suggestion)"
                class="preset-picker-role__badge"
              >
                {{ modeLabel(role.interaction_mode_suggestion) }}
              </span>
              <span
                v-if="role.description"
                class="preset-picker-role__desc"
              >
                {{ role.description }}
              </span>
            </button>
          </li>
        </ul>

        <p class="preset-picker-card__hint">
          {{ t('onboarding.presetPicker.hint') }}
        </p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.preset-picker-overlay {
  position: fixed;
  inset: 0;
  z-index: 12000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 16px;
  background: color-mix(in srgb, var(--text-primary) 42%, transparent);
  backdrop-filter: blur(4px);
}
.preset-picker-card {
  width: min(720px, 100%);
  max-height: min(88vh, 720px);
  overflow: auto;
  padding: 24px 22px 20px;
  border-radius: 16px;
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-lg, 0 12px 40px rgba(0, 0, 0, 0.18));
}
.preset-picker-card__title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}
.preset-picker-card__lead {
  margin: 8px 0 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.preset-picker-grid {
  list-style: none;
  margin: 20px 0 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}
.preset-picker-role {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  width: 100%;
  padding: 14px 14px 12px;
  text-align: left;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  background: var(--bg-secondary);
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease, transform 140ms ease;
}
.preset-picker-role:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--accent-primary, #5b8def) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent-primary, #5b8def) 6%, var(--bg-secondary));
  transform: translateY(-1px);
}
.preset-picker-role:disabled {
  opacity: 0.65;
  cursor: default;
}
.preset-picker-role__name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}
.preset-picker-role__badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent-primary, #5b8def) 14%, transparent);
  color: var(--text-secondary);
}
.preset-picker-role__desc {
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.preset-picker-card__hint {
  margin: 16px 0 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
</style>

<style>
@import '@oclive/shared/styles/win98/dialogs-shared.css';
</style>
