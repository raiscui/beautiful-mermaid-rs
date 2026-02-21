# LATER_PLANS.md
#
# 说明:
# - 用于记录"值得做,但本次不做"的后续计划/备忘.
# - 只能追加到文件尾部,禁止在中间插入内容.
#
# 最近更新: 2026-02-21

## 2026-02-21 - 后续计划: 自动化 Release 产物构建

- 目标: tag 推送后,用 GitHub Actions 自动构建并发布多平台产物(macOS/Linux/Windows)。
- 产物建议:
  - 每个平台独立 tar.gz/zip + sha256
  - 可选: 生成 checksums 汇总文件(单文件更易复制)
