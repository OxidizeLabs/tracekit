# Rust Project Template

This repository is a starting point for Rust-based projects. Use it as a GitHub template to bootstrap a new crate with common project scaffolding (CI, docs, contribution guides, and sensible defaults).

## What’s Included

- Cargo workspace + toolchain configuration (`Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`).
- CI-friendly structure (`src/`, `tests/`, `examples/`, `benches/`) and supporting project files.
- Repository hygiene: `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`, maintainers/owners metadata.

## Getting Started

1. Click “Use this template” on GitHub (or clone/copy the repo).
2. Rename the crate/package:
   - Update `Cargo.toml` package name and metadata.
   - Update this `README.md` (project name, description, badges).
3. Build and test locally:

```sh
cargo test
```

## Development

Common commands:

```sh
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all
```

## License

Dual-licensed under MIT and Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
