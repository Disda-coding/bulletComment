@echo off
chcp 65001 >nul 2>&1
title 弹幕系统 - 启动脚本

echo ========================================
echo   课堂弹幕系统 - Windows 启动脚本
echo ========================================
echo.

cd /d "%~dp0"

if not exist "src-tauri" (
    echo [错误] 请将此脚本放在项目根目录下运行
    echo 项目根目录应包含 src-tauri 文件夹
    pause
    exit /b 1
)

echo [1/3] 检查 Node.js 环境...
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Node.js，请先安装 Node.js
    echo 下载地址: https://nodejs.org/
    pause
    exit /b 1
)
node -v

echo.
echo [2/3] 检查 Rust 环境...
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [警告] 未检测到 Rust，尝试使用预编译版本...
    if not exist "src-tauri\target\release\bullet-comment.exe" (
        echo [错误] 未找到可执行文件，请安装 Rust 或使用开发模式
        echo Rust 安装地址: https://rustup.rs/
        pause
        exit /b 1
    )
    echo 使用预编译版本启动...
    start "" "src-tauri\target\release\bullet-comment.exe"
    goto :end
)
rustc --version

echo.
echo [3/3] 启动弹幕系统 (开发模式)...
echo.
call npm install >nul 2>&1
npx tauri dev

:end
echo.
echo 程序已退出
pause
