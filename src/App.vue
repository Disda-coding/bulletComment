<script setup lang="ts">
import { computed } from "vue";
import ServerView from "./views/ServerView.vue";
import DanmakuOverlay from "./views/DanmakuOverlay.vue";

const route = computed(() => {
  return window.location.hash.slice(1) || "/server";
});
</script>

<template>
  <DanmakuOverlay v-if="route === '/danmaku'" />
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
