<script setup lang="ts">
withDefaults(
  defineProps<{
    title?: string;
    message: string;
    /** 原始堆栈等，展示在「查看详情」 */
    detail?: string;
    showRetry?: boolean;
    showFallback?: boolean;
    retryLabel?: string;
    fallbackLabel?: string;
  }>(),
  {
    title: "",
    detail: "",
    showRetry: true,
    showFallback: false,
    retryLabel: "加载失败，点击重试",
    fallbackLabel: "使用 HTML 版本",
  },
);

defineEmits<{
  retry: [];
  fallback: [];
}>();
</script>

<template>
  <div class="pep" role="alert">
    <h4 v-if="title" class="pep-title">{{ title }}</h4>
    <p class="pep-msg">{{ message }}</p>
    <div class="pep-actions">
      <button
        v-if="showRetry"
        type="button"
        class="pep-btn"
        @click="$emit('retry')"
      >
        {{ retryLabel }}
      </button>
      <button
        v-if="showFallback"
        type="button"
        class="pep-btn pep-btn--secondary"
        @click="$emit('fallback')"
      >
        {{ fallbackLabel }}
      </button>
    </div>
    <details v-if="detail" class="pep-details">
      <summary>查看详情</summary>
      <pre>{{ detail }}</pre>
    </details>
  </div>
</template>

<style scoped>
.pep {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  max-width: 100%;
}
.pep-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}
.pep-msg {
  margin: 0;
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-secondary);
}
.pep-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.pep-btn {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
}
.pep-btn--secondary {
  background: transparent;
}
.pep-details {
  font-size: 11px;
  width: 100%;
  color: var(--text-secondary);
}
.pep-details pre {
  margin: 6px 0 0;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 160px;
  overflow: auto;
  font-size: 10px;
}
</style>
