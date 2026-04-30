<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { save } from "@tauri-apps/api/dialog";
import { writeTextFile } from "@tauri-apps/api/fs";
import { exportChatLogs } from "../utils/tauri-api";
import { downloadTextFile } from "../utils/download";

const props = defineProps<{ roleId: string }>();
const emit = defineEmits<{
  notify: [{ type: "success" | "error" | "info"; message: string }];
}>();

const exportAllRoles = ref(false);
const includePluginDebug = ref(false);
const busy = ref(false);
const { t } = useI18n();

async function runExport(format: "json" | "txt") {
  busy.value = true;
  try {
    const res = await exportChatLogs({
      roleId: exportAllRoles.value ? undefined : props.roleId,
      allRoles: exportAllRoles.value,
      format,
      includePluginResolutionDebug:
        includePluginDebug.value && !exportAllRoles.value,
    });
    const filters =
      format === "json"
        ? [{ name: "JSON", extensions: ["json"] }]
        : [{ name: "Text", extensions: ["txt"] }];

    let path: string | null = null;
    try {
      path = await save({
        defaultPath: res.suggested_filename,
        filters,
      });
    } catch {
      const mime = format === "json" ? "application/json" : "text/plain";
      downloadTextFile(res.suggested_filename, res.content, mime);
      emit("notify", {
        type: "success",
        message: String(t("chatExport.toasts.downloaded", { name: res.suggested_filename })),
      });
      return;
    }

    if (path) {
      await writeTextFile(path, res.content);
      emit("notify", {
        type: "success",
        message: String(t("chatExport.toasts.exported")),
      });
      return;
    }

    emit("notify", { type: "info", message: String(t("chatExport.toasts.cancelled")) });
  } catch (e) {
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="export-bar">
    <label class="chk">
      <input v-model="exportAllRoles" type="checkbox" :disabled="busy" />
      {{ t("chatExport.exportAllRoles") }}
    </label>
    <label class="chk">
      <input
        v-model="includePluginDebug"
        type="checkbox"
        :disabled="busy || exportAllRoles"
      />
      {{ t("chatExport.includePluginDebugSingleRole") }}
    </label>
    <button
      type="button"
      class="btn"
      :disabled="busy"
      @click="runExport('json')"
    >
      {{ t("chatExport.exportJson") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="busy"
      @click="runExport('txt')"
    >
      {{ t("chatExport.exportTxt") }}
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
