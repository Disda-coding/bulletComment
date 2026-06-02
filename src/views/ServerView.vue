<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

const serverAddress = ref("");
const port = ref(9090);
const isRunning = ref(false);
const onlineCount = ref(0);
const localIp = ref("");
const copied = ref(false);
const errorMsg = ref("");
const danmakuSpeed = ref(1.0);
let unlisteners: UnlistenFn[] = [];

async function getIp() {
  try {
    localIp.value = await invoke<string>("get_local_ip");
  } catch {
    localIp.value = "127.0.0.1";
  }
}

async function startServer() {
  errorMsg.value = "";
  try {
    serverAddress.value = await invoke<string>("start_server", { port: port.value });
    isRunning.value = true;
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function stopServer() {
  try {
    await invoke("stop_server");
    isRunning.value = false;
    serverAddress.value = "";
    onlineCount.value = 0;
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function copyAddress() {
  if (!serverAddress.value) return;
  try {
    await navigator.clipboard.writeText(serverAddress.value);
    copied.value = true;
    setTimeout(() => (copied.value = false), 2000);
  } catch {}
}

async function setSpeed(val: number) {
  danmakuSpeed.value = val;
  try {
    await invoke("set_danmaku_speed", { speed: val });
  } catch {}
}

onMounted(async () => {
  await getIp();

  const u1 = await listen<number>("online-count", (event) => {
    onlineCount.value = event.payload;
  });
  const u2 = await listen<string>("server-started", (event) => {
    serverAddress.value = event.payload;
    isRunning.value = true;
  });
  const u3 = await listen("server-stopped", () => {
    isRunning.value = false;
    serverAddress.value = "";
    onlineCount.value = 0;
  });

  unlisteners = [u1, u2, u3];
});

onUnmounted(() => {
  unlisteners.forEach((u) => u());
});
</script>

<template>
  <div class="server-view">
    <div class="scroll-area">
      <div class="header">
        <h1>弹幕系统</h1>
        <p class="subtitle">课堂内网弹幕 - 教师端</p>
      </div>

      <div class="card" v-if="!isRunning">
      <label class="label">端口号</label>
      <input
        v-model.number="port"
        type="number"
        class="input"
        min="1024"
        max="65535"
      />
      <button class="btn btn-primary" @click="startServer">
        启动服务器
      </button>
      <p class="hint">启动后学生可通过浏览器访问发送弹幕</p>
    </div>

    <div class="card" v-else>
      <div class="success-badge">服务器运行中</div>

      <div class="address-box">
        <span class="address-text">{{ serverAddress }}</span>
        <button class="btn-copy" @click="copyAddress">
          {{ copied ? "已复制" : "复制" }}
        </button>
      </div>

      <div class="stats">
        <div class="stat-item">
          <span class="stat-value">{{ onlineCount }}</span>
          <span class="stat-label">在线人数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ port }}</span>
          <span class="stat-label">端口</span>
        </div>
      </div>

      <div class="speed-section">
        <label class="label">弹幕速度</label>
        <div class="speed-btns">
          <button
            class="speed-btn"
            :class="{ active: danmakuSpeed === 0.5 }"
            @click="setSpeed(0.5)"
          >慢速</button>
          <button
            class="speed-btn"
            :class="{ active: danmakuSpeed === 1.0 }"
            @click="setSpeed(1.0)"
          >正常</button>
          <button
            class="speed-btn"
            :class="{ active: danmakuSpeed === 2.0 }"
            @click="setSpeed(2.0)"
          >快速</button>
        </div>
      </div>

      <div class="tip">
        学生在浏览器打开上方地址即可发送弹幕
      </div>

      <button class="btn btn-danger btn-stop" @click="stopServer">
        停止服务器
      </button>
    </div>

    <p class="error" v-if="errorMsg">{{ errorMsg }}</p>
    </div>
  </div>
</template>

<style scoped>
.server-view {
  padding: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #0f172a;
  color: #f1f5f9;
  overflow: hidden;
}

.scroll-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px 20px 24px;
  -webkit-overflow-scrolling: touch;
}

.header {
  text-align: center;
  margin-bottom: 24px;
}

.header h1 {
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(135deg, #818cf8, #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.subtitle {
  color: #94a3b8;
  font-size: 14px;
  margin-top: 4px;
}

.card {
  background: #1e293b;
  border-radius: 16px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.label {
  font-size: 13px;
  color: #94a3b8;
  font-weight: 500;
}

.input {
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid #334155;
  background: #0f172a;
  color: #f1f5f9;
  font-size: 16px;
  outline: none;
  transition: border-color 0.2s;
}

.input:focus {
  border-color: #6366f1;
}

.btn {
  padding: 12px 20px;
  border-radius: 12px;
  border: none;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:active {
  transform: scale(0.97);
}

.btn-primary {
  background: #6366f1;
  color: white;
}

.btn-primary:hover {
  background: #818cf8;
}

.btn-danger {
  background: #dc2626;
  color: white;
}

.btn-danger:hover {
  background: #ef4444;
}

.hint {
  font-size: 13px;
  color: #64748b;
  text-align: center;
}

.success-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 20px;
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
  font-size: 14px;
  font-weight: 500;
  align-self: flex-start;
}

.success-badge::before {
  content: "";
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #22c55e;
}

.address-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-radius: 12px;
  background: #0f172a;
  border: 1px solid #334155;
}

.address-text {
  flex: 1;
  font-size: 15px;
  font-family: "SF Mono", "Fira Code", monospace;
  color: #e2e8f0;
  word-break: break-all;
}

.btn-copy {
  padding: 6px 14px;
  border-radius: 8px;
  border: 1px solid #334155;
  background: #1e293b;
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-copy:hover {
  background: #334155;
  color: #f1f5f9;
}

.stats {
  display: flex;
  gap: 12px;
}

.stat-item {
  flex: 1;
  text-align: center;
  padding: 12px;
  border-radius: 12px;
  background: #0f172a;
}

.stat-value {
  display: block;
  font-size: 24px;
  font-weight: 700;
  color: #818cf8;
}

.stat-label {
  font-size: 12px;
  color: #64748b;
  margin-top: 2px;
}

.speed-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.speed-btns {
  display: flex;
  gap: 8px;
}

.speed-btn {
  flex: 1;
  padding: 8px 0;
  border-radius: 8px;
  border: 1px solid #334155;
  background: #0f172a;
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.speed-btn:hover {
  border-color: #6366f1;
  color: #c7d2fe;
}

.speed-btn.active {
  background: rgba(99, 102, 241, 0.2);
  border-color: #6366f1;
  color: #a5b4fc;
}

.tip {
  font-size: 13px;
  color: #64748b;
  text-align: center;
  padding: 8px;
  border-radius: 8px;
  background: rgba(99, 102, 241, 0.08);
}

.error {
  color: #ef4444;
  text-align: center;
  margin-top: 12px;
  font-size: 14px;
}
</style>
