<script setup lang="ts">
import { computed, ref, watch } from "vue";

interface EnvEntry {
  key: string;
  value: string;
}

const STORAGE_KEY = "oclive.env.overrides";
const rows = ref<EnvEntry[]>([]);
const draftKey = ref("OCLIVE_WEATHER_API_KEY");
const draftValue = ref("");
const copied = ref("");

function load() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const arr = raw ? (JSON.parse(raw) as EnvEntry[]) : [];
    rows.value = Array.isArray(arr) ? arr : [];
  } catch {
    rows.value = [];
  }
}

watch(
  rows,
  () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(rows.value));
  },
  { deep: true },
);

function upsert() {
  const k = draftKey.value.trim();
  if (!k) return;
  const idx = rows.value.findIndex((x) => x.key === k);
  if (idx >= 0) {
    rows.value[idx].value = draftValue.value;
  } else {
    rows.value.push({ key: k, value: draftValue.value });
  }
  draftValue.value = "";
}

function removeKey(k: string) {
  rows.value = rows.value.filter((x) => x.key !== k);
}

const cmdText = computed(() =>
  rows.value
    .map((x) => `$env:${x.key}="${x.value.replace(/"/g, '\\"')}"`)
    .join("; "),
);

async function copyCmd() {
  if (!cmdText.value) return;
  await navigator.clipboard.writeText(cmdText.value);
  copied.value = "已复制";
  window.setTimeout(() => (copied.value = ""), 1500);
}

load();
</script>

<template>
  <section class="evm">
    <h4 class="evm-h">环境变量管理（会话草稿）</h4>
    <div class="evm-row">
      <input v-model="draftKey" class="evm-input" placeholder="OCLIVE_*" />
      <input v-model="draftValue" class="evm-input" placeholder="value" />
      <button type="button" class="evm-btn" @click="upsert">添加/更新</button>
    </div>
    <ul class="evm-list">
      <li v-for="r in rows" :key="r.key">
        <code>{{ r.key }}</code>=<code>{{ r.value }}</code>
        <button type="button" class="evm-link" @click="removeKey(r.key)">删除</button>
      </li>
    </ul>
    <div class="evm-row">
      <button type="button" class="evm-btn" @click="copyCmd">复制为终端命令</button>
      <span class="evm-copied">{{ copied }}</span>
    </div>
    <pre v-if="cmdText" class="evm-pre">{{ cmdText }}</pre>
  </section>
</template>

<style scoped>
.evm { margin-top: 12px; padding-top: 10px; border-top: 1px dashed var(--border-light); }
.evm-h { margin: 0 0 8px; font-size: 13px; }
.evm-row { display:flex; gap:8px; flex-wrap:wrap; align-items:center; margin-bottom:8px; }
.evm-input { flex:1; min-width: 180px; padding:6px 8px; border:1px solid var(--border-light); border-radius:8px; background:var(--bg-elevated); }
.evm-btn { border:1px solid var(--border-light); border-radius:8px; padding:6px 10px; background:var(--bg-elevated); }
.evm-list { margin: 0 0 8px; padding-left: 16px; font-size: 12px; }
.evm-link { margin-left: 8px; border: none; background: none; color: var(--accent); cursor: pointer; }
.evm-pre { margin: 0; padding: 8px; border:1px solid var(--border-light); border-radius:8px; background:var(--panel-bg-soft); font-size:12px; }
.evm-copied { font-size: 12px; color: var(--text-secondary); }
</style>
