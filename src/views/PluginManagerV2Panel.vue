<script setup lang="ts">
import { ref, toRef } from "vue";
import { useI18n } from "vue-i18n";
import { PluginManagerV2 } from "../components/PluginManagerV2";
import { useModalFocusRestore } from "../composables/useModalFocusRestore";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openV1: [];
  focusArchSlot: [slotKey: string];
}>();

const { t } = useI18n();

const dialogRef = ref<HTMLElement | null>(null);
useModalFocusRestore(toRef(props, "visible"), dialogRef);
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="pm2-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('pluginManager.v2PanelAria')"
      @click.self="emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div ref="dialogRef" class="pm2-dialog" tabindex="-1" @click.stop @keydown.escape.stop="emit('close')">
        <PluginManagerV2
          :visible="visible"
          @close="emit('close')"
          @open-v1="emit('openV1')"
          @focus-arch-slot="emit('focusArchSlot', $event)"
        />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pm2-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10060;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.pm2-dialog {
  width: min(1220px, 100%);
  min-height: min(620px, 88vh);
  max-height: min(92vh, 920px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 14px 16px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
@media (max-width: 1080px) {
  .pm2-dialog {
    overflow: auto;
  }
}
</style>
