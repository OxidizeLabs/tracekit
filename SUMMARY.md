# Template Repo Summary

This repository is intended to be used as a template for Rust-based projects.

## Quick Start

- Create a new repo using GitHub “Use this template” (or clone it).
- Update `Cargo.toml` (`package.name`, `description`, `repository`, etc.).
- Replace `README.md` content and badges with your project’s info.

## Commands

```sh
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
cargo run
```

## CI/CD

GitHub workflows live in `.github/workflows/` and provide:

- CI: formatting, clippy, tests, docs build, security audit, MSRV check.
- Benchmarks: scheduled/manual `cargo bench` run with uploaded output.
