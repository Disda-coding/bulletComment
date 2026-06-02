<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";

const isPinned = ref(false);
const speed = ref(1.0);
let unlisteners: UnlistenFn[] = [];
let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;
let winStartPos: { x: number; y: number } | null = null;

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

async function onDragStart(e: MouseEvent | TouchEvent) {
  const appWindow = getCurrentWindow();
  const pos = await appWindow.position();
  winStartPos = { x: pos.x, y: pos.y };

  if (e instanceof MouseEvent) {
    dragStartX = e.screenX;
    dragStartY = e.screenY;
  } else {
    const t = e.touches[0];
    dragStartX = t.screenX;
    dragStartY = t.screenY;
  }
  isDragging = true;
  if (e.preventDefault) e.preventDefault();
}

function onDragMove(e: MouseEvent | TouchEvent) {
  if (!isDragging || !winStartPos) return;

  let clientX: number, clientY: number;
  if (e instanceof MouseEvent) {
    clientX = e.screenX;
    clientY = e.screenY;
  } else {
    const t = e.touches[0];
    clientX = t.screenX;
    clientY = t.screenY;
  }

  const dx = clientX - dragStartX;
  const dy = clientY - dragStartY;
  const newX = winStartPos.x + dx;
  const newY = winStartPos.y + dy;

  getCurrentWindow().setPosition(new PhysicalPosition(newX, newY)).catch(() => {});
}

function onDragEnd() {
  isDragging = false;
  winStartPos = null;
}

onMounted(async () => {
  document.addEventListener("mousemove", onDragMove);
  document.addEventListener("mouseup", onDragEnd);
  document.addEventListener("touchmove", onDragMove, { passive: false });
  document.addEventListener("touchend", onDragEnd);

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
  document.removeEventListener("mousemove", onDragMove);
  document.removeEventListener("mouseup", onDragEnd);
  document.removeEventListener("touchmove", onDragMove);
  document.removeEventListener("touchend", onDragEnd);
});
</script>

<template>
  <div class="control-bar">
    <div class="drag-handle" @mousedown="onDragStart" @touchstart.prevent="onDragStart">⋮⋮</div>
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
  cursor: default;
}

.drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 100%;
  font-size: 10px;
  color: rgba(148, 163, 184, 0.5);
  letter-spacing: -1px;
  cursor: grab;
  user-select: none;
  padding-right: 2px;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  margin-right: 2px;
  line-height: 1;
  flex-shrink: 0;
}

.drag-handle:active {
  cursor: grabbing;
  color: rgba(148, 163, 184, 0.9);
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
  flex-shrink: 0;
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
