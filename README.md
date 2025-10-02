# bookmark-checker

Audit Chrome bookmarks from the command line and capture problematic URLs in a YAML report.

bookmark-checker enumerates Chrome profiles on macOS, Linux, and Windows, parses bookmark files, and checks each entry over HTTPS. Unreachable, unauthorized, and other error statuses are persisted to `bookmark_failures.yml` for easy follow-up or cleanup.

## Features
- Discover Chrome profiles and scan any profile's bookmarks.
- Parallel HTTP validation with actionable summaries.
- Generate a categorized YAML report of failing bookmarks.
- Optional cleanup flow that removes reported bookmarks.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) 1.70+ and Cargo.

### Build & Run
```bash
# Type-check quickly
cargo check

# Format code before committing
cargo fmt

# Lint with Clippy (warnings treated as errors)
cargo clippy --all-targets -- -D warnings

# Run the CLI help
cargo run -- --help

# Scan bookmarks using the default Chrome profile
cargo run -- --scan

# List available profiles, then scan a specific one
cargo run -- --list-profiles
cargo run -- --scan --profile "Profile 1"

# Remove bookmarks previously reported as failing
cargo run -- --clean
```

## Usage Overview
- `--scan` / `-s`: audit bookmarks and write failures to `bookmark_failures.yml`.
- `--max-bookmarks <count>`: limit the scan to the first `count` bookmarks.
- `--profile <name>`: scan or clean a named Chrome profile instead of the default.
- `--list-profiles`: print discovered Chrome profiles and exit.
- `--clean`: remove bookmarks referenced in `bookmark_failures.yml`.
- `--version`: display the CLI version.
- `--help`: show usage information.

## Testing
Run the test suite with:
```bash
cargo test
```

## Project Layout
```
src/
  main.rs      # CLI entry point and argument parsing
  lib.rs       # Library exports
  model.rs     # Shared types and RunConfig
  runner.rs    # Orchestration of scans and cleanup
  locator.rs   # Chrome profile discovery per OS
  parser.rs    # Bookmark JSON parsing
  checker.rs   # Parallel HTTP validation
  report.rs    # YAML report writer
  progress.rs  # Progress indicators
```

## License
This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
