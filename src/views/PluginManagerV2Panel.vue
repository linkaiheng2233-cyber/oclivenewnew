<script setup lang="ts">
import { PluginManagerV2 } from "../components/PluginManagerV2";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openV1: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="pm2-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件与后端管理 V2"
      @click.self="emit('close')"
    >
      <div class="pm2-dialog" @click.stop>
        <PluginManagerV2 :visible="visible" @close="emit('close')" @open-v1="emit('openV1')" />
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
  max-height: min(92vh, 920px);
  overflow: auto;
  padding: 14px 16px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
</style>
