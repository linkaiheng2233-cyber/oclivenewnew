<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    role: "user" | "assistant" | "system";
    content: string;
    timestamp: number;
    /** 异地心声 / 占位 等气泡样式 */
    presenceVariant?: "co_present" | "remote_stub" | "remote_life";
    /** 与后端 `reply_is_fallback` 一致 */
    replyIsFallback?: boolean;
  }>(),
  { replyIsFallback: false },
);

const presenceBubbleClass = computed(() => {
  if (props.presenceVariant === "remote_life") return "presence-remote-life";
  if (props.presenceVariant === "remote_stub") return "presence-remote-stub";
  return "";
});

function hhmm(ts: number): string {
  const d = new Date(ts);
  const h = `${d.getHours()}`.padStart(2, "0");
  const m = `${d.getMinutes()}`.padStart(2, "0");
  return `${h}:${m}`;
}
</script>

<template>
  <div class="row" :class="props.role">
    <div v-if="props.role === 'system'" class="system-line">{{ props.content }}</div>
    <div v-else class="bubble" :class="[props.role, presenceBubbleClass]">
      <div class="content">{{ props.content }}</div>
      <div v-if="props.replyIsFallback && props.role === 'assistant'" class="fallback-hint">
        备用短回复
      </div>
      <div class="time">{{ hhmm(props.timestamp) }}</div>
    </div>
  </div>
</template>

<style scoped>
.row { display: flex; margin: 10px 0; }
.row.user { justify-content: flex-end; }
.row.assistant { justify-content: flex-start; }
.row.system { justify-content: center; }
.system-line {
  width: 100%;
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
  padding: 6px 12px;
}
.bubble {
  max-width: min(85%, 100%);
  padding: 14px 18px;
  border-radius: var(--radius-bubble);
  box-shadow: var(--shadow-sm);
  border: 1px solid var(--bubble-border);
  font-size: 15px;
  line-height: 1.5;
  animation: fadeInUp 0.25s ease;
}
.bubble.user {
  background: var(--bubble-user-bg);
  color: var(--text-primary);
}
.bubble.assistant {
  background: var(--bubble-bot-bg);
  color: var(--text-primary);
}
.bubble.assistant.presence-remote-life {
  opacity: 0.94;
  color: var(--text-secondary);
  border-color: var(--border-light);
}
.bubble.assistant.presence-remote-stub {
  font-style: italic;
  opacity: 0.88;
  color: var(--text-secondary);
}
/* 长回复：限制可视高度，在气泡内滚动，避免一条消息占满窗口 */
.content {
  white-space: pre-wrap;
  word-break: break-word;
  overflow-wrap: anywhere;
}
.bubble.assistant .content {
  max-height: min(36vh, 280px);
  overflow-y: auto;
  overflow-x: hidden;
  overscroll-behavior: contain;
  font-size: 14px;
  line-height: 1.55;
  scrollbar-color: var(--scrollbar-chat-thumb) var(--scrollbar-chat-track);
  scrollbar-width: thin;
}
.bubble.user .content {
  max-height: min(28vh, 220px);
  overflow-y: auto;
  overflow-x: hidden;
  overscroll-behavior: contain;
  scrollbar-color: var(--scrollbar-chat-thumb) var(--scrollbar-chat-track);
  scrollbar-width: thin;
}
.bubble.assistant .content::-webkit-scrollbar,
.bubble.user .content::-webkit-scrollbar {
  width: 4px;
}
.bubble.assistant .content::-webkit-scrollbar-track,
.bubble.user .content::-webkit-scrollbar-track {
  background: var(--scrollbar-chat-track);
  border-radius: 4px;
}
.bubble.assistant .content::-webkit-scrollbar-thumb,
.bubble.user .content::-webkit-scrollbar-thumb {
  background: var(--scrollbar-chat-thumb);
  border-radius: 4px;
}
.fallback-hint {
  margin-top: 6px;
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.85;
}
.time { margin-top: 6px; font-size: 12px; color: var(--text-light); }
@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
