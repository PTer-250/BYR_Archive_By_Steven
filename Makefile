.PHONY: help build run test clean dev release docker docker-run docker-stop fmt clippy check all

# 默认目标
help:
	@echo "BYR jsDelivr Service - Makefile"
	@echo ""
	@echo "可用命令:"
	@echo "  make build       - 编译项目（debug 模式）"
	@echo "  make release     - 编译项目（release 模式）"
	@echo "  make run         - 运行项目（debug 模式）"
	@echo "  make dev         - 开发模式运行（带详细日志）"
	@echo "  make test        - 运行测试"
	@echo "  make check       - 检查代码"
	@echo "  make fmt         - 格式化代码"
	@echo "  make clippy      - 运行 clippy 检查"
	@echo "  make clean       - 清理编译产物"
	@echo "  make docker      - 构建 Docker 镜像"
	@echo "  make docker-run  - 运行 Docker 容器"
	@echo "  make docker-stop - 停止 Docker 容器"
	@echo "  make all         - 格式化、检查、测试、构建"

# 编译项目（debug）
build:
	cargo build

# 编译项目（release）
release:
	cargo build --release

# 运行项目
run:
	cargo run

# 开发模式运行
dev:
	RUST_LOG=byr_jsdelivr=debug,tower_http=debug cargo run

# 运行测试
test:
	cargo test

# 检查代码
check:
	cargo check

# 格式化代码
fmt:
	cargo fmt

# Clippy 检查
clippy:
	cargo clippy -- -D warnings

# 清理
clean:
	cargo clean
	rm -rf target/

# 构建 Docker 镜像
docker:
	docker build -t byr-jsdelivr:latest .

# 运行 Docker 容器
docker-run:
	docker run -d \
		--name byr-jsdelivr \
		-p 3000:3000 \
		-e REGISTRY=https://registry.npmjs.org \
		-e PORT=3000 \
		--restart unless-stopped \
		byr-jsdelivr:latest
	@echo "Docker container started. Access at http://localhost:3000"

# 停止 Docker 容器
docker-stop:
	docker stop byr-jsdelivr || true
	docker rm byr-jsdelivr || true

# Docker Compose 启动
compose-up:
	docker-compose up -d

# Docker Compose 停止
compose-down:
	docker-compose down

# 完整检查
all: fmt clippy test build
	@echo "✅ All checks passed!"

# 安装开发工具
install-tools:
	rustup component add rustfmt clippy

# 查看日志（systemd）
logs:
	sudo journalctl -u byr-jsdelivr -f

# 启动服务（systemd）
start:
	sudo systemctl start byr-jsdelivr

# 停止服务（systemd）
stop:
	sudo systemctl stop byr-jsdelivr

# 重启服务（systemd）
restart:
	sudo systemctl restart byr-jsdelivr

# 查看服务状态（systemd）
status:
	sudo systemctl status byr-jsdelivr

# 生成文档
doc:
	cargo doc --open

# 安装到系统
install: release
	sudo cp target/release/byr-jsdelivr /usr/local/bin/
	@echo "✅ Installed to /usr/local/bin/byr-jsdelivr"

# 卸载
uninstall:
	sudo rm -f /usr/local/bin/byr-jsdelivr
	@echo "✅ Uninstalled from /usr/local/bin/"

# 性能测试
bench:
	@echo "Running basic performance test..."
	@echo "Testing /react endpoint..."
	@time curl -s http://localhost:3000/react > /dev/null

# 完整的 CI 检查
ci: fmt clippy test check
	@echo "✅ CI checks passed!"
