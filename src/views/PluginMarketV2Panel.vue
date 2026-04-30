<script setup lang="ts">
import PluginMarketV2Pane from "../components/PluginManagerV2/PluginMarketV2Pane.vue";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="pm2-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件市场 V2"
      @click.self="emit('close')"
    >
      <div class="pm2-dialog" @click.stop>
        <PluginMarketV2Pane />
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

