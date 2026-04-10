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
    /** 非空时在正文内高亮子串（如历史检索） */
    highlightQuery?: string;
  }>(),
  { replyIsFallback: false, highlightQuery: "" },
);

function contentSegments(
  content: string,
  q: string,
): { text: string; hit: boolean }[] {
  const needle = q.trim().toLowerCase();
  if (!needle) return [{ text: content, hit: false }];
  const lower = content.toLowerCase();
  const segments: { text: string; hit: boolean }[] = [];
  let i = 0;
  while (i < content.length) {
    const j = lower.indexOf(needle, i);
    if (j < 0) {
      if (i < content.length) segments.push({ text: content.slice(i), hit: false });
      break;
    }
    if (j > i) segments.push({ text: content.slice(i, j), hit: false });
    segments.push({ text: content.slice(j, j + needle.length), hit: true });
    i = j + needle.length;
  }
  return segments.length ? segments : [{ text: content, hit: false }];
}

const highlightedSegments = computed(() =>
  contentSegments(props.content, props.highlightQuery ?? ""),
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
    <div v-if="props.role === 'system'" class="system-line">
      <template v-for="(seg, idx) in highlightedSegments" :key="idx">
        <mark v-if="seg.hit" class="search-hit">{{ seg.text }}</mark>
        <span v-else>{{ seg.text }}</span>
      </template>
    </div>
    <div v-else class="bubble-column" :class="props.role">
      <div class="bubble" :class="[props.role, presenceBubbleClass]">
        <div class="content">
          <template v-for="(seg, idx) in highlightedSegments" :key="idx">
            <mark v-if="seg.hit" class="search-hit">{{ seg.text }}</mark>
            <span v-else>{{ seg.text }}</span>
          </template>
        </div>
        <div v-if="props.replyIsFallback && props.role === 'assistant'" class="fallback-hint">
          备用短回复
        </div>
      </div>
      <div class="bubble-meta">
        <span class="time">{{ hhmm(props.timestamp) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/*
 * 垂直间距用 padding 而非 margin：虚拟列表用父级 offsetHeight 测量，
 * 子项 margin 常不计入 DynamicScrollerItem 外框，会导致 item-wrapper overflow:hidden 裁掉底部时间。
 */
.row {
  display: flex;
  margin: 0;
  box-sizing: border-box;
}
.row.user {
  justify-content: flex-end;
  padding: 9px 0 20px min(12%, 52px);
}
.row.assistant {
  justify-content: flex-start;
  padding: 9px min(12%, 52px) 20px 0;
}
.row.system {
  justify-content: center;
  padding: 14px 0;
}
.system-line {
  width: 100%;
  max-width: 42rem;
  margin: 0 auto;
  text-align: center;
  font-size: 12px;
  color: var(--text-secondary);
  padding: 9px 16px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--bg-elevated) 78%, transparent);
  border: 1px solid color-mix(in srgb, var(--border-light) 75%, var(--accent) 25%);
  box-shadow:
    var(--shadow-sm),
    inset 0 1px 0 color-mix(in srgb, var(--text-primary) 4%, transparent);
}

/* 一列：气泡 + 底部时间，对齐方式随角色 */
.bubble-column {
  display: flex;
  flex-direction: column;
  max-width: min(85%, 100%);
  gap: 5px;
}
.bubble-column.user {
  align-items: flex-end;
}
.bubble-column.assistant {
  align-items: flex-start;
}

.bubble-meta {
  display: flex;
  width: 100%;
  padding: 0 4px;
}
.bubble-column.user .bubble-meta {
  justify-content: flex-end;
}
.bubble-column.assistant .bubble-meta {
  justify-content: flex-start;
}

.bubble {
  width: 100%;
  padding: 12px 16px 13px;
  border: 1px solid color-mix(in srgb, var(--bubble-border) 88%, var(--accent) 12%);
  font-size: 15px;
  line-height: 1.55;
  animation: bubbleIn 0.32s cubic-bezier(0.22, 1, 0.36, 1);
  position: relative;
}

/* 聊天气泡：一角略尖，朝向对话中部 */
.bubble.user {
  border-radius: 20px 20px 6px 20px;
  color: var(--text-primary);
  background: linear-gradient(
    168deg,
    color-mix(in srgb, var(--bubble-user-bg) 82%, var(--bg-elevated) 18%) 0%,
    var(--bubble-user-bg) 42%,
    color-mix(in srgb, var(--bubble-user-bg) 90%, var(--accent) 10%) 100%
  );
  box-shadow:
    0 2px 4px color-mix(in srgb, var(--text-primary) 5%, transparent),
    0 8px 22px color-mix(in srgb, var(--text-primary) 7%, transparent),
    inset 0 1px 0 color-mix(in srgb, var(--text-primary) 7%, transparent);
}

.bubble.assistant {
  border-radius: 20px 20px 20px 7px;
  color: var(--text-primary);
  background: linear-gradient(
    172deg,
    color-mix(in srgb, var(--bubble-bot-bg) 72%, var(--bg-elevated) 28%) 0%,
    var(--bubble-bot-bg) 38%,
    color-mix(in srgb, var(--bubble-bot-bg) 93%, var(--accent) 7%) 100%
  );
  box-shadow:
    0 2px 4px color-mix(in srgb, var(--text-primary) 4%, transparent),
    0 7px 20px color-mix(in srgb, var(--text-primary) 6%, transparent),
    inset 0 1px 0 color-mix(in srgb, var(--text-primary) 6%, transparent);
}

.bubble.assistant.presence-remote-life {
  opacity: 0.94;
  color: var(--text-secondary);
  border-color: var(--border-light);
  border-radius: var(--radius-bubble);
  background: var(--bubble-bot-bg);
  box-shadow: var(--shadow-bubble);
}
.bubble.assistant.presence-remote-stub {
  font-style: italic;
  opacity: 0.88;
  color: var(--text-secondary);
  border-color: var(--border-light);
  border-radius: var(--radius-bubble);
  background: var(--bubble-bot-bg);
  box-shadow: var(--shadow-bubble);
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
  margin-top: 8px;
  padding-top: 6px;
  border-top: 1px solid color-mix(in srgb, var(--border-light) 70%, transparent);
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.88;
}

.time {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.02em;
  color: var(--text-light);
  opacity: 0.92;
}
.search-hit {
  background: color-mix(in srgb, var(--accent) 38%, transparent);
  color: inherit;
  padding: 0 1px;
  border-radius: 2px;
}
@keyframes bubbleIn {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .bubble {
    animation: none;
  }
}
</style>
