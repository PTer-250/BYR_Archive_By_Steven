# 多阶段构建 Dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 src/main.rs 来缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/byr-jsdelivr /app/byr-jsdelivr

# 设置环境变量
ENV PORT=3000
ENV REGISTRY=https://registry.npmjs.org

# 暴露端口
EXPOSE 3000

# 运行应用
CMD ["/app/byr-jsdelivr"]
