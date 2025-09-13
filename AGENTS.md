# Repository Guidelines

## Project Structure & Module Organization
- `src/` — Rust library code.
  - `frame/` — frame header, flags, CRC, payload encode/decode.
  - `pipe/` — process pipes + JSON control frames (Tokio).
  - `errors.rs`, `bit_operations.rs` — error types and endian helpers.
- `tests/` — PHP worker used by pipe tests (`worker.php`, Composer deps).
- `.github/` — CI, PR template, Dependabot.

## Build, Test, and Development Commands
- Build: `cargo build` (use `--release` for optimized build).
- Format check: `cargo fmt --all -- --check` (CI runs this).
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`.
- Tests: `cargo test` (requires PHP CLI for pipe tests; see below).
- PHP deps for tests: `cd tests && composer install` (or `update` to refresh).

## Coding Style & Naming Conventions
- Rust 2024 edition, stable toolchain (`rust-toolchain.toml`).
- Use `rustfmt` defaults; 4‑space indentation; wrap lines reasonably.
- Run `clippy`; fix or explicitly justify lints (CI denies warnings).
- Naming: modules/functions `snake_case`; types/traits `UpperCamelCase`; constants `SCREAMING_SNAKE_CASE`.
- Prefer `anyhow` for fallible ops in `pipe`; avoid `unwrap()` in library code.

## Testing Guidelines
- Unit tests live next to code (`#[cfg(test)]` in modules like `frame` and `bit_operations`).
- Async tests use `#[tokio::test]`.
- Pipe tests call `php tests/worker.php`; ensure PHP 8+ in `PATH` and Composer deps installed.
- Add focused tests for frame parsing, CRC verification, and header/options boundaries.

## Commit & Pull Request Guidelines
- Commit style: use clear prefixes seen in history (e.g., `feature: ...`, `fix: ...`, `chore: ...`, `build(deps): ...`).
- Sign commits: `git commit -s` (enforced by PR checklist).
- PRs: follow `.github/pull_request_template.md`; include rationale, concise description, tests, and any doc updates. Link issues (`Closes #123`).

## Security & Configuration Tips
- Validate CRC before trusting payloads; treat external process I/O as untrusted.
- Avoid adding new runtime deps unless necessary; keep APIs zero‑cost where possible.
- Local concurrency: prefer Tokio primitives; do not block the async runtime.
