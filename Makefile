.PHONY: install build release clean sync-vendor sync-vendor-verify validate-docs

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
#
# 说明：
# - `beautiful-mermaid-rs` 的运行时逻辑依赖 `vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js`
# - 如果只 build/install 而忘了同步 TS bundle，安装出去的二进制可能在逻辑上“落后于上游”
# - 因此这里让 `install` 固定先跑一次 `sync-vendor-verify`（同步 bundle + cargo test），再做 release 构建与安装
install:
	@$(MAKE) sync-vendor-verify
	@$(MAKE) release
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

# 批量校验 Markdown 文档中的 Mermaid code fence（```mermaid ... ```）
#
# 说明：
# - 校验范围：README.md + docs/**/*.md
# - 成功：exit code=0
# - 失败：exit code=1，并打印具体文件与错误细节（stderr）
validate-docs:
	@bash -euo pipefail -c '\
		cargo build --quiet; \
		bin="./target/debug/beautiful-mermaid-rs"; \
		tmp="$$(mktemp)"; \
		trap "rm -f \"$$tmp\"" EXIT; \
		echo "开始校验 Markdown Mermaid: README.md + docs/**/*.md"; \
		echo "使用二进制: $$bin"; \
		echo ""; \
		echo "校验: README.md"; \
		: > "$$tmp"; \
		if ! "$$bin" --validate-markdown < "README.md" >/dev/null 2> "$$tmp"; then \
			echo "Mermaid 校验失败: README.md"; \
			cat "$$tmp"; \
			exit 1; \
		fi; \
		if [ -d "docs" ]; then \
			while IFS= read -r -d "" file; do \
				echo "校验: $$file"; \
				: > "$$tmp"; \
				if ! "$$bin" --validate-markdown < "$$file" >/dev/null 2> "$$tmp"; then \
					echo "Mermaid 校验失败: $$file"; \
					cat "$$tmp"; \
					exit 1; \
				fi; \
			done < <(find docs -type f -name "*.md" -print0); \
		fi; \
		echo ""; \
		echo "全部 Markdown Mermaid 校验通过"; \
	'
