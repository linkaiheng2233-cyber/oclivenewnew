<script setup lang="ts">
import { inject, onMounted, ref } from "vue";

type Oclive = {
  invoke: (command: string, params?: unknown) => Promise<unknown>;
};

const oclive = inject<Oclive>("oclive");
const count = ref(0);

onMounted(async () => {
  try {
    const r = await oclive?.invoke("list_roles", {});
    if (Array.isArray(r)) {
      count.value = r.length;
    }
  } catch {
    /* ignore */
  }
});
</script>

<template>
  <button type="button" class="tb-btn" :title="`Vue 插槽 · 角色数 ${count}`">
    Vue 工具栏 · {{ count }} 个角色
  </button>
</template>

<style scoped>
.tb-btn {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: var(--radius-btn, 6px);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-accent);
  cursor: pointer;
  font-family: var(--font-ui);
}
</style>
