<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{
  message: string;
  busy?: boolean;
}>();

const emit = defineEmits<{
  "update:message": [value: string];
  "send-message": [];
  "create-event": [];
}>();

const { t } = useI18n();

function onMessageInput(event: Event): void {
  const el = event.target as HTMLInputElement;
  emit("update:message", el.value);
}
</script>

<template>
  <section class="card">
    <h2>{{ t("chat.demoTitle") }}</h2>
    <div class="row">
      <input
        :value="message"
        :placeholder="t('chat.demoPlaceholder')"
        @input="onMessageInput"
      />
      <button :disabled="busy" @click="emit('send-message')">{{ t("chat.demoSend") }}</button>
      <button :disabled="busy" @click="emit('create-event')">{{ t("chat.demoCreateEvent") }}</button>
    </div>
  </section>
</template>

<style scoped>
.card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}
.row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
