<script setup>
defineProps({
  message: {
    type: String,
    required: true,
  },
  busy: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["update:message", "send-message", "create-event"]);

function onMessageInput(event) {
  emit("update:message", event.target.value);
}
</script>

<template>
  <section class="card">
    <h2>对话操作</h2>
    <div class="row">
      <input :value="message" placeholder="输入消息" @input="onMessageInput" />
      <button :disabled="busy" @click="emit('send-message')">send_message</button>
      <button :disabled="busy" @click="emit('create-event')">create_event(Praise)</button>
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
