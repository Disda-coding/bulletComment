# 修复代码错误 + 编写架构设计文档

## 一、代码修复计划

### 1. [BUG] vote_close 不广播更新给客户端
**文件**: `src-tauri/src/server.rs` 第 304-311 行
**问题**: 收到 `vote_close` 后只修改本地状态，没有广播 `vote_update`，客户端 UI 不会更新投票状态
**修复**: 在 `vote.closed = true` 之后，广播 `vote_update` 消息

### 2. [BUG] StdMutex `.unwrap()` 在异步任务中可能 panic
**文件**: `src-tauri/src/server.rs` 第 147 行、第 322 行
**问题**: 如果锁被中毒，`.unwrap()` 会导致 panic
**修复**: 改为 `.lock().unwrap_or_else(|e| e.into_inner())` 或统一使用 `if let Ok(...)` 模式

### 3. [清理] 删除死代码 `src/composables/useWindowManager.ts`
**问题**: 从未被任何组件引用，且使用了不存在的 API (`getAllWebviewWindows`)
**修复**: 删除整个文件

### 4. [清理] 删除死代码 `src/components/ControlBar.vue`
**问题**: 从未被导入使用，与 `src/views/ControlBar.vue` 重复且功能不同
**修复**: 删除整个文件

### 5. [修复] `ServerStatus` 接口与后端不匹配
**文件**: `src/types/index.ts`
**问题**: `ServerStatus` 定义了 4 个字段但后端只返回 `bool`，且从未被使用
**修复**: 删除未使用的 `ServerStatus` 接口

### 6. [修复] `substr()` 已废弃
**文件**: `web-client/index.html` 第 1137 行
**修复**: `substr(2, 6)` → `slice(2, 8)`

### 7. [修复] `clientIp` 变量声明位置靠后
**文件**: `web-client/index.html`
**修复**: 将 `let clientIp = ''` 移到其他变量声明区域（第 425 行附近）

### 8. [修复] tauri.conf.json 使用非官方 schema URL
**文件**: `src-tauri/tauri.conf.json` 第 2 行
**修复**: 移除 `$schema` 字段或替换为 Tauri 官方 schema

### 9. [修复] `@tauri-apps/api` 放在 devDependencies
**文件**: `package.json`
**修复**: 将 `@tauri-apps/api` 和 `@tauri-apps/plugin-opener` 从 devDependencies 移到 dependencies

---

## 二、架构设计文档

在 `doc/architecture.md` 中编写，包含以下章节：

1. **项目概述** — 定位、目标用户、核心场景
2. **系统架构** — 整体架构图（教师端 Tauri 桌面应用 + 学生端 Web 浏览器客户端）
3. **技术选型** — 每个技术选择的理由
   - Tauri 2.0 vs Electron
   - Rust + Axum vs Node.js
   - Vue 3 + TypeScript
   - WebSocket 双通道（文本 + 二进制）
   - VP8/WebM 编码选择
   - 自签名 HTTPS（rcgen）
   - rust-embed 嵌入 Web 客户端
4. **模块详解**
   - 教师端控制面板（ServerView.vue）
   - 弹幕显示窗口（DanmakuOverlay.vue）— 轨道分配、动画、速度控制
   - 控制条窗口（ControlBar.vue）— 拖拽、固定/移动
   - 学生端 Web 客户端（web-client/index.html）— 单文件架构
   - Rust 服务器（server.rs）— WebSocket 处理、广播、直播中继
   - Tauri 命令层（commands.rs）— 窗口管理、服务生命周期
   - 系统托盘（tray.rs）
5. **通信协议** — WebSocket 消息类型定义
   - 文本消息：danmaku, system, online_count, live_start, live_stop, vote_create, vote_cast, vote_update, vote_close
   - 二进制消息：视频帧广播
6. **直播链路** — 从 getDisplayMedia 到 MediaSource 的完整数据流
   - 推流端：getDisplayMedia → MediaRecorder → WebSocket Binary
   - 服务端：Binary 帧接收 → Bytes 零拷贝 → broadcast channel → 所有客户端
   - 拉流端：WebSocket Binary → SourceBuffer → MediaSource → Video
   - 初始化片段缓存机制
7. **投票系统** — 数据结构、消息流、防重复投票
8. **弹幕历史** — 新连接时发送最近 100 条
9. **性能优化** — Bytes 零拷贝、码率/分辨率调节、缓冲区管理
10. **安全考量** — 自签名证书、XSS 防护、输入验证
11. **浏览器兼容性** — 开启直播 vs 观看直播的版本要求

---

## 三、验证步骤

1. `cd src-tauri && cargo check` — Rust 编译无错误
2. `npm run build` — 前端构建无错误
3. 确认删除的文件没有被任何地方引用
4. 确认架构文档内容与代码一致
