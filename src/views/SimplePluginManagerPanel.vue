<script setup lang="ts">
import { ref, toRef } from "vue";
import { useI18n } from "vue-i18n";
import SimplePluginManager from "./SimplePluginManager.vue";
import { useModalFocusRestore } from "../composables/useModalFocusRestore";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openMarket: [];
}>();

const { t } = useI18n();
const dialogRef = ref<HTMLElement | null>(null);
useModalFocusRestore(toRef(props, "visible"), dialogRef);
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="spm-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('simplePluginManager.panelAria')"
      @click.self="emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="dialogRef"
        class="spm-dialog"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <h2 class="spm-heading">{{ t("simplePluginManager.title") }}</h2>
        <SimplePluginManager
          :visible="visible"
          @close="emit('close')"
          @open-market="emit('openMarket')"
        />
      </div>
    </div>
  </Teleport>
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
.spm-heading {
  margin: 0 0 10px;
  font-size: 1.1rem;
  font-weight: 600;
}
</style>
