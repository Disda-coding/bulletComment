<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{
  (e: "toggle-pin"): void;
  (e: "close"): void;
}>();

const isPinned = ref(false);

function togglePin() {
  isPinned.value = !isPinned.value;
  emit("toggle-pin");
}
</script>

<template>
  <div class="control-bar" data-tauri-drag-region>
    <button class="ctrl-btn pin-btn" @click="togglePin" :title="isPinned ? '解除固定' : '固定窗口'">
      {{ isPinned ? "📌 已固定" : "📌 固定" }}
    </button>
    <div class="drag-area" data-tauri-drag-region></div>
    <button class="ctrl-btn close-btn" @click="$emit('close')" title="关闭">
      ✕
    </button>
  </div>
</template>

<style scoped>
.control-bar {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 8px;
  background: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(10px);
  border-top: 1px solid rgba(51, 65, 85, 0.5);
  gap: 8px;
  -webkit-app-region: drag;
}

.ctrl-btn {
  padding: 4px 12px;
  border-radius: 6px;
  border: none;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  -webkit-app-region: no-drag;
  color: #94a3b8;
  background: rgba(51, 65, 85, 0.5);
}

.ctrl-btn:hover {
  background: rgba(51, 65, 85, 0.8);
  color: #f1f5f9;
}

.pin-btn:hover {
  background: rgba(99, 102, 241, 0.3);
  color: #818cf8;
}

.close-btn:hover {
  background: rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.drag-area {
  flex: 1;
  height: 100%;
  cursor: grab;
}
</style>
