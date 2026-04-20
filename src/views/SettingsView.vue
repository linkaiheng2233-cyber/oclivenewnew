<script setup lang="ts">
import { ref } from "vue";
import HelpHint from "../components/HelpHint.vue";
import HotkeySettingsSection from "../components/HotkeySettingsSection.vue";
import PluginSettingsPanelSlots from "../components/PluginSettingsPanelSlots.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import { useAppToast } from "../composables/useAppToast";
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from "../stores/pluginStore";
import { useUiStore } from "../stores/uiStore";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const pluginStore = usePluginStore();
const uiStore = useUiStore();
const { showToast } = useAppToast();

type SettingsTab = "general" | "plugins";

const tab = ref<SettingsTab>("general");

async function onToggleForceIframe(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  pluginStore.pluginState = {
    ...pluginStore.pluginState,
    force_iframe_mode: checked,
  };
  try {
    await pluginStore.persist();
    showToast("info", "已保存。重启应用后强制 iframe 模式将完全生效。");
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
    pluginStore.pluginState = {
      ...pluginStore.pluginState,
      force_iframe_mode: !checked,
    };
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="sv-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="设置"
      @click.self="emit('close')"
    >
      <div class="sv-dialog" @click.stop>
        <header class="sv-head">
          <h2 class="sv-title">设置</h2>
          <button type="button" class="sv-close" aria-label="关闭" @click="emit('close')">×</button>
        </header>

        <nav class="sv-nav" aria-label="设置分区">
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'general' ? 'page' : undefined"
            @click="tab = 'general'"
          >
            常规
          </button>
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'plugins' ? 'page' : undefined"
            @click="tab = 'plugins'"
          >
            插件扩展
          </button>
        </nav>

        <div v-show="tab === 'general'" class="sv-body">
          <p class="sv-lead">
            顶栏<strong>「更多」</strong>集中设置入口；打开设置可用 <strong>Ctrl+Shift+S</strong>；目录插件与后端会话覆盖用 <strong>Ctrl+Shift+F</strong>。
          </p>
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">快捷</span>
              <HelpHint text="Ctrl+Shift+S 打开设置；Ctrl+Shift+F 打开插件与后端管理；Ctrl+Shift+D 开关调试面板。" />
            </div>
            <p class="sv-muted">
              虚拟时间、叙事场景等仅在沉浸模式下显示于「更多」。
            </p>
          </section>
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">实验性功能</span>
              <HelpHint
                text="灰度入口：用于预览新版插件管理界面（V2）。若当前构建未集成 V2，会继续使用现有专业模式。"
              />
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="uiStore.experimentalPluginManagerV2 === true"
                @change="uiStore.setExperimentalPluginManagerV2(($event.target as HTMLInputElement).checked)"
              />
              <span class="sv-toggle-text">
                <strong>启用新版插件管理界面（V2 预览）</strong>
                <span class="sv-muted sv-toggle-desc">
                  开启后，Ctrl+Shift+F 与「更多」里的入口将优先尝试打开 V2。
                </span>
              </span>
            </label>
          </section>
          <section class="sv-section">
            <h3 class="sv-h3">扩展区（settings.advanced）</h3>
            <p class="sv-muted">manifest 中声明 <code>settings.advanced</code> 的插件显示于此。</p>
            <PluginSlotEmbed
              :slot-name="SLOT_SETTINGS_ADVANCED"
              aria-label="设置扩展区"
              :bootstrap-epoch="pluginStore.bootstrapEpoch"
            />
          </section>

          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">安全</span>
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="pluginStore.pluginState.force_iframe_mode === true"
                @change="onToggleForceIframe"
              />
              <span class="sv-toggle-text">
                <strong>强制 iframe 模式</strong>
                <span class="sv-muted sv-toggle-desc">
                  开启后，所有插件界面将使用 iframe 加载，更安全但体验可能下降。保存后需重启应用以完全生效。
                </span>
              </span>
            </label>
          </section>
        </div>

        <div v-show="tab === 'plugins'" class="sv-body">
          <section class="sv-section">
            <div class="sv-row-h">
              <h3 class="sv-h3">目录插件 · 设置页插槽</h3>
              <HelpHint
                :paragraphs="[
                  '在插件 manifest 的 ui_slots 中声明 slot 为 settings.panel，即可在此嵌入配置页。',
                  '与 chat_toolbar 相同，使用 https://ocliveplugin.localhost/<id>/<entry> 加载；可在插件管理中调整顺序或隐藏。',
                ]"
              />
            </div>
            <PluginSettingsPanelSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
          </section>

          <HotkeySettingsSection />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.sv-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10040;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.sv-dialog {
  position: relative;
  width: min(640px, 100%);
  max-height: min(90vh, 800px);
  overflow: auto;
  padding: 16px 18px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.sv-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 8px;
}
.sv-title {
  margin: 0;
  font-size: 18px;
}
.sv-close {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.sv-nav {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid var(--border-light);
  padding-bottom: 8px;
}
.sv-nav-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-nav-btn[aria-current="page"] {
  border-color: var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.sv-body {
  flex: 1;
  min-height: 0;
}
.sv-lead {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.sv-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sv-row-h {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.sv-label {
  font-weight: 600;
  font-size: 14px;
}
.sv-h3 {
  margin: 0;
  font-size: 15px;
}
.sv-muted {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.sv-toggle-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  cursor: pointer;
  font-size: 13px;
  line-height: 1.45;
}
.sv-toggle-row input {
  margin-top: 3px;
  flex-shrink: 0;
}
.sv-toggle-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.sv-toggle-desc {
  display: block;
  font-weight: 400;
}
</style>
