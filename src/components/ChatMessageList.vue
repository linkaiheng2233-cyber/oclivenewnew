<script setup lang="ts">
// 长列表：当前会话区与「此前的聊天记录」折叠区内均用 DynamicScroller。
import { computed, nextTick, ref, watch } from "vue";
import { DynamicScroller, DynamicScrollerItem } from "vue-virtual-scroller";
import ChatMessage from "./ChatMessage.vue";

export type ChatMsg = {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  presenceVariant?: "co_present" | "remote_stub" | "remote_life";
  /** 与后端 `reply_is_fallback` 一致 */
  replyIsFallback?: boolean;
};

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

    <details
      v-if="split > 0 && historicalMessages.length > 0"
      class="history-details"
    >
      <summary class="history-summary">
        此前的聊天记录（{{ historicalMessages.length }} 条）
      </summary>
      <div class="history-list">
        <DynamicScroller
          class="history-virtual-scroller"
          :items="historicalMessages"
          :min-item-size="72"
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
              ]"
            >
              <ChatMessage
                :role="item.role"
                :content="item.content"
                :timestamp="item.timestamp"
                :presence-variant="item.presenceVariant"
                :reply-is-fallback="item.replyIsFallback"
              />
            </DynamicScrollerItem>
          </template>
        </DynamicScroller>
      </div>
    </details>

    <DynamicScroller
      ref="scrollerRef"
      class="virtual-scroller"
      :items="currentMessages"
      :min-item-size="72"
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
          ]"
        >
          <ChatMessage
            :role="item.role"
            :content="item.content"
            :timestamp="item.timestamp"
            :presence-variant="item.presenceVariant"
            :reply-is-fallback="item.replyIsFallback"
          />
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
.history-details {
  flex-shrink: 0;
  margin-bottom: 10px;
  border-radius: var(--radius-bubble);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  overflow: hidden;
}
.history-summary {
  cursor: pointer;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-secondary);
  list-style: none;
}
.history-summary::-webkit-details-marker {
  display: none;
}
.history-summary::before {
  content: "▸ ";
  display: inline-block;
  transition: transform 0.15s ease;
}
.history-details[open] .history-summary::before {
  transform: rotate(90deg);
}
.history-list {
  height: min(40vh, 320px);
  min-height: 0;
  padding: 0 10px 10px;
  border-top: 1px solid var(--border-light);
}
.history-virtual-scroller {
  height: 100%;
  min-height: 0;
}
.virtual-scroller {
  flex: 1;
  min-height: 0;
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
