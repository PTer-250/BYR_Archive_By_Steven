#!/bin/bash
# 启动脚本

set -e

echo "🚀 Starting BYR jsDelivr Service..."

# 检查环境变量
if [ -z "$REGISTRY" ]; then
    export REGISTRY="https://registry.npmjs.org"
    echo "📦 Using default npm registry: $REGISTRY"
else
    echo "📦 Using custom registry: $REGISTRY"
fi

if [ -z "$PORT" ]; then
    export PORT="3000"
fi

echo "🌐 Server will listen on port: $PORT"

# 检查是否需要编译
if [ ! -f "target/release/byr-jsdelivr" ]; then
    echo "🔨 Building release binary..."
    cargo build --release
fi

# 启动服务
echo "✅ Starting server..."
exec ./target/release/byr-jsdelivr
