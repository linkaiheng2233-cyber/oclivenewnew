<script setup lang="ts">
import PluginBackendSessionPanel from "../components/PluginBackendSessionPanel.vue";

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
        <header class="pm2-head">
          <h2 class="pm2-title">插件与后端管理 V2（预览）</h2>
          <p class="pm2-sub">
            当前为可读性优先版，适合快速切换后端与查看会话覆盖。
          </p>
        </header>
        <section class="pm2-body">
          <PluginBackendSessionPanel />
        </section>
        <footer class="pm2-foot">
          <button type="button" class="pm2-btn secondary" @click="emit('openV1')">
            打开专业模式（V1）
          </button>
          <button type="button" class="pm2-btn primary" @click="emit('close')">关闭</button>
        </footer>
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
  width: min(860px, 100%);
  max-height: min(90vh, 840px);
  overflow: auto;
  padding: 16px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm2-head {
  margin-bottom: 10px;
}
.pm2-title {
  margin: 0 0 6px;
  font-size: 18px;
}
.pm2-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-body {
  padding-top: 4px;
}
.pm2-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--border-light);
}
.pm2-btn {
  padding: 8px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  font-size: 13px;
  cursor: pointer;
}
.pm2-btn.secondary {
  background: transparent;
}
.pm2-btn.primary {
  background: var(--accent, #3b82f6);
  color: #fff;
  border-color: transparent;
}
</style>
