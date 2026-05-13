<script setup lang="ts">
import { PluginManagerV2 } from "../components/PluginManagerV2";

withDefaults(
  defineProps<{
    visible: boolean;
    /** 嵌入设置右栏：无 Teleport、无全屏遮罩 */
    embedded?: boolean;
  }>(),
  { embedded: false },
);

const emit = defineEmits<{
  close: [];
  openV1: [];
  openV1Backends: [];
}>();
</script>

<template>
  <div v-if="embedded && visible" class="pm2-embed-host">
    <PluginManagerV2
      :visible="visible"
      embedded
      @close="emit('close')"
      @open-v1="emit('openV1')"
      @open-v1-backends="emit('openV1Backends')"
    />
  </div>
  <Teleport v-else-if="visible" to="body">
    <div
      class="pm2-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件与后端管理 V2"
      @click.self="emit('close')"
    >
      <div class="pm2-dialog" @click.stop>
        <PluginManagerV2
          :visible="visible"
          @close="emit('close')"
          @open-v1="emit('openV1')"
          @open-v1-backends="emit('openV1Backends')"
        />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pm2-embed-host {
  width: 100%;
  min-width: 0;
  min-height: 200px;
}
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
  overflow-x: hidden;
  overflow-y: auto;
  padding: 14px 16px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
</style>
