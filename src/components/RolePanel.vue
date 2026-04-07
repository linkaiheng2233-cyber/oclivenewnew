<script setup>
defineProps({
  roleId: {
    type: String,
    required: true,
  },
  status: {
    type: String,
    default: "",
  },
  busy: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits([
  "update:roleId",
  "load-role",
  "get-role-info",
  "reload-policy-plugins",
]);

function onRoleInput(event) {
  emit("update:roleId", event.target.value);
}
</script>

<template>
  <section class="card">
    <h2>角色操作</h2>
    <div class="row">
      <input :value="roleId" placeholder="role_id" @input="onRoleInput" />
      <button :disabled="busy" @click="emit('load-role')">load_role</button>
      <button :disabled="busy" @click="emit('get-role-info')">get_role_info</button>
      <button :disabled="busy" @click="emit('reload-policy-plugins')">
        reload_policy_plugins
      </button>
    </div>
    <p class="status">{{ status }}</p>
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
.status {
  margin-top: 10px;
  color: #3366cc;
  min-height: 20px;
}
</style>
