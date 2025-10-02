# Repository Guidelines

## Project Structure & Module Organization
bookmark-checker is a single-crate Rust CLI. `src/main.rs` wires arguments into `src/lib.rs`, which now re-exports cohesive modules: `src/model.rs` (shared types and errors), `src/runner.rs` (orchestration and CLI flow), `src/locator.rs` (profile discovery), `src/parser.rs` (Serde JSON), `src/checker.rs` (Rayon HTTP checks), `src/report.rs` (YAML output), and `src/progress.rs` (indicatif progress). Unit tests live inline under `#[cfg(test)]`; add end-to-end coverage under `tests/`. The `target/` directory is cargo output and should stay untracked.

## Build, Test, and Development Commands
- `cargo check` — fast type-check before deeper changes.
- `cargo fmt` — apply rustfmt; run `cargo fmt --check` in CI discussions.
- `cargo clippy --all-targets -- -D warnings` — static analysis with warnings promoted to errors.
- `cargo test` — execute module and doc tests; use `cargo test -- --nocapture` to view progress logs.
- `cargo run -- --help` — inspect CLI flags such as `--max`.
- `cargo run -- --list-profiles` — print Chrome profiles detected.
- `cargo run -- --profile "Profile 1"` — check bookmarks for a specific Chrome profile.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults (4-space indentation, 100-column wrap). New modules should stay in `src/` with snake_case file names; public types and traits use PascalCase, functions and variables use snake_case. Prefer early returns with `?` for error propagation and keep user-facing strings routed through `BookmarkError`. Run `cargo fmt` and `cargo clippy` before opening a PR.

## Testing Guidelines
Unit tests already cover profile paths across macOS, Linux, Windows, plus parser behavior. Extend them when touching OS-specific code or Serde logic. Favor pure functions so tests stay deterministic; if HTTP checks need fixtures, guard them with `#[cfg(test)]` helpers or mark them `#[ignore]` and explain why. When live checks fail, inspect `bookmark_failures.yml` (written to the project root) for categorized `not_found`, `unauthorized`, and `connection_errors` lists.

## Commit & Pull Request Guidelines
The history currently follows Conventional Commits (`feat: initial commit`); continue using `type: subject` summaries in 50 characters or fewer. Each PR should include: a concise description of the change, any relevant issue IDs, how you validated it (commands and sample output), and screenshots when UX output changes. Ensure `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` pass locally before requesting review.

## Security & Configuration Notes
Bookmark discovery reads paths inside the Chrome profile under the current user (`~/Library/...` on macOS, `%LOCALAPPDATA%\\Google\\Chrome\\...` on Windows). Never commit real bookmark files or personal data. Use the existing TLS-enabled `reqwest` client; avoid enabling `default-features` unless you have audited the impact.
