<script setup lang="ts">
import { useI18n } from 'vue-i18n'

export interface SceneDestinationOption { id: string, label: string }

defineProps<{
  togetherVisible: boolean
  postReplyVisible: boolean
  destinationOptions: SceneDestinationOption[]
  togetherSelectedId: string
  postReplySelectedId: string
}>()

const emit = defineEmits<{
  'update:togetherSelectedId': [value: string]
  'update:postReplySelectedId': [value: string]
  'confirmTogether': [together: boolean]
  'dismissTogether': []
  'confirmPostReply': [together: boolean]
  'dismissPostReply': []
}>()

const { t } = useI18n()
</script>

<template>
  <div
    v-if="togetherVisible"
    class="post-reply-scene-bar"
    role="region"
    :aria-label="t('common.sceneTravel.togetherAria')"
  >
    <label class="post-reply-label" for="together-travel-select">{{
      t("common.sceneTravel.togetherLabel")
    }}</label>
    <select
      id="together-travel-select"
      class="post-reply-select"
      name="together_travel_scene"
      :value="togetherSelectedId"
      @change="emit('update:togetherSelectedId', ($event.target as HTMLSelectElement).value)"
    >
      <option disabled value="">
        {{ t("common.sceneTravel.pickPlaceholder") }}
      </option>
      <option v-for="s in destinationOptions" :key="s.id" :value="s.id">
        {{ s.label }}
      </option>
    </select>
    <button type="button" class="post-reply-btn" @click="emit('confirmTogether', false)">
      {{
        t("common.sceneTravel.solo")
      }}
    </button>
    <button
      type="button"
      class="post-reply-btn post-reply-btn--primary"
      @click="emit('confirmTogether', true)"
    >
      {{ t("common.sceneTravel.together") }}
    </button>
    <button type="button" class="post-reply-btn" @click="emit('dismissTogether')">
      {{
        t("common.sceneTravel.dismiss")
      }}
    </button>
  </div>
  <div
    v-if="postReplyVisible"
    class="post-reply-scene-bar"
    role="region"
    :aria-label="t('common.sceneTravel.postAria')"
  >
    <label class="post-reply-label" for="post-reply-scene-select">{{
      t("common.sceneTravel.postLabel")
    }}</label>
    <select
      id="post-reply-scene-select"
      class="post-reply-select"
      name="post_reply_scene"
      :value="postReplySelectedId"
      @change="emit('update:postReplySelectedId', ($event.target as HTMLSelectElement).value)"
    >
      <option disabled value="">
        {{ t("common.sceneTravel.pickPlaceholder") }}
      </option>
      <option v-for="s in destinationOptions" :key="s.id" :value="s.id">
        {{ s.label }}
      </option>
    </select>
    <button type="button" class="post-reply-btn" @click="emit('confirmPostReply', false)">
      {{
        t("common.sceneTravel.solo")
      }}
    </button>
    <button
      type="button"
      class="post-reply-btn post-reply-btn--primary"
      @click="emit('confirmPostReply', true)"
    >
      {{ t("common.sceneTravel.together") }}
    </button>
    <button type="button" class="post-reply-btn" @click="emit('dismissPostReply')">
      {{
        t("common.sceneTravel.dismiss")
      }}
    </button>
  </div>
</template>

<style scoped>
.post-reply-scene-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 10px 18px;
  font-size: 13px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
}
.post-reply-label {
  flex-shrink: 0;
}
.post-reply-select {
  flex: 1;
  min-width: 160px;
  max-width: 100%;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 6px 10px;
  font-size: 13px;
  color: var(--text-primary);
  background: var(--bg-elevated);
}
.post-reply-btn {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.post-reply-btn--primary {
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
  border-color: var(--accent);
}
</style>
