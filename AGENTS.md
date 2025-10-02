# Repository Guidelines

## Project Structure & Module Organization
bookmark-checker is a Rust 2024 workspace with a single crate. `src/main.rs` is the CLI entry point and delegates to the public API in `src/lib.rs`. Feature-specific modules are split across `src/locator.rs` (Chrome profile discovery per OS), `src/parser.rs` (Serde JSON parsing), `src/checker.rs` (Rayon-backed HTTP verification), and `src/progress.rs` (indicatif progress display). Unit tests live inline under `#[cfg(test)]`; add integration tests under `tests/` if you need end-to-end coverage. The `target/` directory is cargo output and should remain untracked.

## Build, Test, and Development Commands
- `cargo check` — fast type-check before deeper changes.
- `cargo fmt` — apply rustfmt; run `cargo fmt --check` in CI discussions.
- `cargo clippy --all-targets -- -D warnings` — static analysis with warnings promoted to errors.
- `cargo test` — execute module and doc tests; use `cargo test -- --nocapture` to view progress logs.
- `cargo run -- --help` — inspect CLI flags such as `--max`.
- `cargo run -- --list-profiles` — print Chrome profiles detected.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults (4-space indentation, 100-column wrap). New modules should stay in `src/` with snake_case file names; public types and traits use PascalCase, functions and variables use snake_case. Prefer early returns with `?` for error propagation and keep user-facing strings routed through `BookmarkError`. Run `cargo fmt` and `cargo clippy` before opening a PR.

## Testing Guidelines
Existing unit tests assert path resolution across macOS, Linux, and Windows footprints and parser behavior. Extend them when touching OS-specific code or Serde logic. Favor pure functions for new logic so they remain deterministic without live network calls; if HTTP checks require fixtures, guard them with `#[cfg(test)]` helpers or mark them `#[ignore]` and document the reason.

## Commit & Pull Request Guidelines
The history currently follows Conventional Commits (`feat: initial commit`); continue using `type: subject` summaries in 50 characters or fewer. Each PR should include: a concise description of the change, any relevant issue IDs, how you validated it (commands and sample output), and screenshots when UX output changes. Ensure `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` pass locally before requesting review.

## Security & Configuration Notes
Bookmark discovery reads paths inside the Chrome profile under the current user (`~/Library/...` on macOS, `%LOCALAPPDATA%\\Google\\Chrome\\...` on Windows). Never commit real bookmark files or personal data. Use the existing TLS-enabled `reqwest` client; avoid enabling `default-features` unless you have audited the impact.
