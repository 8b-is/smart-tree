# Repository Guidelines

This guide gives contributors the essentials for working on Smart Tree in this repository. Keep updates concise and run the pre-commit checks before opening a PR.

## Project Structure & Module Organization
- `src/`: Main Rust crate. Key areas: `main.rs`/`cli.rs` (entry & clap args), `scanner.rs` (walks the filesystem), `formatters/` (output formats), `mcp/` (JSON-RPC tools), `mem8/` (persistence), `web_dashboard/` (web UI backend), and `bin/` targets for `st`, `mq`, `m8`, `n8x`.
- `scripts/`: Automation (`manage.sh`, `build-and-install.sh`) plus helper shell functions.
- `tests/`: Rust integration tests; fixtures live alongside in `test`, `test_files`, and `test-dirs`.
- `docs/` and `examples/`: User-facing guides and sample outputs.
- `marqant/`: Compression submodule; `expert_prompt_engineer/`: secondary workspace crate.

## Build, Test, and Development Commands
- `./scripts/manage.sh test` — pre-commit bundle: `cargo test` + `cargo clippy -D warnings` + `cargo fmt --check`.
- `cargo build --release [--features tui|full]` — optimized binaries; default builds the core CLI.
- `cargo run --bin st -- --help` — smoke-test the CLI; add flags like `--mode ai --compress` as needed.
- `cargo test` or `cargo test name -- --exact --nocapture` — run all or targeted tests with output.
- `cargo fmt --all -- --check` and `cargo clippy -- -D warnings` — enforce style/lints when iterating.
- `./scripts/build-and-install.sh` — rebuild and install locally, clearing shell command cache.

## Coding Style & Naming Conventions
- Rust 2021, 4-space indent. Keep functions small; prefer composition over long match blocks.
- Use `anyhow::Result` with `.context(...)`; avoid panics in library code.
- File/module names snake_case; types/enums struct PascalCase; CLI flags stay kebab-case to match `cli.rs`.
- Logging via `tracing` macros; keep comments brief and in the existing light, direct tone.

## Testing Guidelines
- Primary suite: `cargo test`; async tests use `#[tokio::test]`.
- Integration coverage in `tests/`; data fixtures under `test`/`test_files`/`test-dirs`.
- CLI/regression scripts (`test_spicy.sh`, `test_feature_flags.sh`, etc.) can be run directly with `bash`.
- For verbose diagnostics: `RUST_LOG=debug cargo test -- --nocapture`.
- Aim for zero clippy warnings and add tests for new branches or error paths.

## Commit & Pull Request Guidelines
- Commit format: `type: short description` where `type` ∈ {feat, fix, docs, style, refactor, test, chore}. Keep commits focused.
- Before PR: ensure `./scripts/manage.sh test` passes and docs are updated when behavior changes.
- PR description should summarize intent, link issues, list relevant feature flags, and include output or screenshots for UX-facing changes (TUI/dashboard).

## Security & Configuration Tips
- Run `st --cleanup` after installs and `st --security-scan .` before releases to detect risky MCP hooks or supply-chain issues.
- Keep secrets and local settings out of the repo; confirm `.gitignore` coverage before adding new generated files.
- Prefer local/offline MCP servers; discuss any new networked hooks in the PR to ensure review.
