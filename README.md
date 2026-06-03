# 课堂弹幕系统 (BulletComment)

跨平台内网弹幕 + 直播系统，支持 macOS / Windows / Linux。

## 功能特性

- **弹幕发送**：学生通过浏览器发送弹幕（支持手机/平板/电脑）
- **弹幕显示**：教师端半透明置顶窗口实时显示弹幕
- **固定/移动**：弹幕窗口可固定（鼠标穿透）或自由拖动
- **速度调节**：弹幕滚动速度可调（慢速/正常/快速）
- **屏幕直播**：教师共享屏幕直播给学生观看
- **直播参数调节**：开播前可调节码率（300kbps~3Mbps）和分辨率（480p/540p/720p/1080p）
- **播放器控件**：全屏、窗口全屏、小窗、画中画
- **投票功能**：发起投票和参与投票，实时显示投票结果
- **历史弹幕**：新加入的用户可以看到最近 100 条弹幕记录
- **日志记录**：自动保存弹幕日志到 `logs/` 目录
- **自动打开浏览器**：启动服务器后自动打开浏览器访问客户端页面

## 浏览器要求

### 开启直播（教师端）

| 功能 | 最低浏览器版本 |
|------|---------------|
| `getDisplayMedia` 屏幕共享 | Chrome 72+, Edge 79+, Safari 13+ |
| `MediaRecorder` VP8 编码 | Chrome 47+, Edge 79+ |
| **推荐** | **Chrome 72+** 或 **Edge 79+** |

> Safari 不支持 VP8 编码的 MediaRecorder，无法开启直播。

### 观看直播（学生端）

| 功能 | 最低浏览器版本 |
|------|---------------|
| `MediaSource` VP8 解码 | Chrome 23+, Edge 12+, Safari 8+ |
| WebSocket 二进制帧 | Chrome 14+, Firefox 11+, Safari 6+ |
| **推荐** | **Chrome 23+** 或 **Edge 12+** 或 **Safari 8+** |

> 观看直播的浏览器要求远低于开启直播。几乎所有现代浏览器都支持观看，包括手机浏览器。
> 如果观看时黑屏，请确保已接受自签名证书（地址栏无红色警告）。

### 弹幕功能

所有现代浏览器均支持，无特殊要求。

## 快速开始

### 方式一：使用启动脚本（推荐）

**Windows：**
```bash
双击运行 start.bat
```

**macOS / Linux：**
```bash
chmod +x start.sh
./start.sh
```

### 方式二：手动启动

**1. 安装依赖**

| 依赖 | 版本要求 | 安装方式 |
|------|----------|----------|
| Node.js | >= 18 | [nodejs.org](https://nodejs.org/) |
| Rust | >= 1.70 | [rustup.rs](https://rustup.rs/) |

**2. 克隆项目**

```bash
git clone git@github.com:Disda-coding/bulletComment.git
cd bulletComment
```

**3. 安装前端依赖**

```bash
npm install
```

**4. 启动开发模式**

```bash
npx tauri dev
```

首次启动会编译 Rust 后端（约 1-3 分钟），之后启动会很快。

## 使用说明

### 教师端操作流程

1. 启动程序后点击 **「启动服务器」**，浏览器会自动打开客户端页面
2. 复制显示的地址（如 `https://192.168.1.100:9090`）发给学生
3. 点击 **「📌」** 固定弹幕窗口（鼠标穿透），再点一次解除固定
4. 使用 **「慢/中/快」** 调节弹幕滚动速度
5. 切换到 **「开启直播」** 标签页 → 调节码率和分辨率 → 点击 **「开始屏幕共享」** 进行直播
6. 拖拽控制条最左边的 **「⋮⋮」** 可移动控制条位置
7. 停止服务后日志自动保存到 `logs/` 目录

### 学生端操作流程

1. 用浏览器打开教师提供的地址
2. 首次访问需接受自签名证书警告：
   - Chrome: 点击 **「高级」** → **「继续访问」**
   - Safari: 点击 **「显示详情」** → **「访问此网站」**
3. 在 **「弹幕」** 标签页输入弹幕内容并发送
   - 可选填入自定义昵称（默认显示 IP + 设备名）
   - 可选择弹幕颜色
4. 在 **「观看直播」** 标签页观看教师屏幕共享
   - 播放器支持：全屏、窗口全屏、小窗（可拖拽）、画中画
5. 在 **「投票」** 标签页参与投票或发起投票
   - 默认进入参与投票页面，点击选项即可投票
   - 切换到「发起投票」可创建新投票

## 项目结构

```
bulletComment/
├── src-tauri/              # Rust 后端 (Tauri)
│   ├── src/
│   │   ├── main.rs         # 入口
│   │   ├── lib.rs          # Tauri 插件注册
│   │   ├── server.rs       # HTTP/WebSocket 服务器
│   │   ├── commands.rs     # Tauri 命令 (启停服务器等)
│   │   └── tray.rs         # 系统托盘
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── src/                    # Vue 前端
│   ├── views/
│   │   ├── ServerView.vue      # 控制面板
│   │   ├── DanmakuOverlay.vue  # 弹幕显示窗口
│   │   └── ControlBar.vue      # 控制条窗口
│   ├── App.vue             # 路由分发
│   └── types/index.ts      # 类型定义
├── web-client/             # 学生端 Web 客户端
│   └── index.html          # 单文件客户端 (内嵌到二进制)
├── logs/                   # 弹幕日志目录
├── start.bat               # Windows 启动脚本
├── start.sh                # macOS/Linux 启动脚本
└── package.json            # Node.js 配置
```

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | Vue 3 + TypeScript |
| 构建工具 | Vite |
| 后端 | Rust + Axum |
| 通信 | WebSocket (JSON + 二进制) |
| 直播 | MediaRecorder API → WebSocket → MediaSource API |
| HTTPS | 自签名证书 (rcgen) |
| 视频编码 | VP8 (WebM) |
| 零拷贝广播 | Bytes (引用计数共享) |

---

## 系统架构

```
┌─────────────────────────────────────────────────────────┐
│                    教师端 (Tauri 桌面应用)                  │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ 控制面板   │  │ 弹幕显示窗口  │  │ 控制条窗口        │   │
│  │ServerView │  │DanmakuOverlay│  │  ControlBar      │   │
│  │ (main)    │  │  (danmaku)   │  │  (control-bar)   │   │
│  └─────┬─────┘  └──────┬───────┘  └────────┬─────────┘   │
│        │               │                    │             │
│        └───────────────┼────────────────────┘             │
│                        │ Tauri Event Bus                   │
│  ┌─────────────────────┴──────────────────────────────┐   │
│  │              Tauri Commands (commands.rs)            │   │
│  │  start_server / stop_server / toggle_pin / ...      │   │
│  └─────────────────────┬──────────────────────────────┘   │
│                        │                                   │
│  ┌─────────────────────┴──────────────────────────────┐   │
│  │           Rust HTTP/WS Server (server.rs)            │   │
│  │  ┌─────────┐  ┌────────────┐  ┌─────────────────┐  │   │
│  │  │ axum    │  │ broadcast  │  │ rust-embed      │  │   │
│  │  │ +rustls │  │ channel    │  │ (web-client)    │  │   │
│  │  │ HTTPS   │  │ (文本+二进制)│  │ 嵌入HTML        │  │   │
│  │  └────┬────┘  └─────┬──────┘  └─────────────────┘  │   │
│  └───────┼─────────────┼──────────────────────────────┘   │
└──────────┼─────────────┼──────────────────────────────────┘
           │             │
     ┌─────┴─────┐ ┌────┴─────┐
     │  HTTPS    │ │ WebSocket │
     │  静态页面  │ │ 文本+二进制 │
     └─────┬─────┘ └────┬─────┘
           │             │
┌──────────┴─────────────┴──────────────────────────────────┐
│              学生端 (Web 浏览器)                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  弹幕     │  │ 观看直播  │  │ 开启直播  │  │  投票     │  │
│  │  Tab     │  │  Tab     │  │  Tab     │  │  Tab     │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
│              web-client/index.html (单文件)                 │
└───────────────────────────────────────────────────────────┘
```

### 关键设计决策

1. **双窗口方案**：弹幕窗口（可穿透/固定）+ 控制条窗口（始终可交互），解决 `set_ignore_cursor_events(true)` 导致整个窗口不可点击的问题
2. **单文件 Web 客户端**：`web-client/index.html` 通过 `rust-embed` 编译时嵌入 Rust 二进制，无需额外文件服务
3. **双通道 WebSocket**：文本帧（弹幕/投票/控制消息）和二进制帧（视频流）共用一个 WebSocket 连接
4. **自签名 HTTPS**：`getDisplayMedia` 要求安全上下文，使用 `rcgen` 运行时生成证书

---

## 技术选型

### Tauri 2.0 vs Electron

| 维度 | Tauri 2.0 | Electron |
|------|-----------|----------|
| 安装包大小 | ~5MB | ~80MB |
| 内存占用 | ~30MB | ~150MB |
| 后端语言 | Rust | Node.js |
| 系统集成 | 原生 API | Chromium |
| 透明窗口 | 原生支持 | 需要配置 |

选择 Tauri 的原因：轻量化是核心需求，课堂电脑配置参差不齐，Rust 后端性能远优于 Node.js。

### Rust + Axum vs Node.js

| 维度 | Rust + Axum | Node.js (ws) |
|------|-------------|--------------|
| WebSocket 并发 | 轻松 1000+ | 需要集群 |
| 内存占用 | 极低 | 较高 |
| 零拷贝广播 | Bytes 引用计数 | Buffer 拷贝 |
| TLS 性能 | rustls (原生) | OpenSSL 绑定 |

选择 Rust + Axum 的原因：与 Tauri 共享同一进程，无需额外启动 Node.js 服务；`tokio::sync::broadcast` 天然适合消息广播场景。

### VP8/WebM 编码

| 编码 | 浏览器支持 | 编码延迟 | 解码门槛 |
|------|-----------|---------|---------|
| VP8 | Chrome 47+, Edge 79+ | 低 | Chrome 23+ |
| H.264 | 全浏览器 | 高（需硬件） | 全浏览器 |
| VP9 | Chrome 48+ | 较高 | Chrome 48+ |

选择 VP8 的原因：`MediaRecorder` 原生支持，编码延迟低，解码门槛最低（Chrome 23+ 即可观看），适合课堂场景。

### 自签名 HTTPS (rcgen)

`getDisplayMedia()` 要求安全上下文（HTTPS 或 localhost）。使用 `rcgen` 在服务器启动时动态生成自签名证书，配合 `axum-server` + `rustls` 提供 HTTPS 服务。无需预先生成证书文件，零配置。

### rust-embed 嵌入 Web 客户端

将 `web-client/index.html` 在编译时嵌入 Rust 二进制，通过 Axum 路由直接返回。优点：
- 单一可执行文件，无需额外文件
- 编译时校验文件存在性
- 零运行时文件 I/O

---

## 模块详解

### 教师端控制面板 — ServerView.vue

**路由**: `/server`（主窗口默认路由）

**职责**: 服务器启停控制、显示访问地址和在线人数、弹幕速度调节、启动后自动打开浏览器

**关键交互**:
- `invoke("start_server", { port })` → 启动服务器 + 创建弹幕窗口 + 创建控制条窗口 + 打开浏览器
- `invoke("stop_server")` → 保存日志 + 关闭窗口 + 取消服务器
- `listen("online-count")` → 实时更新在线人数

### 弹幕显示窗口 — DanmakuOverlay.vue

**路由**: `/danmaku`（独立窗口，透明置顶）

**弹幕轨道分配算法**:
1. 遍历 10 条轨道，找到最近 2 秒内没有弹幕的轨道
2. 如果所有轨道都忙，则轮询分配 (nextId % TRACK_COUNT)
3. 每条轨道间隔 38px，弹幕从右向左滚动

**弹幕移除机制**:
- CSS `animationend` 事件：动画结束时立即移除 DOM
- `setTimeout` 兜底：`duration + 100ms` 后强制移除
- `MAX_DANMAKU = 200`：超过上限时移除最旧的弹幕

**速度控制**: 慢速 0.5x → 16000ms / 正常 1.0x → 8000ms / 快速 2.0x → 4000ms

### 控制条窗口 — ControlBar.vue

**路由**: `/control-bar`（独立窗口，200×36px，透明置顶）

**拖拽**: 使用 Tauri 原生 `getCurrentWindow().startDragging()` API，通过 `⋮⋮` 手柄触发。

**固定/移动**: 调用 `invoke("toggle_pin")` → 后端 `set_ignore_cursor_events(true/false)` → 弹幕窗口鼠标穿透切换。

### 学生端 Web 客户端 — web-client/index.html

**单文件架构**: 所有 HTML/CSS/JS 在一个文件中，通过 `rust-embed` 嵌入 Rust 二进制。

**四个标签页**:
1. **弹幕**: 发送弹幕 + 历史弹幕列表 + 颜色选择 + 昵称自定义
2. **观看直播**: MediaSource API 播放 + 全屏/小窗/画中画控件
3. **开启直播**: getDisplayMedia 屏幕共享 + 码率/分辨率调节
4. **投票**: 发起投票 + 参与投票（默认进入参与投票页）

**WebSocket 重连**: 指数退避策略（3s → 30s），最多 20 次。

### Rust 服务器 — server.rs

**核心数据结构**:
```rust
pub struct AppState {
    pub tx: broadcast::Sender<String>,           // 文本消息广播
    pub bin_tx: broadcast::Sender<Bytes>,         // 二进制帧广播（零拷贝）
    pub app_handle: AppHandle,                    // Tauri 事件发射
    pub online_count: Arc<AtomicUsize>,           // 在线人数
    pub live_active: Arc<AtomicBool>,             // 直播状态
    pub init_segment: Arc<Mutex<Option<Vec<u8>>>>, // WebM 初始化片段缓存
    pub danmaku_log: Arc<StdMutex<Vec<String>>>,  // 弹幕日志
    pub danmaku_history: Arc<StdMutex<Vec<String>>>, // 弹幕历史（最近100条）
    pub active_votes: Arc<StdMutex<Vec<Vote>>>,   // 活跃投票
}
```

**连接处理流程**:
1. 新连接 → 发送 `system connected`（含客户端 IP）+ 历史弹幕 + 活跃投票 + init_segment
2. 为每个连接启动两个 tokio 任务：`send_task`（广播转发）和 `recv_task`（消息处理）
3. 连接断开 → 更新在线人数 + 广播

**二进制帧广播**: 使用 `Bytes`（引用计数）替代 `Vec<u8>`，N 个客户端共享同一块内存，避免 N 次拷贝。`broadcast::channel::<Bytes>(512)` 缓冲 512 帧。

### Tauri 命令层 — commands.rs

| 命令 | 功能 | 返回值 |
|------|------|--------|
| `start_server(port)` | 启动服务器 + 创建弹幕窗口 + 创建控制条窗口 | 服务器地址 |
| `stop_server()` | 保存日志 + 关闭窗口 + 取消服务器 | - |
| `toggle_pin()` | 切换弹幕窗口鼠标穿透 | 当前固定状态 |
| `set_danmaku_speed(speed)` | 调节弹幕速度 | - |
| `get_local_ip()` | 获取本机局域网 IP | IP 地址 |
| `get_server_status()` | 获取服务器运行状态 | bool |

### 系统托盘 — tray.rs

提供两个菜单项：
- **切换固定/移动**: 发射 `toggle-pin` 事件 → ControlBar 接收后调用 `toggle_pin()`
- **退出**: `app.exit(0)`

---

## 通信协议

### WebSocket 文本消息

所有文本消息为 JSON 格式，包含 `type` 字段区分消息类型：

| type | 方向 | 用途 | 关键字段 |
|------|------|------|---------|
| `danmaku` | 客户端→服务端→广播 | 弹幕消息 | content, color, size, nickname, device |
| `system` | 服务端→客户端 | 系统消息 | action, content, ip |
| `online_count` | 服务端→广播 | 在线人数 | count |
| `live_start` | 客户端→服务端→广播 | 开始直播 | content |
| `live_stop` | 客户端→服务端→广播 | 停止直播 | content |
| `vote_create` | 客户端→服务端→广播 | 创建投票 | vote (Vote 结构体) |
| `vote_cast` | 客户端→服务端 | 投票 | vote_id, option_idx, voter |
| `vote_update` | 服务端→广播 | 投票更新 | vote (Vote 结构体) |
| `vote_close` | 客户端→服务端 | 关闭投票 | vote_id |

### WebSocket 二进制消息

- **推流端 → 服务端**: MediaRecorder 输出的 WebM VP8 帧
- **服务端 → 拉流端**: 广播所有二进制帧（Bytes 零拷贝）

### 投票数据结构

```typescript
interface Vote {
  id: string;           // v_时间戳_随机串
  question: string;     // 投票问题
  options: {            // 选项列表
    label: string;
    count: number;
  }[];
  voters: {             // 投票记录（防重复）
    [voterKey: string]: number;  // voterKey → option_idx
  };
  created_by: string;   // 创建者
  created_at: string;   // 创建时间
  closed: boolean;      // 是否已关闭
}
```

---

## 直播链路

### 完整数据流

```
推流端 (教师浏览器)                    服务端 (Rust)                     拉流端 (学生浏览器)
─────────────────                    ─────────────                    ─────────────────
getDisplayMedia()
     │
     ▼
MediaRecorder
(VP8, 800kbps)
     │
     ▼
ondataavailable
(500ms 间隔)
     │
     ├── 首帧 ──→ init_segment 缓存 ──→ 新连接时发送
     │
     ▼
WebSocket.send()  ──→  bin_tx.send(Bytes)  ──→  broadcast  ──→  所有 bin_rx 订阅者
                                                                    │
                                                                    ▼
                                                              handleVideoChunk()
                                                                    │
                                                                    ▼
                                                         sourceBufferQueue
                                                                    │
                                                                    ▼
                                                         sourceBuffer.appendBuffer()
                                                                    │
                                                                    ▼
                                                              MediaSource
                                                                    │
                                                                    ▼
                                                                <video>
```

### 初始化片段缓存机制

WebM 流的第一个二进制帧包含编解码器初始化信息（init segment）。新客户端连接时如果错过了这个帧，SourceBuffer 无法解码后续帧，导致黑屏。

解决方案：服务端缓存第一个二进制帧到 `init_segment`。新客户端连接时，在发送历史弹幕和投票后，立即发送缓存的 init_segment，确保 SourceBuffer 能正确初始化。

### 缓冲区管理

- **队列上限**: 150 帧，超出时丢弃最旧的帧
- **自动清理**: 缓冲区超过 30 秒时，移除前 20 秒的数据
- **错误恢复**: 最多重置 5 次，超过后显示错误提示

---

## 投票系统

### 消息流

```
发起者                     服务端                       所有客户端
  │                         │                              │
  │── vote_create ────────→ │                              │
  │                         │── 存入 active_votes ──────→  │
  │                         │── vote_create (广播) ────→   │  (其他客户端收到)
  │                         │                              │
  │── vote_cast ──────────→ │                              │
  │                         │── 验证：未投过/未关闭/选项有效  │
  │                         │── 更新 count + voters        │
  │                         │── vote_update (广播) ────→   │  (所有人看到实时结果)
  │                         │                              │
  │── vote_close ─────────→ │                              │
  │                         │── vote.closed = true         │
  │                         │── vote_update (广播) ────→   │  (所有人看到已结束)
```

### 防重复投票

- 服务端维护 `voters: HashMap<String, usize>`，key 为投票者标识（昵称或 IP），value 为所选选项索引
- 投票时检查 `!vote.voters.contains_key(voter)`，确保每人每票只能投一次
- 客户端也维护 `myVotedIds: Set` 用于 UI 状态判断

---

## 弹幕历史

- 服务端维护 `danmaku_history: Vec<String>`，存储最近 100 条弹幕的原始 JSON
- 新客户端连接时，在 `system connected` 消息之后，按顺序发送所有历史弹幕
- 弹幕消息同时存入 `danmaku_history` 和 `danmaku_log`（后者用于持久化保存）

---

## 性能优化

### Bytes 零拷贝广播

```rust
// 旧方案：每帧数据为每个客户端完整克隆
let _ = state_recv.bin_tx.send(data.to_vec());  // N 个客户端 = N 次拷贝

// 新方案：Bytes 引用计数共享
let _ = state_recv.bin_tx.send(Bytes::copy_from_slice(&data));  // N 个客户端共享同一块内存
```

### 码率/分辨率调节

开播前可在客户端调节：
- 码率：300kbps ~ 3Mbps（默认 800kbps）
- 分辨率：480p / 540p / 720p / 1080p（默认 540p）

### 性能参考

4C 8G 笔记本，默认参数（800kbps / 540p）：

| 场景 | 预估并发 |
|------|---------|
| WiFi 上行 ~30Mbps | ~37 台 |
| 有线上行 ~100Mbps | ~100+ 台 |

调低码率或分辨率可支撑更多客户端。

---

## 安全考量

### 自签名证书

- 使用 `rcgen` 运行时生成，CN=BulletComment
- 浏览器会显示证书警告，用户需手动信任
- 这是 `getDisplayMedia` 要求安全上下文的必要代价

### XSS 防护

- `escapeHtml()` 函数对投票问题、选项等用户输入进行转义
- 弹幕内容通过 `textContent` 设置（非 `innerHTML`），天然防 XSS
- 投票 ID 拼入 `onclick` 属性存在理论风险，但 ID 由服务端生成（`v_时间戳_随机串`），实际风险极低

### 输入验证

- 弹幕长度限制 100 字符（前端）
- 投票问题限制 100 字符，选项限制 50 字符（前端）
- 服务端对投票操作做基本验证（未投过、未关闭、选项索引有效）

---

## 端口与网络

- 默认端口：`9090`
- 协议：HTTPS（自签名证书）
- 监听地址：`0.0.0.0`（所有网卡）
- 学生端和教师端需要在同一局域网内

## 日志文件

弹幕日志保存在项目目录下的 `logs/` 文件夹：

文件命名格式：`danmaku_YYYYMMDD_HHMMSS.log`

日志内容示例：
```
=== 弹幕日志 2026-06-02 15:30:01 ===

[15:30:05] [172.16.102.86 / Mac] 大家好
[15:30:12] [小明 (192.168.1.100)] 老师好
[15:31:00] [172.16.102.87 / iPhone] 这个知识点没听懂
```

## 构建发布版本

```bash
npm run tauri build
```

构建产物在 `src-tauri/target/release/bundle/` 下：
- Windows: `.msi` / `.exe`
- macOS: `.dmg` / `.app`
- Linux: `.deb` / `.AppImage`

## 常见问题

### Q: 浏览器提示"不安全连接"
A: 这是正常的。系统使用自签名证书以支持屏幕共享功能（getDisplayMedia 要求 HTTPS）。点击"高级"→"继续访问"即可。

### Q: 直播画面是黑屏
A: 确保：
1. 使用 Chrome 或 Edge 浏览器
2. 已接受自签名证书（地址栏无红色警告）
3. 教师端已开启屏幕共享
4. 打开浏览器 F12 控制台查看 `[Viewer]` 日志

### Q: 开启直播时浏览器提示不支持
A: 开启直播需要 Chrome 72+ 或 Edge 79+。Safari 不支持 VP8 编码，无法开启直播。但观看直播的要求低很多，Chrome 23+ 即可。

### Q: 第二次启动服务连不上
A: 已修复。停止服务后会等待端口释放（500ms）后再允许重新启动。

### Q: 弹幕窗口不见了
A: 可能被其他窗口遮挡了。检查系统托盘图标，右键可以重新打开弹幕窗口。

### Q: 投票后看不到结果
A: 投票结果会实时更新。如果看不到，请检查网络连接是否正常。
