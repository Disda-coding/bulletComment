<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";

const isPinned = ref(false);
const speed = ref(1.0);
let unlisteners: UnlistenFn[] = [];

async function togglePin() {
  try {
    isPinned.value = await invoke<boolean>("toggle_pin");
  } catch (e) {
    console.error("Toggle pin failed:", e);
  }
}

async function changeSpeed(val: number) {
  speed.value = val;
  try {
    await invoke("set_danmaku_speed", { speed: val });
  } catch (e) {
    console.error("Set speed failed:", e);
  }
}

onMounted(async () => {
  const u1 = await listen<boolean>("pin-state-changed", (event) => {
    isPinned.value = event.payload;
  });
  const u2 = await listen("toggle-pin", () => {
    togglePin();
  });
  unlisteners = [u1, u2];
});

onUnmounted(() => {
  unlisteners.forEach((u) => u());
});
</script>

<template>
  <div class="control-bar">
    <button class="ctrl-btn pin-btn" @click="togglePin">
      {{ isPinned ? "🔓" : "📌" }}
    </button>
    <button
      class="ctrl-btn speed-btn"
      :class="{ active: speed === 0.5 }"
      @click="changeSpeed(0.5)"
    >慢</button>
    <button
      class="ctrl-btn speed-btn"
      :class="{ active: speed === 1.0 }"
      @click="changeSpeed(1.0)"
    >中</button>
    <button
      class="ctrl-btn speed-btn"
      :class="{ active: speed === 2.0 }"
      @click="changeSpeed(2.0)"
    >快</button>
  </div>
</template>

<style scoped>
.control-bar {
  display: inline-flex;
  align-items: center;
  height: 100%;
  padding: 0 6px;
  background: rgba(15, 23, 42, 0.35);
  backdrop-filter: blur(6px);
  border-radius: 6px;
  gap: 4px;
}

.ctrl-btn {
  padding: 3px 8px;
  border-radius: 4px;
  border: none;
  font-size: 12px;
  cursor: pointer;
  color: rgba(148, 163, 184, 0.8);
  background: rgba(51, 65, 85, 0.3);
  transition: all 0.15s;
  white-space: nowrap;
  line-height: 1;
}

.ctrl-btn:hover {
  background: rgba(99, 102, 241, 0.35);
  color: #c7d2fe;
}

.speed-btn.active {
  color: #a5b4fc;
  background: rgba(99, 102, 241, 0.4);
}
</style>
