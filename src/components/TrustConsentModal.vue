<script setup lang="ts">
/**
 * 安全说明弹层：遮罩 + 摘要 + 分组能力说明 + 主次按钮。
 * 视觉与 PluginManagerPanel 的 pm-modal 系一致；权限粒度参考常见 IDE，不做界面仿造。
 */
const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title: string;
    subtitle?: string;
    trustSummary?: string;
    trustSummaryTitle?: string;
    hint: string;
    /** 只读展示的能力行（建议文案自带分组前缀，便于扫读） */
    capabilities?: readonly string[];
    confirmLabel: string;
    cancelLabel?: string;
    /** trust：主按钮强调色；danger：主按钮警示（如仍要清除） */
    variant?: "trust" | "danger";
    /** 禁止点击遮罩关闭（强制显式选择） */
    requireExplicitDismiss?: boolean;
  }>(),
  {
    subtitle: "",
    trustSummary: "",
    trustSummaryTitle: "",
    capabilities: () => [],
    cancelLabel: "",
    variant: "trust",
    requireExplicitDismiss: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [open: boolean];
  confirm: [];
  cancel: [];
}>();

function close(): void {
  emit("update:modelValue", false);
}

function onBackdropClick(): void {
  if (props.requireExplicitDismiss) return;
  emit("cancel");
  close();
}

function onCancel(): void {
  emit("cancel");
  close();
}

function onConfirm(): void {
  emit("confirm");
  close();
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="tcm-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
      @click.self="onBackdropClick"
    >
      <div class="tcm-modal" @click.stop>
        <div class="tcm-head">
          <h2 class="tcm-title">{{ title }}</h2>
          <p v-if="subtitle" class="tcm-sub">{{ subtitle }}</p>
        </div>

        <div v-if="trustSummary" class="tcm-trust-block">
          <div v-if="trustSummaryTitle" class="tcm-trust-h">{{ trustSummaryTitle }}</div>
          <div class="tcm-trust-mono">{{ trustSummary }}</div>
        </div>

        <p class="tcm-hint">{{ hint }}</p>

        <ul v-if="capabilities.length" class="tcm-cap-list" role="list">
          <li v-for="(line, idx) in capabilities" :key="`cap-${idx}`" class="tcm-cap-li" role="listitem">
            <span class="tcm-cap-icon" aria-hidden="true" />
            <span class="tcm-cap-text">{{ line }}</span>
          </li>
        </ul>

        <div class="tcm-actions">
          <button type="button" class="tcm-btn secondary" @click="onCancel">
            {{ cancelLabel || $t("common.cancel") }}
          </button>
          <button
            type="button"
            class="tcm-btn"
            :class="{ primary: variant === 'trust', danger: variant === 'danger' }"
            @click="onConfirm"
          >
            {{ confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.tcm-backdrop {
  position: fixed;
  inset: 0;
  /* 高于 Toast(12000)、本地模型/设置等 Teleport 遮罩，避免按钮点击被挡 */
  z-index: 50000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 55%, transparent));
}
.tcm-modal {
  width: min(520px, 100%);
  max-height: min(86vh, 720px);
  overflow: auto;
  padding: 16px 18px 14px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.tcm-head {
  margin-bottom: 10px;
}
.tcm-title {
  margin: 0 0 4px;
  font-size: 15px;
  font-weight: 650;
  line-height: 1.35;
  color: var(--text-primary);
}
.tcm-sub {
  margin: 0 0 2px;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.tcm-trust-block {
  margin: 0 0 10px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
}
.tcm-trust-h {
  font-weight: 600;
  margin-bottom: 6px;
  color: var(--text-secondary);
}
.tcm-trust-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New",
    monospace;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.45;
}
.tcm-hint {
  margin: 0 0 10px;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-secondary);
}
.tcm-cap-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.tcm-cap-li {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin: 0;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-primary);
}
.tcm-cap-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  margin-top: 1px;
  border-radius: 4px;
  border: 1px solid color-mix(in srgb, var(--accent, #3b82f6) 40%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 12%, transparent);
}
.tcm-cap-text {
  flex: 1;
  min-width: 0;
}
.tcm-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
}
.tcm-btn {
  padding: 8px 16px;
  border-radius: var(--radius-btn, 8px);
  border: 1px solid var(--border-light);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--text-primary);
}
.tcm-btn.secondary:hover {
  background: color-mix(in srgb, var(--border-light) 55%, transparent);
}
.tcm-btn.primary {
  background: var(--accent, #3b82f6);
  color: var(--bg-elevated, #fff);
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 85%, var(--text-primary) 15%);
}
.tcm-btn.primary:hover {
  filter: brightness(1.05);
}
.tcm-btn.danger {
  background: color-mix(in srgb, #b91c1c 92%, var(--bg-primary) 8%);
  color: #fff;
  border-color: #7f1d1d;
}
.tcm-btn.danger:hover {
  filter: brightness(1.06);
}
</style>
