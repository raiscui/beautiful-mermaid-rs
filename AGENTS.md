# Repository Guidelines

## Project Structure & Module Organization

- js beautiful-mermaid.browser.global.js 源代码 :/Users/cuiluming/local_doc/l_dev/ref/typescript/beautiful-mermaid

- `src/`: Rust library + CLI entrypoint.
  - `src/lib.rs`: public API (`render_mermaid`, `render_mermaid_ascii`).
  - `src/main.rs`: CLI (reads Mermaid from stdin).
  - `src/js.rs`: QuickJS (`rquickjs`) integration + bundled JS execution.
  - `src/theme.rs`, `src/types.rs`, `src/error.rs`: options, themes, error types.
- `tests/`: integration-style tests.
  - `tests/ascii_testdata.rs`: golden tests against `tests/testdata/{ascii,unicode}/*.txt`.
  - `tests/svg_smoke.rs`: basic SVG sanity checks.
- `vendor/beautiful-mermaid/`: vendored JS bundle (`beautiful-mermaid.browser.global.js`).
- `docs/`: extra docs (e.g. `docs/code-agent-cli.md`).

## Build, Test, and Development Commands

- `cargo build`: debug build.
- `cargo test`: run all tests (golden ASCII/Unicode + SVG smoke).
- `printf 'graph LR\nA --> B\n' | cargo run --quiet`: run CLI (SVG to stdout).
- `printf ... | cargo run --quiet -- --ascii [--use-ascii]`: ASCII/Unicode output.
- `make release`: `cargo build --release`.
- `make install INSTALL_DIR=/path/to/bin`: sync bundle + `cargo test` + `cargo build --release` + install `target/release/beautiful-mermaid-rs`.
  - Override TS repo path via `TS_REPO_DIR=/path/to/beautiful-mermaid`.
- `make sync-vendor`: rebuild upstream TS bundle and copy into `vendor/` (skips Rust tests).
- `make sync-vendor-verify`: sync bundle + run `cargo test` (recommended).
  - Override TS repo path via `TS_REPO_DIR=/path/to/beautiful-mermaid`.
  - Implementation: `scripts/sync-vendor-bundle.sh`.

## Coding Style & Naming Conventions

- Run `cargo fmt --all` before pushing changes.
- Keep Rust naming idiomatic: `snake_case` fns/vars, `CamelCase` types, `SCREAMING_SNAKE_CASE` consts.
- Prefer small, direct APIs; avoid `unsafe` unless there is a strong reason.
- Docs/comments in this repo are mostly Simplified Chinese—keep consistency in files you touch.

## Testing Guidelines

- If you change rendering output, update/extend golden files under `tests/testdata/`.
- New cases should use `snake_case.txt` names and stay deterministic (stable ordering, no timestamps).

## Commit & Pull Request Guidelines

- Commit messages follow `type: summary` (current history: `init: ...`); prefer `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`.
- PRs should include: what/why, how verified (`cargo test`), and at least one CLI example.
- If updating the vendored JS bundle, describe the source/build steps and keep licensing accurate.
