<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import ServerView from "./views/ServerView.vue";
import DanmakuOverlay from "./views/DanmakuOverlay.vue";
import ControlBar from "./views/ControlBar.vue";

const currentRoute = ref(window.location.hash.slice(1) || "/server");

function onHashChange() {
  currentRoute.value = window.location.hash.slice(1) || "/server";
}

onMounted(() => {
  window.addEventListener("hashchange", onHashChange);
});

onUnmounted(() => {
  window.removeEventListener("hashchange", onHashChange);
});
</script>

<template>
  <DanmakuOverlay v-if="currentRoute === '/danmaku'" />
  <ControlBar v-else-if="currentRoute === '/control-bar'" />
  <ServerView v-else />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "PingFang SC", "Microsoft YaHei", sans-serif;
}

body {
  background: transparent;
}
</style>
