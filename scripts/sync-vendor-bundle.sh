#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# 同步 beautiful-mermaid 的 browser bundle 到 Rust vendor
#
# 为什么要有它？
# - beautiful-mermaid-rs 的运行时逻辑来自：vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js
# - 如果只改了 TS 源码但忘记重新 build + copy，Rust 侧会继续使用旧 bundle
#
# 这个脚本做什么？
# 1) 在 TS 仓库执行 `bun run build`
# 2) 把 dist/beautiful-mermaid.browser.global.js 拷贝到 Rust 仓库 vendor
# 3) （可选）执行 `cargo test` 做端到端验证
# ============================================================================

DEFAULT_TS_DIR="/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid"

usage() {
  cat <<'EOF'
用法:
  scripts/sync-vendor-bundle.sh [--ts-dir <path>] [--skip-rust-tests]

参数:
  --ts-dir <path>        beautiful-mermaid (TypeScript) 仓库路径
  --skip-rust-tests      只同步 bundle，不跑 Rust 测试
  -h, --help             显示帮助

示例:
  # 直接用默认路径（我的本机路径）
  scripts/sync-vendor-bundle.sh

  # 指定 TS 仓库路径
  scripts/sync-vendor-bundle.sh --ts-dir /path/to/beautiful-mermaid

  # 只同步，不跑测试（更快）
  scripts/sync-vendor-bundle.sh --skip-rust-tests
EOF
}

TS_DIR=""
RUN_RUST_TESTS="true"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ts-dir)
      TS_DIR="${2:-}"
      shift 2
      ;;
    --skip-rust-tests)
      RUN_RUST_TESTS="false"
      shift 1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "未知参数: $1" >&2
      echo >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "${TS_DIR}" ]]; then
  TS_DIR="${DEFAULT_TS_DIR}"
fi

# 通过脚本位置定位 Rust 仓库根目录，避免依赖当前工作目录。
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RUST_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd -P)"

SRC_BUNDLE="${TS_DIR}/dist/beautiful-mermaid.browser.global.js"
DST_BUNDLE="${RUST_DIR}/vendor/beautiful-mermaid/beautiful-mermaid.browser.global.js"

if [[ ! -d "${TS_DIR}" ]]; then
  echo "TS 仓库目录不存在: ${TS_DIR}" >&2
  exit 1
fi

if [[ ! -f "${TS_DIR}/package.json" ]]; then
  echo "TS 仓库缺少 package.json（路径可能不对）: ${TS_DIR}" >&2
  exit 1
fi

if [[ ! -d "${RUST_DIR}/vendor/beautiful-mermaid" ]]; then
  echo "Rust vendor 目录不存在: ${RUST_DIR}/vendor/beautiful-mermaid" >&2
  exit 1
fi

if ! command -v bun >/dev/null 2>&1; then
  echo "未找到 bun，请先安装 bun（本项目 TS 侧默认使用 bun 运行构建/测试）" >&2
  exit 1
fi

echo "==> [1/3] 构建 TS bundle"
echo "TS_DIR=${TS_DIR}"
(
  cd "${TS_DIR}"

  # 如果没有依赖目录，先装依赖，避免 build 直接失败。
  if [[ ! -d "node_modules" ]]; then
    echo "未发现 node_modules，先执行: bun install"
    bun install
  fi

  bun run build
)

if [[ ! -f "${SRC_BUNDLE}" ]]; then
  echo "找不到构建产物: ${SRC_BUNDLE}" >&2
  exit 1
fi

echo "==> [2/3] 同步 bundle 到 Rust vendor"
cp "${SRC_BUNDLE}" "${DST_BUNDLE}"

# 用 sha256 做一次“确实同步成功”的确认，避免误拷贝/拷贝失败不自知。
SRC_SHA256="$(shasum -a 256 "${SRC_BUNDLE}" | awk '{print $1}')"
DST_SHA256="$(shasum -a 256 "${DST_BUNDLE}" | awk '{print $1}')"

if [[ "${SRC_SHA256}" != "${DST_SHA256}" ]]; then
  echo "同步后 hash 不一致，可能拷贝失败：" >&2
  echo "  SRC=${SRC_SHA256}" >&2
  echo "  DST=${DST_SHA256}" >&2
  exit 1
fi

echo "bundle sha256: ${DST_SHA256}"
echo "bundle path:   ${DST_BUNDLE}"

echo "==> [3/3] 端到端验证（Rust）"
if [[ "${RUN_RUST_TESTS}" == "true" ]]; then
  (
    cd "${RUST_DIR}"
    cargo test --quiet
  )
  echo "Rust tests: OK"
else
  echo "已跳过 Rust 测试（--skip-rust-tests）"
fi

echo "✅ 完成"

