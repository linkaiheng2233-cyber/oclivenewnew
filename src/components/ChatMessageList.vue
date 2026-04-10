<script setup lang="ts">
// 长列表：当前会话区与「此前的聊天记录」折叠区内均用 DynamicScroller。
import { computed, nextTick, ref, watch } from "vue";
import { DynamicScroller, DynamicScrollerItem } from "vue-virtual-scroller";
import ChatMessage from "./ChatMessage.vue";
import type { ChatMsg } from "../types/chatMsg";

export type { ChatMsg };

/** 每次在主聊天区多展开的历史条数 */
const PREVIEW_STEP = 20;

/** 已展开的「20 条」段数，0 表示收起 */
const historyPreviewChunks = ref(0);

const props = withDefaults(
  defineProps<{
    messages: ChatMsg[];
    /** 进入本场景时已有条数；slice(0, n) 为历史折叠区 */
    historySplitIndex?: number;
    loading: boolean;
    roleSwitching: boolean;
  }>(),
  {
    historySplitIndex: 0,
  },
);

const split = computed(() =>
  Math.min(props.historySplitIndex, props.messages.length),
);
const historicalMessages = computed(() =>
  props.messages.slice(0, split.value),
);
const currentMessages = computed(() => props.messages.slice(split.value));

const historyPreviewMaxChunks = computed(() =>
  Math.ceil(historicalMessages.value.length / PREVIEW_STEP),
);

const visibleHistoryInChat = computed(() => {
  const k = historyPreviewChunks.value;
  if (k <= 0) return [];
  const cap = Math.min(k * PREVIEW_STEP, historicalMessages.value.length);
  return historicalMessages.value.slice(-cap);
});

const canExpandMoreHistory = computed(
  () => historyPreviewChunks.value < historyPreviewMaxChunks.value,
);

function expandHistoryStep() {
  if (!canExpandMoreHistory.value) return;
  historyPreviewChunks.value += 1;
}

function collapseHistoryPreview() {
  historyPreviewChunks.value = 0;
}

const scrollerRef = ref<InstanceType<typeof DynamicScroller> | null>(null);

async function scrollToBottom(): Promise<void> {
  await nextTick();
  const n = currentMessages.value.length;
  if (n === 0) return;
  const sc = scrollerRef.value as unknown as {
    scrollToItem?: (i: number) => void;
  } | null;
  sc?.scrollToItem?.(n - 1);
}

watch(
  () => currentMessages.value.length,
  () => {
    void scrollToBottom();
  },
  { flush: "post" },
);

defineExpose({ scrollToBottom });
</script>

<template>
  <div
    class="chat-list-root"
    :class="{
      'has-messages': messages.length > 0 || roleSwitching,
    }"
  >
    <div v-if="roleSwitching" class="switching">切换中...</div>

    <div
      v-if="split > 0 && historicalMessages.length > 0"
      class="history-toolbar"
    >
      <button
        v-if="historyPreviewChunks === 0"
        type="button"
        class="history-cta-card"
        @click="expandHistoryStep"
      >
        <span class="history-cta-accent" aria-hidden="true" />
        <span class="history-cta-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
            <path d="M8 10h12M8 6h12M8 14h5" />
            <rect x="2" y="5" width="4" height="14" rx="1" />
          </svg>
        </span>
        <span class="history-cta-text">
          <span class="history-cta-title">在此展开此前的对话</span>
          <span class="history-cta-sub"
            >每次 20 条，可多次展开 · 共 {{ historicalMessages.length }} 条</span
          >
        </span>
      </button>
      <template v-else>
        <div class="history-toolbar-active">
          <div class="history-toolbar-row">
            <span class="history-meta">
              <span class="history-meta-label">已展开</span>
              <span class="history-meta-nums"
                >{{ visibleHistoryInChat.length }} /
                {{ historicalMessages.length }}</span
              >
              条
            </span>
            <div class="history-toolbar-actions">
              <button
                v-if="canExpandMoreHistory"
                type="button"
                class="history-pill history-pill--primary"
                @click="expandHistoryStep"
              >
                再显示更早 20 条
              </button>
              <button
                type="button"
                class="history-pill"
                @click="collapseHistoryPreview"
              >
                收起
              </button>
            </div>
          </div>
        </div>
      </template>
    </div>

    <div
      v-if="visibleHistoryInChat.length > 0"
      class="history-inline-chat"
      aria-label="此前的对话"
    >
      <div class="history-inline-header">
        <span class="history-inline-title">此前的对话</span>
        <span class="history-inline-hint">与下方本次会话同一滚动区域</span>
      </div>
      <ChatMessage
        v-for="m in visibleHistoryInChat"
        :key="m.id"
        :role="m.role"
        :content="m.content"
        :timestamp="m.timestamp"
        :presence-variant="m.presenceVariant"
        :reply-is-fallback="m.replyIsFallback"
      />
    </div>

    <div
      v-if="visibleHistoryInChat.length > 0"
      class="session-split-line"
      role="separator"
    >
      <span>以下为本次会话</span>
    </div>

    <DynamicScroller
      ref="scrollerRef"
      class="virtual-scroller"
      :items="currentMessages"
      :min-item-size="96"
      key-field="id"
    >
      <template #default="{ item, index, active }">
        <DynamicScrollerItem
          :item="item"
          :active="active"
          :data-index="index"
          :size-dependencies="[
            item.content,
            item.role,
            item.presenceVariant,
            item.replyIsFallback,
            item.timestamp,
          ]"
        >
          <div class="chat-scroller-slot">
            <ChatMessage
              :role="item.role"
              :content="item.content"
              :timestamp="item.timestamp"
              :presence-variant="item.presenceVariant"
              :reply-is-fallback="item.replyIsFallback"
            />
          </div>
        </DynamicScrollerItem>
      </template>
    </DynamicScroller>

    <div
      v-if="messages.length === 0 && !loading && !roleSwitching"
      class="empty-hint"
    >
      暂无消息，开始聊天吧~
    </div>

    <div v-if="loading" class="thinking">
      <div class="dot-wrap">
        <span class="dot" />
        <span class="dot" />
        <span class="dot" />
      </div>
      <span>正在想...</span>
    </div>
  </div>
</template>

<style scoped>
.chat-list-root {
  min-height: min-content;
}
.chat-list-root {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
}
.history-toolbar {
  flex-shrink: 0;
  margin-bottom: 10px;
}

/* 未展开：易发现、低张扬的入口卡片 */
.history-cta-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  width: 100%;
  margin: 0;
  padding: 12px 14px 12px 12px;
  border: 1px solid color-mix(in srgb, var(--border-light) 88%, var(--accent) 12%);
  border-radius: var(--radius-card);
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--bg-elevated) 92%, var(--accent) 8%) 0%,
    var(--bg-secondary) 48%,
    var(--bg-secondary) 100%
  );
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  text-align: left;
  font-family: var(--font-ui);
  transition: var(--control-transition);
  position: relative;
  overflow: hidden;
}

.history-cta-card:hover {
  border-color: color-mix(in srgb, var(--border-light) 72%, var(--accent) 28%);
  box-shadow: var(--shadow-md);
}

.history-cta-card:focus-visible {
  outline: 2px solid var(--focus-ring-color);
  outline-offset: 2px;
}

.history-cta-accent {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  border-radius: var(--radius-card) 0 0 var(--radius-card);
  background: color-mix(in srgb, var(--accent) 42%, var(--border-light) 58%);
  opacity: 0.85;
}

.history-cta-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  margin-top: 1px;
  border-radius: var(--radius-btn);
  color: color-mix(in srgb, var(--accent) 75%, var(--text-secondary) 25%);
  background: color-mix(in srgb, var(--accent) 10%, var(--bg-elevated) 90%);
  border: 1px solid color-mix(in srgb, var(--accent) 16%, var(--border-light) 84%);
}

.history-cta-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  padding-top: 1px;
}

.history-cta-title {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--text-primary);
  line-height: 1.35;
}

.history-cta-sub {
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

/* 已展开：紧凑状态条 */
.history-toolbar-active {
  padding: 10px 12px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-secondary) 94%, var(--accent) 6%);
  box-shadow: var(--shadow-sm);
}

.history-toolbar-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px 14px;
}

.history-meta {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.history-meta-label {
  margin-right: 4px;
  color: var(--text-light);
}

.history-meta-nums {
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.history-toolbar-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.history-pill {
  margin: 0;
  padding: 7px 12px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 12px;
  font-family: var(--font-ui);
  font-weight: 500;
  cursor: pointer;
  transition: var(--control-transition);
}

.history-pill:hover {
  color: var(--text-primary);
  border-color: color-mix(in srgb, var(--text-primary) 12%, var(--border-light));
  background: color-mix(in srgb, var(--bg-elevated) 88%, var(--accent) 12%);
}

.history-pill:focus-visible {
  outline: 2px solid var(--focus-ring-color);
  outline-offset: 2px;
}

.history-pill--primary {
  border-color: color-mix(in srgb, var(--accent) 35%, var(--border-light));
  color: color-mix(in srgb, var(--text-primary) 55%, var(--accent) 45%);
  background: color-mix(in srgb, var(--accent) 9%, var(--bg-elevated) 91%);
}

.history-pill--primary:hover {
  border-color: color-mix(in srgb, var(--accent) 48%, var(--border-light));
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-elevated) 86%);
}

/* 内联历史块：与主聊天一致的气泡 + 轻分区 */
.history-inline-chat {
  flex-shrink: 0;
  margin-bottom: 6px;
  padding: 0 10px 10px;
  border-radius: var(--radius-card);
  border: 1px solid color-mix(in srgb, var(--border-light) 82%, var(--accent) 18%);
  background: color-mix(in srgb, var(--bg-secondary) 91%, var(--accent) 9%);
  box-shadow: inset 0 1px 0 color-mix(in srgb, var(--text-primary) 5%, transparent);
}

.history-inline-header {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 6px 12px;
  padding: 10px 2px 8px;
  margin-bottom: 2px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 90%, var(--accent) 10%);
}

.history-inline-title {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: color-mix(in srgb, var(--text-secondary) 65%, var(--accent) 35%);
}

.history-inline-hint {
  font-size: 11px;
  color: var(--text-light);
}

.session-split-line {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 6px 0 14px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: color-mix(in srgb, var(--text-secondary) 78%, var(--accent) 22%);
}

.session-split-line span {
  max-width: 14rem;
  text-align: center;
  line-height: 1.35;
}

.session-split-line::before,
.session-split-line::after {
  content: "";
  flex: 1;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    color-mix(in srgb, var(--border-light) 75%, var(--accent) 25%) 22%,
    color-mix(in srgb, var(--border-light) 75%, var(--accent) 25%) 78%,
    transparent
  );
}
/* 虚拟列表内部留白：滚到底时最后一条与输入栏留出空隙 */
.virtual-scroller {
  flex: 1;
  min-height: 0;
  padding-bottom: max(52px, env(safe-area-inset-bottom, 0px));
  box-sizing: border-box;
}

/* 计入 DynamicScrollerItem 的测量高度，避免子项外缘被裁 */
.chat-scroller-slot {
  min-height: min-content;
  padding-bottom: 8px;
}
.empty-hint {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
  padding: 24px 12px;
}
.switching {
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 6px;
}
.thinking {
  margin: 10px 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-bubble);
  background: var(--bubble-bot-bg);
  box-shadow: var(--shadow-sm);
  color: var(--text-secondary);
}
.dot-wrap {
  display: inline-flex;
  gap: 3px;
}
.dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--text-light);
  animation: blink 900ms infinite ease-in-out;
}
.dot:nth-child(2) {
  animation-delay: 120ms;
}
.dot:nth-child(3) {
  animation-delay: 240ms;
}
@keyframes blink {
  0%,
  80%,
  100% {
    opacity: 0.3;
    transform: scale(0.8);
  }
  40% {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
