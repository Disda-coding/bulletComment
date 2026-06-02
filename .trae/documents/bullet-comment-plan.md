# 内网弹幕系统 - 技术方案与实施计划

## 项目概述

跨平台内网弹幕系统，用于课堂场景。教师端运行桌面应用显示弹幕，学生端通过浏览器发送弹幕（适配手机/平板/电脑）。

## 技术栈

| 层级 | 技术选型 | 理由 |
|------|---------|------|
| 桌面框架 | **Tauri 2.0** | 轻量（~10MB）、原生窗口管理 API 完善、跨平台 |
| 桌面前端 | **Vue 3 + TypeScript + Vite** | Tauri 官方模板、开发体验好 |
| 后端 | **Rust**（Tauri 内置） | 高性能、与 Tauri 深度集成 |
| Web 客户端 | **纯 HTML/CSS/JS**（单页面） | 无需框架、手机浏览器兼容、极轻量 |
| 通信协议 | **WebSocket**（tokio-tungstenite） | 实时双向通信、内网低延迟 |
| HTTP 服务 | **axum** | Rust 生态最成熟的 Web 框架、服务 Web 客户端页面 |
| 包管理 | **pnpm** | 快速、磁盘高效 |

### 架构变更说明

客户端从桌面应用改为 **Web 页面**：
- 学生无需安装任何软件，浏览器打开即用
- 适配手机、平板、笔记本等各种设备
- Tauri 应用内嵌 HTTP 服务器，同时提供 Web 客户端页面
- 学生访问 `http://教师IP:端口` 即可发送弹幕

## 架构设计

### 整体架构

```
┌──────────────────────────────────────────────────────┐
│            Tauri 桌面应用（教师端）                     │
│                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │ WebSocket    │  │ HTTP Server  │  │ 弹幕显示    │ │
│  │ Server       │  │ (axum)       │  │ 窗口        │ │
│  │ :9090/ws     │  │ :9090        │  │ (透明置顶)  │ │
│  └──────┬───────┘  └──────┬───────┘  └──────▲─────┘ │
│         │                 │                  │        │
│         │    提供 Web 客户端页面              │        │
│         │                 │          弹幕数据推送      │
└─────────┼─────────────────┼──────────────────┼────────┘
          │                 │                  │
          ▼                 ▼                  │
   ┌──────────┐      ┌──────────┐             │
   │ 学生手机  │      │ 学生电脑  │             │
   │ 浏览器    │      │ 浏览器    │             │
   └──────────┘      └──────────┘             │
          │                 │                  │
          └────WebSocket────┘                  │
               发送弹幕 ────────────────────────┘
```

### 通信流程

1. 教师启动 Tauri 应用 → 选择服务器模式 → 自动启动 HTTP + WebSocket 服务器
2. 界面显示连接地址（如 `http://192.168.1.100:9090`）
3. 学生浏览器打开该地址 → 加载 Web 客户端页面
4. 页面自动建立 WebSocket 连接
5. 学生发送弹幕 → WebSocket → 服务器广播 → 弹幕显示窗口渲染

### 消息协议（JSON）

```json
{
  "type": "danmaku",
  "content": "老师讲得好！",
  "color": "#FFFFFF",
  "size": 24
}
```

```json
{
  "type": "system",
  "action": "connected",
  "content": "已连接到服务器"
}
```

```json
{
  "type": "online_count",
  "count": 42
}
```

## 核心功能设计

### 1. 教师端 - 服务器模式（Tauri 桌面应用）

#### 启动界面

- 显示本机 IP 地址和端口（如 `http://192.168.1.100:9090`）
- 可配置端口号
- 显示当前在线人数
- 一键复制连接地址
- 启动/停止服务器按钮

#### 弹幕显示窗口

- **半透明**: `transparent: true` + CSS `background: rgba(0,0,0,0.3)`
- **窗口效果**: macOS 使用 `hudWindow`，Windows 使用 `acrylic`
- **置顶**: `always_on_top: true`
- **无标题栏**: `decorations: false`（自定义拖拽区域）
- **弹幕动画**: CSS animation 从右向左滚动，多轨道避免重叠

#### 固定/移动模式

**固定模式（鼠标穿透）**:
- 调用 `window.set_ignore_cursor_events(true)` 实现鼠标穿透
- 窗口不影响教师操作其他软件（如 PPT、代码编辑器）
- 通过**系统托盘**或**键盘快捷键**切换回移动模式
- 也可通过窗口底部的**小控制条**（独立小窗口）切换

**移动模式（可交互）**:
- `set_ignore_cursor_events(false)` 恢复交互
- 自定义标题栏区域 `data-tauri-drag-region` 支持拖拽
- 窗口边缘支持调整大小

**特定区域触发方案**（双窗口方案）:

```
┌──────────────────────────────────┐
│                                  │  ← 主窗口（固定时鼠标穿透）
│        弹幕滚动区域              │
│                                  │
│                                  │
├──────────────────────────────────┤
│  📌 固定/解除  │  ✕ 关闭        │  ← 控制条窗口（始终可交互）
└──────────────────────────────────┘
```

- 主窗口：弹幕显示，固定时 `ignore_cursor_events: true`
- 控制条窗口：紧贴主窗口底部，始终可交互，包含固定/关闭按钮
- 解除固定后，控制条融入主窗口，整体可拖拽/调整大小

#### 系统托盘

- 固定/移动切换
- 退出应用
- 显示服务器 IP 和端口信息

### 2. 学生端 - Web 客户端（浏览器页面）

#### 功能

- 自动连接 WebSocket（页面加载后）
- 弹幕输入框 + 发送按钮
- 颜色选择（预设几种颜色）
- 连接状态指示
- 在线人数显示
- 响应式设计：适配手机竖屏、平板、桌面浏览器

#### UI 设计

```
┌─────────────────────────┐
│     🎯 课堂弹幕          │
│     在线: 42人            │
│                          │
│  ┌────────────────────┐  │
│  │ 输入弹幕...         │  │
│  └────────────────────┘  │
│  ⚪白 🟡黄 🔵蓝 🟢绿 🔴红 │
│                          │
│      [ 发送 ]            │
│                          │
│  ── 最近弹幕 ──          │
│  同学A: 老师好！          │
│  同学B: 听懂了            │
│  同学C: 能再讲一遍吗      │
└─────────────────────────┘
```

- 手机端：全屏输入界面，底部固定输入框
- 桌面端：居中卡片式布局

### 3. 后端服务（Rust）

#### HTTP 服务器（axum）

- `GET /` → 返回 Web 客户端 HTML 页面
- `GET /ws` → WebSocket 升级端点
- 静态资源内嵌到 Tauri 应用中（使用 `include_str!` 或 `rust-embed`）

#### WebSocket 服务器

- 使用 `tokio-tungstenite` + `axum` 的 WebSocket 支持
- 监听 `0.0.0.0:9090`（可配置端口）
- 使用 `tokio::sync::broadcast` 通道实现消息广播
- 收到客户端弹幕 → 广播给所有连接的客户端 + 通过 Tauri 事件推送到弹幕显示窗口
- 自动获取本机 IP 显示在界面上

## 项目结构

```
bulletComment/
├── src/                          # Vue 前端（教师端桌面界面）
│   ├── App.vue
│   ├── main.ts
│   ├── views/
│   │   ├── ServerView.vue        # 服务器模式主页（控制面板）
│   │   └── DanmakuOverlay.vue    # 弹幕显示覆盖层
│   ├── components/
│   │   ├── ControlBar.vue        # 控制条组件
│   │   └── ServerInfo.vue        # 服务器信息显示
│   ├── composables/
│   │   └── useWindowManager.ts   # 窗口管理逻辑
│   ├── types/
│   │   └── index.ts
│   └── styles/
│       └── danmaku.css
├── web-client/                   # Web 客户端（学生端浏览器页面）
│   ├── index.html                # 单页面（内嵌 CSS + JS）
│   └── (无需构建工具，纯静态文件)
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # Tauri 入口
│   │   ├── commands.rs           # Tauri 命令
│   │   ├── server.rs             # HTTP + WebSocket 服务器（axum）
│   │   └── tray.rs               # 系统托盘
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
│       └── default.json
├── package.json
├── vite.config.ts
├── tsconfig.json
└── .gitignore
```

### Web 客户端说明

`web-client/index.html` 是一个纯静态单页面：
- 无需 Vue/React 等框架，纯 HTML + CSS + 原生 JS
- 通过 `rust-embed` 或 `include_str!` 嵌入到 Rust 二进制中
- axum 服务器直接返回该 HTML，学生无需下载额外资源
- 所有样式和脚本内联，一个文件搞定，加载极快

## 实施步骤

### 阶段 1: 项目初始化

1. 使用 `pnpm create tauri-app` 初始化 Tauri 2.0 + Vue 3 + TypeScript 项目
2. 配置 `tauri.conf.json`：窗口透明、置顶、无装饰、权限
3. 配置 `capabilities/default.json`：窗口管理等权限
4. 添加 Rust 依赖：`axum`、`tokio`、`tokio-tungstenite`、`serde`、`serde_json`、`rust-embed`、`local-ip-address`
5. 初始化 Git 仓库，关联远程 `git@github.com:Disda-coding/bulletComment.git`

### 阶段 2: HTTP + WebSocket 服务器

1. 实现 `server.rs`：基于 axum 的 HTTP + WebSocket 服务器
   - `GET /` 返回 Web 客户端页面
   - `GET /ws` WebSocket 端点
   - 广播通道管理
   - 连接/断开事件处理
2. 实现 `commands.rs`：Tauri 命令
   - `start_server(port)` → 启动服务器
   - `stop_server()` → 停止服务器
   - `get_local_ip()` → 获取本机 IP
3. Web 客户端页面通过 `rust-embed` 嵌入

### 阶段 3: Web 客户端页面

1. 创建 `web-client/index.html`：
   - 响应式布局（手机/平板/桌面）
   - WebSocket 连接逻辑
   - 弹幕输入和发送
   - 颜色选择
   - 连接状态和在线人数
   - 最近弹幕列表
2. 移动端优化：底部固定输入框、触摸友好

### 阶段 4: 教师端控制面板

1. 创建 `ServerView.vue`：
   - 服务器 IP 和端口显示
   - 一键复制连接地址
   - 在线人数
   - 启动/停止按钮
   - 端口配置
2. 弹幕数据通过 Tauri 事件从 Rust 推送到前端

### 阶段 5: 弹幕显示窗口

1. 创建弹幕显示窗口（透明、置顶、无装饰）
2. 实现 `DanmakuOverlay.vue`：
   - 弹幕轨道管理（避免重叠）
   - CSS animation 从右向左滚动
   - 支持不同颜色和大小
   - DOM 节点回收（动画结束后移除）
3. 服务器收到弹幕 → Tauri 事件 → 前端渲染

### 阶段 6: 窗口管理（固定/移动）

1. 实现控制条窗口（独立小窗口）：
   - 固定/解除按钮
   - 关闭按钮
   - 窗口位置跟随主窗口
2. 固定模式：主窗口 `set_ignore_cursor_events(true)`
3. 移动模式：自定义拖拽区域 `data-tauri-drag-region`
4. 系统托盘：固定/移动切换、退出
5. 键盘快捷键（如 `Cmd/Ctrl+Shift+D`）切换固定状态

### 阶段 7: 打包与分发

1. macOS: 构建 `.dmg` 安装包
2. Windows: 构建 `.msi` / `.exe` 安装包
3. 内网测试：多设备连接、弹幕并发
4. 推送到 GitHub

## 关键技术要点

### 窗口透明配置

```json
// tauri.conf.json
{
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "label": "main",
        "url": "/server",
        "width": 400,
        "height": 300
      }
    ]
  }
}
```

弹幕显示窗口在 Rust 侧动态创建：

```rust
WebviewWindowBuilder::new(app, "danmaku", WebviewUrl::App("/danmaku".into()))
    .title("弹幕")
    .inner_size(1200.0, 300.0)
    .position(100.0, 50.0)
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .effects(EffectsBuilder::new()
        .effects(vec![Effect::HudWindow, Effect::Acrylic])
        .radius(12.0)
        .build())
    .build()?;
```

### 鼠标穿透（固定模式）

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

// 固定 - 鼠标穿透
await appWindow.setIgnoreCursorEvents(true);

// 解除固定 - 恢复交互
await appWindow.setIgnoreCursorEvents(false);
```

### axum HTTP + WebSocket 服务器

```rust
use axum::{Router, routing::get, extract::ws};
use tokio::sync::broadcast;

async fn ws_handler(
    ws: ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: ws::WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // 发送任务：广播通道 → 客户端
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            sender.send(ws::Message::Text(msg.into())).await.ok();
        }
    });

    // 接收任务：客户端 → 广播通道
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let ws::Message::Text(text) = msg {
                state.tx.send(text.to_string()).ok();
                // 同时通过 Tauri 事件推送到弹幕窗口
                state.app_handle.emit("danmaku", text).ok();
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
```

### Web 客户端核心逻辑

```javascript
const ws = new WebSocket(`ws://${location.host}/ws`);

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.type === 'danmaku') {
        appendToRecentList(msg);
    } else if (msg.type === 'online_count') {
        updateOnlineCount(msg.count);
    }
};

function sendDanmaku(content, color) {
    ws.send(JSON.stringify({
        type: 'danmaku',
        content,
        color,
        size: 24
    }));
}
```

### 弹幕动画（CSS）

```css
@keyframes danmaku-scroll {
    from { transform: translateX(100%); }
    to { transform: translateX(-100vw); }
}

.danmaku-item {
    position: absolute;
    white-space: nowrap;
    animation: danmaku-scroll 8s linear forwards;
    pointer-events: none;
}
```

## 风险与注意事项

1. **macOS 透明窗口**: 需要启用 `macOSPrivateApi: true`，App Store 审核可能受限（内网使用无影响）
2. **Windows acrylic 效果**: 仅 Windows 10 1803+ 支持，旧版本回退为普通半透明
3. **防火墙**: 服务器端需开放 HTTP/WebSocket 端口，课堂环境可能需要配置防火墙
4. **多窗口定位**: 控制条窗口跟随主窗口移动需要精确的位置计算
5. **性能**: 大量弹幕时注意 DOM 节点回收，避免内存泄漏
6. **Web 客户端兼容性**: 使用原生 WebSocket API，主流浏览器均支持（IE 除外）
7. **内网发现**: 可考虑后续增加 mDNS/Bonjour 自动发现，当前版本手动输入 IP 即可
