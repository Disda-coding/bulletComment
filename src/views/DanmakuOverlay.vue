<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { DanmakuMessage } from "../types";

interface DanmakuItem {
  id: number;
  content: string;
  color: string;
  size: number;
  track: number;
}

const danmakuList = ref<DanmakuItem[]>([]);
const isPinned = ref(false);
let nextId = 0;
let unlisteners: UnlistenFn[] = [];
const trackLastTime: number[] = [];
const TRACK_COUNT = 12;
const DANMAKU_DURATION = 8000;

function getAvailableTrack(): number {
  const now = Date.now();
  for (let i = 0; i < TRACK_COUNT; i++) {
    if (!trackLastTime[i] || now - trackLastTime[i] > 2000) {
      trackLastTime[i] = now;
      return i;
    }
  }
  const track = nextId % TRACK_COUNT;
  trackLastTime[track] = now;
  return track;
}

function addDanmaku(raw: string) {
  try {
    const msg: DanmakuMessage = JSON.parse(raw);
    if (msg.type !== "danmaku") return;

    const track = getAvailableTrack();
    const item: DanmakuItem = {
      id: nextId++,
      content: msg.content,
      color: msg.color || "#FFFFFF",
      size: msg.size || 24,
      track,
    };
    danmakuList.value.push(item);

    setTimeout(() => {
      danmakuList.value = danmakuList.value.filter((d) => d.id !== item.id);
    }, DANMAKU_DURATION);
  } catch {}
}

onMounted(async () => {
  const u1 = await listen<string>("danmaku", (event) => {
    addDanmaku(event.payload);
  });
  const u2 = await listen("toggle-pin", () => {
    togglePin();
  });

  unlisteners = [u1, u2];
});

onUnmounted(() => {
  unlisteners.forEach((u) => u());
});

async function togglePin() {
  isPinned.value = !isPinned.value;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const appWindow = getCurrentWindow();
  await appWindow.setIgnoreCursorEvents(isPinned.value);
}
</script>

<template>
  <div class="danmaku-overlay" :class="{ pinned: isPinned }">
    <div class="danmaku-container">
      <div
        v-for="item in danmakuList"
        :key="item.id"
        class="danmaku-item"
        :style="{
          color: item.color,
          fontSize: item.size + 'px',
          top: item.track * 36 + 'px',
          animationDuration: DANMAKU_DURATION + 'ms',
        }"
      >
        {{ item.content }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.danmaku-overlay {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: transparent;
}

.danmaku-overlay.pinned {
  pointer-events: none;
}

.danmaku-container {
  width: 100%;
  height: 100%;
  position: relative;
}

.danmaku-item {
  position: absolute;
  white-space: nowrap;
  right: 0;
  animation: danmaku-scroll linear forwards;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5);
  font-weight: 500;
  pointer-events: none;
}

@keyframes danmaku-scroll {
  from {
    transform: translateX(100%);
    right: 0;
  }
  to {
    transform: translateX(-100vw);
    right: 0;
  }
}
</style>
