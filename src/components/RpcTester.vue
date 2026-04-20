<script setup lang="ts">
import { computed } from "vue";
import type { RpcHistoryItem } from "../composables/usePluginDebug";

const props = defineProps<{
  methods: string[];
  busy: boolean;
  history: RpcHistoryItem[];
  method: string;
  params: string;
  /** 避免同页多插件 datalist id 冲突 */
  datalistId: string;
}>();

const emit = defineEmits<{
  "update:method": [value: string];
  "update:params": [value: string];
  discover: [];
  send: [];
  applyHistory: [item: RpcHistoryItem];
}>();

const methodOptions = computed(() => {
  const m = new Set(props.methods);
  if (props.method.trim()) {
    m.add(props.method.trim());
  }
  return [...m].filter(Boolean).sort();
});

function onFormat() {
  try {
    const v = props.params.trim() ? JSON.parse(props.params) : {};
    emit("update:params", `${JSON.stringify(v, null, 2)}\n`);
  } catch {
    /* keep */
  }
}
</script>

<template>
  <div class="pm-dbg-rpc">
    <div class="pm-dbg-rpc-row">
      <label class="pm-dbg-lab">方法</label>
      <input
        :value="method"
        class="pm-dbg-input"
        :list="datalistId"
        placeholder="resolve / health / rpc.discover …"
        autocomplete="off"
        @input="emit('update:method', ($event.target as HTMLInputElement).value)"
      />
      <datalist :id="datalistId">
        <option v-for="m in methodOptions" :key="m" :value="m" />
      </datalist>
      <button type="button" class="pm-dbg-btn secondary" :disabled="busy" @click="emit('discover')">
        发现方法
      </button>
    </div>
    <label class="pm-dbg-lab">参数 JSON</label>
    <textarea
      :value="params"
      class="pm-dbg-ta"
      rows="8"
      spellcheck="false"
      @input="emit('update:params', ($event.target as HTMLTextAreaElement).value)"
    />
    <div class="pm-dbg-actions">
      <button type="button" class="pm-dbg-btn" :disabled="busy" @click="emit('send')">发送</button>
      <button type="button" class="pm-dbg-btn secondary" @click="onFormat">格式化</button>
    </div>
    <div v-if="history.length" class="pm-dbg-hist">
      <div class="pm-dbg-sub">请求历史（点击回填）</div>
      <ul class="pm-dbg-hlist">
        <li v-for="h in history" :key="h.id">
          <button type="button" class="pm-dbg-link" @click="emit('applyHistory', h)">
            {{ h.method }}
            <span class="pm-dbg-muted">{{ new Date(h.at).toLocaleTimeString() }}</span>
          </button>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.pm-dbg-rpc {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 12px;
}
.pm-dbg-rpc-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm-dbg-lab {
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-dbg-input {
  flex: 1;
  min-width: 140px;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
}
.pm-dbg-ta {
  width: 100%;
  box-sizing: border-box;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  resize: vertical;
  min-height: 120px;
}
.pm-dbg-actions {
  display: flex;
  gap: 6px;
}
.pm-dbg-btn {
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  cursor: pointer;
}
.pm-dbg-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.pm-dbg-btn.secondary {
  background: transparent;
}
.pm-dbg-hist {
  margin-top: 6px;
  padding-top: 8px;
  border-top: 1px dashed var(--border-light);
}
.pm-dbg-sub {
  font-weight: 600;
  margin-bottom: 4px;
}
.pm-dbg-hlist {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.pm-dbg-link {
  display: block;
  width: 100%;
  text-align: left;
  padding: 4px 6px;
  border-radius: 4px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
}
.pm-dbg-link:hover {
  border-color: var(--border-light);
  background: var(--bg-elevated);
}
.pm-dbg-muted {
  margin-left: 6px;
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
