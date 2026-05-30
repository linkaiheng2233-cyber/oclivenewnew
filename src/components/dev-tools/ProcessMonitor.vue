<script setup lang="ts">
import type { PluginProcessDebugInfo } from '../../api'
import { useI18n } from 'vue-i18n'

defineProps<{
  pluginId: string
  /** Whether manifest declares `process`; when false, this panel cannot spawn a child process */
  spawnSupported?: boolean
  processInfo: PluginProcessDebugInfo | null
  allProcesses: PluginProcessDebugInfo[]
  busy: boolean
}>()

const emit = defineEmits<{
  spawn: []
  kill: []
  restart: []
  refreshAll: []
  killManaged: [id: string]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="pm-dbg-proc">
    <div class="pm-dbg-proc-row">
      <span class="pm-dbg-label">{{ t("devTools.proc.thisPlugin") }}</span>
      <span v-if="processInfo" class="pm-dbg-pill ok">
        {{ t("devTools.proc.runningPid", { pid: processInfo.pid }) }}
      </span>
      <span v-else class="pm-dbg-pill">{{ t("devTools.proc.notStarted") }}</span>
    </div>
    <p v-if="processInfo" class="pm-dbg-mono">
      {{ processInfo.rpcUrl }}
    </p>
    <p v-if="spawnSupported === false" class="pm-dbg-warn">
      {{ t("devTools.proc.noSpawnHint") }}
    </p>
    <div class="pm-dbg-actions">
      <button
        type="button"
        class="pm-dbg-btn"
        :disabled="busy || spawnSupported === false"
        @click="emit('spawn')"
      >
        {{ t("devTools.proc.start") }}
      </button>
      <button type="button" class="pm-dbg-btn" :disabled="busy || !processInfo" @click="emit('kill')">
        {{ t("devTools.proc.stop") }}
      </button>
      <button type="button" class="pm-dbg-btn" :disabled="busy || !processInfo" @click="emit('restart')">
        {{ t("devTools.proc.restart") }}
      </button>
      <button type="button" class="pm-dbg-btn secondary" :disabled="busy" @click="emit('refreshAll')">
        {{ t("devTools.proc.refresh") }}
      </button>
    </div>
    <div v-if="allProcesses.length" class="pm-dbg-global">
      <div class="pm-dbg-sub">
        {{ t("devTools.proc.hostManaged") }}
      </div>
      <ul class="pm-dbg-plist">
        <li v-for="p in allProcesses" :key="p.pluginId" class="pm-dbg-pli">
          <span class="pm-dbg-mono">{{ p.pluginId }}</span>
          <span class="pm-dbg-muted">PID {{ p.pid }}</span>
          <button
            type="button"
            class="pm-dbg-btn danger"
            :disabled="busy"
            @click="emit('killManaged', p.pluginId)"
          >
            {{ t("devTools.proc.kill") }}
          </button>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.pm-dbg-warn {
  margin: 0;
  padding: 8px;
  border-radius: 8px;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--accent-soft) 35%, var(--bg-primary));
  border: 1px solid var(--border-light);
}
.pm-dbg-warn code {
  font-size: 10px;
}
.pm-dbg-proc {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 12px;
}
.pm-dbg-proc-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pm-dbg-label {
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-dbg-pill {
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-dbg-pill.ok {
  border-color: color-mix(in srgb, var(--success) 38%, var(--border-light));
  color: var(--success);
  background: color-mix(in srgb, var(--success) 8%, var(--bg-elevated));
}
.pm-dbg-mono {
  margin: 0;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
  color: var(--text-secondary);
}
.pm-dbg-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.pm-dbg-btn {
  padding: 5px 10px;
  border-radius: var(--radius-btn);
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
.pm-dbg-btn.danger {
  border-color: color-mix(in srgb, var(--error) 35%, var(--border-light));
  color: var(--error);
}
.pm-dbg-global {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--border-light);
}
.pm-dbg-sub {
  font-weight: 600;
  margin-bottom: 6px;
}
.pm-dbg-plist {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm-dbg-pli {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pm-dbg-muted {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
