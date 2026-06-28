# Developer Guide

This guide explains the contributor workflow for the Monotony project.

## Crate Boundaries

Production code must remain dependency-free. The core crate surface is limited
to the `MonotonicClock` trait and the `StdMonotonicClock` adapter.

Reusable deterministic clocks live in `src/test_util.rs` and are exposed only
through the `test-util` feature. Keep helpers in that module when they are
intended for downstream crate tests; keep private test-only fixtures inside
individual test modules when they are useful only to Monotony's own tests.

## Local Workflow

Use `make all` as the public entrypoint for formatting, linting, and tests.
`make lint` runs rustdoc, Clippy, and Whitaker. `make test` prefers
`cargo nextest run` and falls back to `cargo test` when cargo-nextest is not
available. Compile-time API contracts live under `tests/trybuild/` and run
through the same test entrypoint with `trybuild`. `make audit` derives the Rust
workspace root with `cargo metadata`, logs workspace member manifests, and runs
`cargo audit` once from the workspace root. `make coverage` uses
`cargo llvm-cov` with `lld`.

GitHub Actions Act validation lives in `.github/workflows/act-validation.yml`.
The main `.github/workflows/ci.yml` workflow deliberately does not run
`make test WITH_ACT=1`; the separate Act workflow runs those slower
container-backed checks in parallel.

## Tooling

Development builds use Cranelift for debug code generation. On Linux targets,
`.cargo/config.toml` configures clang to link with `mold` so debug builds link
quickly. Coverage generation uses `lld` because LLVM coverage tooling expects
LLVM-compatible linker behaviour.

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.

### Security audit ignores

Security audit jobs may set `CARGO_AUDIT_IGNORES` for narrowly scoped RustSec
advisories that affect unused or tooling-only dependency paths. Keep each
ignore tied to a documented runtime impact analysis, and remove it when the
affected dependency leaves the graph or the project starts using the advised
runtime path.
