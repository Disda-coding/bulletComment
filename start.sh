#!/bin/bash
set -e

echo "========================================"
echo "  课堂弹幕系统 - Linux/macOS 启动脚本"
echo "========================================"
echo

cd "$(dirname "$0")"

if [ ! -d "src-tauri" ]; then
    echo "[错误] 请将此脚本放在项目根目录下运行"
    echo "项目根目录应包含 src-tauri 文件夹"
    exit 1
fi

OS="$(uname -s)"
case "$OS" in
    Linux*)  OS_NAME="Linux" ;;
    Darwin*) OS_NAME="macOS" ;;
    *)       OS_NAME="Unknown" ;;
esac
echo "检测到系统: $OS_NAME"
echo

echo "[1/3] 检查 Node.js 环境..."
if ! command -v node &> /dev/null; then
    echo "[错误] 未检测到 Node.js，请先安装 Node.js"
    echo "安装方式: https://nodejs.org/ 或包管理器 (apt/brew)"
    exit 1
fi
node -v

echo
echo "[2/3] 检查 Rust 环境..."
if ! command -v rustc &> /dev/null; then
    echo "[警告] 未检测到 Rust，尝试使用预编译版本..."
    if [ "$OS_NAME" = "Linux" ]; then
        BINARY="src-tauri/target/release/bullet-comment"
    else
        BINARY="src-tauri/target/release/bullet-comment.app/Contents/MacOS/bullet-comment"
    fi
    if [ ! -f "$BINARY" ]; then
        echo "[错误] 未找到可执行文件，请安装 Rust 或使用开发模式"
        echo "Rust 安装: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    echo "使用预编译版本启动..."
    exec "$BINARY"
fi
rustc --version

echo
echo "[3/3] 安装依赖并启动弹幕系统 (开发模式)..."
echo

npm install --silent
npx tauri dev
