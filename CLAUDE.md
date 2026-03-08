# CLAUDE.md

This file provides guidance for AI assistants working in this codebase.

## Project Overview

`exact` is an arbitrary-precision calculator written in Rust. It is in early development (v0.1.0 skeleton); the current source is a placeholder that will grow into a full calculator implementation.

- **Repository**: https://github.com/aexoden/exact
- **License**: Apache-2.0 OR MIT (dual-licensed)
- **Rust edition**: 2024
- **Minimum Rust version**: 1.93

## Development Environment

This project uses [Devbox](https://www.jetify.com/devbox) to provide a reproducible, Nix-based development environment. All tooling is pinned to specific versions.

### Setup

```bash
devbox shell   # Enter the development environment
```

The shell init hook automatically runs `cargo fetch` and `pre-commit install`.

### Pinned tool versions (devbox.json)

| Tool        | Version |
|-------------|---------|
| rustc       | 1.93.0  |
| cargo       | 1.93.0  |
| clippy      | 1.93.0  |
| rustfmt     | 1.93.0  |
| cargo-deny  | 0.19.0  |
| pre-commit  | 4.5.1   |

## Common Commands

```bash
cargo run                                      # Run the binary
cargo test -- --show-output                    # Run tests with output
cargo fmt --all -- --check                     # Check formatting (do not auto-format in CI)
cargo clippy --workspace -- -D warnings        # Lint (warnings are errors)
cargo deny check                               # Security/license audit
cargo doc                                      # Build documentation
```

Devbox script aliases:

```bash
devbox run start       # cargo run
devbox run test        # cargo test -- --show-output
devbox run format      # formatting check
devbox run lint        # clippy
devbox run build-docs  # cargo doc
```

## Code Quality Standards

All of the following must pass before a commit is accepted (enforced by pre-commit hooks and CI):

1. **`cargo fmt`** — code must be formatted according to `rustfmt` defaults.
2. **`cargo clippy -- -D warnings`** — zero warnings permitted. Both `cargo` and `pedantic` lint groups are enabled at warn level.
3. **`cargo test`** — all tests must pass.
4. **`cargo deny check`** — dependencies must comply with the license allowlist and have no known security advisories.

### Clippy lint configuration (Cargo.toml)

```toml
[lints.clippy]
cargo     = { level = "warn", priority = -1 }
pedantic  = { level = "warn", priority = -1 }
expect_used           = "warn"
unwrap_used           = "warn"
missing_errors_doc    = "allow"
multiple_crate_versions = "allow"

[lints.rust]
future_incompatible = { level = "warn", priority = -1 }
let_underscore      = { level = "warn", priority = -1 }
```

Key implications:
- Avoid `.unwrap()` and `.expect()` — use `?` or explicit error handling instead.
- Pedantic lints are active; prefer explicit, idiomatic Rust.
- `anyhow` is the standard error-handling library; return `anyhow::Result<T>` from fallible functions.

## Commit Message Convention

Commits **must** follow [Conventional Commits](https://www.conventionalcommits.org/) (enforced by `conventional-pre-commit`).

Format: `<type>: <description>`

Common types seen in this repo:

| Type    | Usage                                    |
|---------|------------------------------------------|
| `chore` | Dependencies, tooling, configuration     |
| `docs`  | Documentation changes                    |
| `feat`  | New features                             |
| `fix`   | Bug fixes                                |
| `ops`   | CI/CD and infrastructure changes         |
| `refactor` | Code restructuring without behavior change |
| `test`  | Adding or updating tests                 |

Examples from the git log:
```
docs: update CHANGELOG for v0.0.1
chore: add renovate.json
ops: add CI using GitHub Actions
```

## Source Code Structure

```
src/
├── main.rs   # Binary entry point — calls exact::run(), handles top-level errors
└── lib.rs    # Library root — exposes run() -> anyhow::Result<()>
```

The library/binary split means:
- All logic belongs in `src/lib.rs` (and modules beneath it).
- `src/main.rs` is a thin wrapper that exits with a non-zero code on error.
- Integration tests go in `tests/` (not yet present).

## Dependency Policy

Allowed licenses (deny.toml): `Apache-2.0`, `MIT`, `MPL-2.0`, `Unicode-3.0`.

Before adding a new dependency, confirm its license is on the allowlist. Run `cargo deny check` after any `Cargo.toml` change.

## CI/CD

GitHub Actions runs on every push and pull request to `main` (`.github/workflows/ci.yaml`). The single **Lint** job executes inside Devbox and performs:

1. `cargo fmt --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test`
4. `cargo deny check`

Concurrent runs for the same branch are automatically cancelled.

## Changelog

Maintained in `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) and [Semantic Versioning](https://semver.org/). Update the `[Unreleased]` section with every user-facing change; move it to a versioned section on release.

## Editor Settings

`.editorconfig` enforces:
- **Indentation**: 4 spaces (2 spaces for YAML/JSON)
- **Line endings**: LF
- **Trailing whitespace**: trimmed
- **Final newline**: required
- **Charset**: UTF-8
