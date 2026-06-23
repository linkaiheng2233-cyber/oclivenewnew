<template>
  <div class="shell-vue">
    <h1 class="title">整壳示例（Vue）</h1>
    <p class="hint">由 <code>shell.vueEntry</code> 加载；失败时回退 <code>shell.entry</code> HTML。</p>
    <p v-if="roleLine" class="role">{{ roleLine }}</p>
    <p v-if="err" class="err">{{ err }}</p>
    <button type="button" class="btn" @click="refresh">拉取当前角色</button>
  </div>
</template>

<script setup lang="ts">
import { inject, ref } from "vue";

const oclive = inject<{ invoke: (c: string, p?: unknown) => Promise<unknown> }>("oclive");
const roleLine = ref("");
const err = ref("");

async function refresh() {
  err.value = "";
  roleLine.value = "";
  if (!oclive) {
    err.value = "未注入 oclive";
    return;
  }
  try {
    const r = (await oclive.invoke("get_current_role", {})) as { role_id?: string };
    roleLine.value = `get_current_role → role_id: ${r?.role_id ?? JSON.stringify(r)}`;
  } catch (e) {
    err.value = e instanceof Error ? e.message : String(e);
  }
}

void refresh();
</script>

<style scoped>
.shell-vue {
  padding: 24px;
  font-family: system-ui, sans-serif;
  max-width: 640px;
}
.title {
  margin: 0 0 8px;
  font-size: 1.35rem;
}
.hint {
  margin: 0 0 16px;
  color: #555;
  font-size: 0.9rem;
}
.role {
  margin: 0 0 12px;
  word-break: break-all;
}
.err {
  color: #b00020;
  margin: 0 0 12px;
}
.btn {
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid #ccc;
  background: #fff;
  cursor: pointer;
}
code {
  font-size: 0.85em;
}
</style>
