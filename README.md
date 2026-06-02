# 课堂弹幕系统 (BulletComment)

跨平台内网弹幕 + 直播系统，支持 macOS / Windows / Linux。

## 功能特性

- **弹幕发送**：学生通过浏览器发送弹幕（支持手机/平板/电脑）
- **弹幕显示**：教师端半透明置顶窗口实时显示弹幕
- **固定/移动**：弹幕窗口可固定（鼠标穿透）或自由拖动
- **速度调节**：弹幕滚动速度可调（慢速/正常/快速）
- **屏幕直播**：教师共享屏幕直播给学生观看
- **播放器控件**：全屏、窗口全屏、小窗、画中画
- **历史弹幕**：新加入的用户可以看到最近 100 条弹幕记录
- **日志记录**：自动保存弹幕日志到 `logs/` 目录

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

1. 启动程序后点击 **「启动服务器」**
2. 复制显示的地址（如 `https://192.168.1.100:9090`）发给学生
3. 点击 **「📌」** 固定弹幕窗口（鼠标穿透），再点一次解除固定
4. 使用 **「慢/中/快」** 调节弹幕滚动速度
5. 切换到 **「开启直播」** 标签页 → 点击 **「开始屏幕共享」** 进行直播
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
5. 播放器支持：全屏、窗口全屏、小窗（可拖拽）、画中画

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

## 端口与网络

- 默认端口：`9090`
- 协议：HTTPS（自签名证书）
- 监听地址：`0.0.0.0`（所有网卡）
- 学生端和教师端需要在同一局域网内

## 日志文件

弹幕日志保存在程序运行目录下的 `logs/` 文件夹：

| 平台 | 路径 |
|------|------|
| macOS | `bullet-comment.app/Contents/Resources/logs/` |
| Windows | `程序目录\logs\` |
| Linux | `程序目录/logs/` |

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
2. 已接受自签名证书
3. 教师端已开启屏幕共享
4. 打开浏览器 F12 控制台查看 `[Viewer]` 日志

### Q: 第二次启动服务连不上
A: 已修复。停止服务后会等待端口释放（500ms）后再允许重新启动。

### Q: 弹幕窗口不见了
A: 可能被其他窗口遮挡了。检查系统托盘图标，右键可以重新打开弹幕窗口。
