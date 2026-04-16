<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { usePluginStore } from "../stores/pluginStore";
import type { HotkeyAction } from "../utils/tauri-api";

const pluginStore = usePluginStore();

const launcherOpen = ref(false);
const hotkeyTarget = ref<{
  pluginId: string;
  slot: string;
  appearanceId: string;
} | null>(null);

const activeSlot = computed(() => {
  const t = hotkeyTarget.value;
  if (!t) {
    return null;
  }
  return pluginStore.bootstrapUiSlots.find(
    (s) =>
      s.pluginId === t.pluginId &&
      s.slot === t.slot &&
      (s.appearanceId ?? "") === (t.appearanceId ?? ""),
  );
});

let unlisten: UnlistenFn | undefined;

onMounted(async () => {
  unlisten = await listen<{ bindingId: string; action: HotkeyAction }>(
    "hotkey-action",
    (e) => {
      const a = e.payload.action;
      if (a.type === "openLauncherList") {
        launcherOpen.value = true;
        return;
      }
      if (a.type === "openPluginSlot") {
        hotkeyTarget.value = {
          pluginId: a.pluginId,
          slot: a.slot,
          appearanceId: (a.appearanceId ?? "").trim(),
        };
      }
    },
  );
});

onBeforeUnmount(() => {
  unlisten?.();
});

function closeHotkeyPlugin(): void {
  hotkeyTarget.value = null;
}

function closeLauncher(): void {
  launcherOpen.value = false;
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="hotkeyTarget && activeSlot"
      class="hk-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件快捷窗口"
      @click.self="closeHotkeyPlugin"
    >
      <div class="hk-dialog">
        <header class="hk-head">
          <span class="hk-title">{{ activeSlot.pluginId }} · {{ activeSlot.slot }}</span>
          <button type="button" class="hk-close" aria-label="关闭" @click="closeHotkeyPlugin">
            ×
          </button>
        </header>
        <iframe
          class="hk-frame"
          :src="activeSlot.url"
          :title="`plugin ${activeSlot.pluginId}`"
          referrerpolicy="no-referrer"
        />
      </div>
    </div>
    <div
      v-else-if="hotkeyTarget"
      class="hk-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="插件未找到"
      @click.self="closeHotkeyPlugin"
    >
      <div class="hk-dialog hk-dialog--narrow">
        <header class="hk-head">
          <span class="hk-title">无法打开插件页</span>
          <button type="button" class="hk-close" aria-label="关闭" @click="closeHotkeyPlugin">
            ×
          </button>
        </header>
        <p class="hk-muted">
          当前角色 bootstrap 中未找到
          <strong>{{ hotkeyTarget.pluginId }}</strong> 在槽
          <strong>{{ hotkeyTarget.slot }}</strong>
          的界面；请确认插件已启用、未隐藏该槽贡献，并已保存插件配置。
        </p>
      </div>
    </div>
    <div
      v-if="launcherOpen"
      class="hk-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="可启动插件"
      @click.self="closeLauncher"
    >
      <div class="hk-dialog hk-dialog--narrow">
        <header class="hk-head">
          <span class="hk-title">插件目录</span>
          <button type="button" class="hk-close" aria-label="关闭" @click="closeLauncher">
            ×
          </button>
        </header>
        <ul class="hk-launch-list">
          <li v-for="p in pluginStore.catalog" :key="p.id">
            <span class="hk-launch-id">{{ p.id }}</span>
            <span v-if="p.uiSlotNames?.length" class="hk-launch-slots">{{
              p.uiSlotNames.join(", ")
            }}</span>
          </li>
        </ul>
        <p v-if="!pluginStore.catalog.length" class="hk-muted">未扫描到插件。</p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.hk-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10070;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.hk-dialog {
  width: min(720px, 100%);
  max-height: min(88vh, 640px);
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  overflow: hidden;
}
.hk-dialog--narrow {
  width: min(420px, 100%);
  max-height: min(80vh, 520px);
  padding: 0 0 12px;
}
.hk-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
}
.hk-title {
  font-size: 14px;
  font-weight: 600;
}
.hk-close {
  border: none;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.hk-frame {
  width: 100%;
  min-height: 360px;
  flex: 1;
  border: none;
  background: var(--bg-elevated);
}
.hk-launch-list {
  margin: 0;
  padding: 8px 14px;
  list-style: none;
  max-height: 420px;
  overflow: auto;
  font-size: 13px;
}
.hk-launch-list li {
  padding: 8px 0;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.hk-launch-id {
  font-weight: 600;
}
.hk-launch-slots {
  font-size: 12px;
  color: var(--text-secondary);
}
.hk-muted {
  margin: 8px 14px;
  font-size: 13px;
  color: var(--text-secondary);
}
</style>
