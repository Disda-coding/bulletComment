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
  duration: number;
}

const danmakuList = ref<DanmakuItem[]>([]);
let nextId = 0;
let unlisteners: UnlistenFn[] = [];
const trackLastTime: number[] = [];
const TRACK_COUNT = 10;
const DEFAULT_DURATION = 8000;
const currentSpeed = ref(1.0);
const MAX_DANMAKU = 200;

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

function removeDanmaku(id: number) {
  const idx = danmakuList.value.findIndex((d) => d.id === id);
  if (idx !== -1) {
    danmakuList.value.splice(idx, 1);
  }
}

function addDanmaku(raw: string) {
  try {
    const msg: DanmakuMessage = JSON.parse(raw);
    if (msg.type !== "danmaku") return;

    while (danmakuList.value.length >= MAX_DANMAKU) {
      danmakuList.value.shift();
    }

    const track = getAvailableTrack();
    const duration = DEFAULT_DURATION / currentSpeed.value;
    const item: DanmakuItem = {
      id: nextId++,
      content: msg.content,
      color: msg.color || "#FFFFFF",
      size: msg.size || 24,
      track,
      duration,
    };
    danmakuList.value.push(item);

    setTimeout(() => {
      removeDanmaku(item.id);
    }, duration + 100);
  } catch (e) {
    console.error("Failed to parse danmaku:", e);
  }
}

function onDanmakuEnd(e: Event) {
  const target = e.target as HTMLElement;
  if (target && target.dataset.id) {
    removeDanmaku(parseInt(target.dataset.id));
  }
}

onMounted(async () => {
  const u1 = await listen<string>("danmaku", (event) => {
    addDanmaku(event.payload);
  });
  const u2 = await listen<number>("danmaku-speed", (event) => {
    currentSpeed.value = event.payload;
  });

  unlisteners = [u1, u2];
});

onUnmounted(() => {
  unlisteners.forEach((u) => u());
  danmakuList.value = [];
});
</script>

<template>
  <div class="danmaku-overlay">
    <div class="danmaku-container" @animationend="onDanmakuEnd">
      <div
        v-for="item in danmakuList"
        :key="item.id"
        :data-id="item.id"
        class="danmaku-item"
        :style="{
          color: item.color,
          fontSize: item.size + 'px',
          top: item.track * 38 + 'px',
          animationDuration: item.duration + 'ms',
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

.danmaku-container {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.danmaku-item {
  position: absolute;
  white-space: nowrap;
  right: 0;
  animation: danmaku-scroll linear forwards;
  text-shadow:
    1px 1px 2px rgba(0, 0, 0, 0.9),
    -1px -1px 2px rgba(0, 0, 0, 0.9),
    1px -1px 2px rgba(0, 0, 0, 0.9),
    -1px 1px 2px rgba(0, 0, 0, 0.9),
    0 0 8px rgba(0, 0, 0, 0.6);
  font-weight: 600;
  pointer-events: none;
  opacity: 1;
}

@keyframes danmaku-scroll {
  0% {
    transform: translateX(100%);
    opacity: 1;
  }
  95% {
    opacity: 1;
  }
  100% {
    transform: translateX(calc(-100vw - 100%));
    opacity: 0;
  }
}
</style>
