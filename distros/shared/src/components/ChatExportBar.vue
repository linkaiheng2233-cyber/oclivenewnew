<script setup lang="ts">
import { exportChatLogs, writeUserTextFile } from '@oclive/shared/api'
import { downloadTextFile } from '@oclive/shared/utils/download'
import { save } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{ roleId: string }>()
const emit = defineEmits<{
  notify: [{ type: 'success' | 'error' | 'info', message: string }]
}>()

const { t } = useI18n()
const exportAllRoles = ref(false)
const includePluginDebug = ref(false)
const busy = ref(false)

async function runExport(format: 'json' | 'txt') {
  busy.value = true
  try {
    const res = await exportChatLogs({
      roleId: exportAllRoles.value ? undefined : props.roleId,
      allRoles: exportAllRoles.value,
      format,
      includePluginResolutionDebug:
        includePluginDebug.value && !exportAllRoles.value,
    })
    const filters
      = format === 'json'
        ? [{ name: 'JSON', extensions: ['json'] }]
        : [{ name: 'Text', extensions: ['txt'] }]

    let path: string | null = null
    try {
      path = await save({
        defaultPath: res.suggested_filename,
        filters,
      })
    }
    catch {
      const mime = format === 'json' ? 'application/json' : 'text/plain'
      downloadTextFile(res.suggested_filename, res.content, mime)
      emit('notify', {
        type: 'success',
        message: t('editor.chatExport.downloaded', { name: res.suggested_filename }),
      })
      return
    }

    if (path) {
      await writeUserTextFile(path, res.content)
      emit('notify', {
        type: 'success',
        message: t('editor.chatExport.success'),
      })
      return
    }

    emit('notify', { type: 'info', message: t('editor.chatExport.saveCancelled') })
  }
  catch (e) {
    emit('notify', {
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
    })
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="export-bar">
    <label class="chk">
      <input v-model="exportAllRoles" type="checkbox" :disabled="busy">
      {{ t("editor.chatExport.allRoles") }}
    </label>
    <label class="chk">
      <input
        v-model="includePluginDebug"
        type="checkbox"
        :disabled="busy || exportAllRoles"
      >
      {{ t("editor.chatExport.pluginDebug") }}
    </label>
    <button
      type="button"
      class="btn"
      :disabled="busy"
      @click="runExport('json')"
    >
      {{ t("editor.chatExport.exportJson") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="busy"
      @click="runExport('txt')"
    >
      {{ t("editor.chatExport.exportTxt") }}
    </button>
  </div>
</template>

<style scoped>
.export-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
  font-size: 13px;
  color: var(--text-secondary);
}
.chk {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
}
.btn {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-pill);
  padding: 6px 12px;
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.btn:hover {
  border-color: var(--accent);
}
.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
