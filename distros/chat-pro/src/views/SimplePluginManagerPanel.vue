<script setup lang="ts">
import SimplePluginManager from '@oclive/shared/components/plugin-manager/SimplePluginManager.vue'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { ref, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import MarketView from './MarketView.vue'

export type PluginsPanelSubview = 'list' | 'market'

const props = withDefaults(
  defineProps<{
    visible: boolean
    embedded?: boolean
    subview?: PluginsPanelSubview
  }>(),
  { embedded: false, subview: 'list' },
)

const emit = defineEmits<{
  'close': []
  'openMarket': []
  'update:subview': [subview: PluginsPanelSubview]
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'visible'), dialogRef)

function selectSubview(next: PluginsPanelSubview): void {
  emit('update:subview', next)
}
</script>

<template>
  <component :is="embedded ? 'div' : 'Teleport'" v-bind="embedded ? {} : { to: 'body' }">
    <div
      v-if="visible"
      :class="embedded ? 'spm-embedded' : 'spm-backdrop'"
      :role="embedded ? undefined : 'dialog'"
      :aria-modal="embedded ? undefined : 'true'"
      :aria-label="embedded ? undefined : t('simplePluginManager.panelAria')"
      @click.self="!embedded && emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="dialogRef"
        :class="embedded ? 'spm-embedded-inner' : 'spm-dialog'"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <h2 v-if="!embedded" class="spm-heading">
          {{ t("simplePluginManager.title") }}
        </h2>
        <nav
          v-if="embedded"
          class="spm-subnav tool-mgmt-subtabs"
          role="tablist"
          :aria-label="t('simplePluginManager.embeddedNavAria')"
        >
          <button
            type="button"
            role="tab"
            class="tool-mgmt-subtabs__btn"
            :class="{ 'tool-mgmt-subtabs__btn--active': subview === 'list' }"
            :aria-selected="subview === 'list'"
            @click="selectSubview('list')"
          >
            {{ t("simplePluginManager.tabInstalled") }}
          </button>
          <button
            type="button"
            role="tab"
            class="tool-mgmt-subtabs__btn"
            :class="{ 'tool-mgmt-subtabs__btn--active': subview === 'market' }"
            :aria-selected="subview === 'market'"
            @click="selectSubview('market')"
          >
            {{ t("simplePluginManager.tabMarket") }}
          </button>
        </nav>
        <SimplePluginManager
          v-if="!embedded || subview === 'list'"
          :visible="visible"
          :embedded="embedded"
          @close="emit('close')"
          @open-market="embedded ? selectSubview('market') : emit('openMarket')"
        />
        <MarketView
          v-else-if="embedded && subview === 'market'"
          embedded
          :visible="visible"
          @back="selectSubview('list')"
        />
      </div>
    </div>
  </component>
</template>

<style scoped>
.spm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10055;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.spm-dialog {
  width: min(720px, 100%);
  max-height: min(88vh, 720px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 14px 16px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.spm-embedded {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.spm-embedded-inner {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: visible;
  background: var(--tool-chrome-editor, var(--bg-primary));
}
.spm-heading {
  margin: 0 0 10px;
  font-size: 1.1rem;
  font-weight: 600;
}
</style>

<style>
@import '@oclive/shared/styles/win98/panel-plugins.css';
</style>
