.PHONY: install build release clean sync-vendor sync-vendor-verify

# 安装目标目录
INSTALL_DIR = /Users/cuiluming/local_doc/l_dev/tool

# 二进制文件名
BINARY_NAME = beautiful-mermaid-rs

# Release 构建目录
RELEASE_DIR = target/release

# TS 仓库路径（可覆盖）：
#   make TS_REPO_DIR=/path/to/beautiful-mermaid sync-vendor
TS_REPO_DIR ?= /Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid

# 构建 release 版本
release:
	cargo build --release

# 安装到指定目录
install: release
	@echo "正在安装 $(BINARY_NAME) 到 $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@cp $(RELEASE_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "安装完成: $(INSTALL_DIR)/$(BINARY_NAME)"

# 清理构建文件
clean:
	cargo clean

# 同步上游 TS 的 browser bundle 到 vendor（不跑 Rust 测试，更快）
sync-vendor:
	./scripts/sync-vendor-bundle.sh --ts-dir "$(TS_REPO_DIR)" --skip-rust-tests

# 同步 bundle + 运行 Rust 测试做端到端验证（推荐）
sync-vendor-verify:
	./scripts/sync-vendor-bundle.sh --ts-dir "$(TS_REPO_DIR)"
