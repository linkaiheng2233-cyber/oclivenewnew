<script setup lang="ts">
import PluginSlotEmbed from "./PluginSlotEmbed.vue";
import { SLOT_LAUNCHER_PALETTE } from "../stores/pluginStore";

withDefaults(
  defineProps<{
    modelValue: boolean;
    /** 与插件 bootstrap 同步 */
    bootstrapEpoch?: number;
  }>(),
  { bootstrapEpoch: 0 },
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const rows: { keys: string; desc: string }[] = [
  { keys: "Ctrl + Shift + S", desc: "打开设置（扩展区、安全、快捷键与插件配置）" },
  { keys: "Ctrl + Shift + F", desc: "打开插件与后端管理（界面插件 / 后端模块 / 插槽顺序）" },
  { keys: "Ctrl（长按约 1 秒）", desc: "打开本快捷键说明" },
];
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="sh-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="快捷键"
      @click.self="emit('update:modelValue', false)"
    >
      <div class="sh-dialog" @click.stop>
        <header class="sh-head">
          <h2 class="sh-title">快捷键</h2>
          <button
            type="button"
            class="sh-close"
            aria-label="关闭"
            @click="emit('update:modelValue', false)"
          >
            ×
          </button>
        </header>
        <table class="sh-table">
          <tbody>
            <tr v-for="(r, i) in rows" :key="i">
              <td class="sh-keys">{{ r.keys }}</td>
              <td class="sh-desc">{{ r.desc }}</td>
            </tr>
          </tbody>
        </table>
        <p class="sh-foot">更多快捷键将随功能迭代补充。</p>
        <section class="sh-slot" aria-label="启动器插槽">
          <h3 class="sh-slot-h">插件槽（launcher.palette）</h3>
          <PluginSlotEmbed
            :slot-name="SLOT_LAUNCHER_PALETTE"
            aria-label="启动器插槽"
            :bootstrap-epoch="bootstrapEpoch"
          />
        </section>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.sh-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10060;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.sh-dialog {
  width: min(420px, 100%);
  max-height: min(80vh, 520px);
  overflow: auto;
  padding: 16px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.sh-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.sh-title {
  margin: 0;
  font-size: 17px;
}
.sh-close {
  border: none;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.sh-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.sh-table td {
  padding: 8px 6px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 60%, transparent);
  vertical-align: top;
}
.sh-keys {
  white-space: nowrap;
  font-weight: 600;
  color: var(--text-primary);
  width: 46%;
}
.sh-desc {
  color: var(--text-secondary);
}
.sh-foot {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.sh-slot {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid color-mix(in srgb, var(--border-light) 60%, transparent);
}
.sh-slot-h {
  margin: 0 0 8px;
  font-size: 13px;
}
</style>
