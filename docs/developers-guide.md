# Developer Guide

This guide explains the contributor workflow for the Monotony project.

Architectural rationale for the clock abstraction and the `test-util` feature
boundary lives in [clock design](clock-design.md). Path ownership and
repository boundaries live in [repository layout](repository-layout.md).

## Local Workflow

Use `make all` as the public entrypoint for formatting, linting, and tests.
`make lint` runs rustdoc, Clippy, and Whitaker. `make test` prefers
`cargo nextest run` and falls back to `cargo test` when cargo-nextest is not
available. Use `make test-fast` to run the same test entrypoint with the opt-in
`mold` linker route for local test builds. Compile-time API contracts live under
`tests/trybuild/` and run through the same test entrypoint with `trybuild`.
`make audit` derives the Rust workspace root with `cargo metadata`, logs
workspace member manifests, and runs `cargo audit` once from the workspace root.
`make coverage` uses `cargo llvm-cov` with `lld`.

GitHub Actions Act validation lives in `.github/workflows/act-validation.yml`.
The main `.github/workflows/ci.yml` workflow deliberately does not run
`make test WITH_ACT=1`; the separate Act workflow runs those slower
container-backed checks in parallel.

## Clock extension boundary

`MonotonicClock` remains the stable one-method production trait. Add
elapsed-time conveniences to `MonotonicClockExt` instead of adding default
methods to `MonotonicClock`, so downstream implementors do not need to update
their core trait implementations for minor releases.

Keep `MonotonicClockExt` limited to monotonic measurement. Sleeping, timers,
timeouts, retry cadence, async runtime integration, and accelerated logical
time are consumer-owned policy and must stay outside Monotony's production API.

## Test utility boundary

Deterministic clocks live behind the opt-in `test-util` feature so downstream
integration tests can use them without changing Monotony's default production
surface. `ManualMonotonicClock` is the single-owner manual test clock, while
`SharedManualMonotonicClock` is the cloneable test helper for cases where code
under test owns one clock handle and the test advances time through another.

Do not make sleeper policy part of `SharedManualMonotonicClock`. Tests that
need waiting behaviour should define a local sleeper or timer adapter and use
the shared manual clock only to observe and advance monotonic time.

## Tooling

Development builds use Cranelift for debug code generation. On Linux targets,
`.cargo/config.toml` configures clang with the repository's LLD baseline.
`make test-fast` opts into `mold` for faster local test linking. Coverage
generation uses `lld` because LLVM coverage tooling expects LLVM-compatible
linker behaviour.

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.

### Security audit ignores

Security audit jobs may set `CARGO_AUDIT_IGNORES` for narrowly scoped RustSec
advisories that affect unused or tooling-only dependency paths. Keep each
ignore tied to a documented runtime impact analysis, and remove it when the
affected dependency leaves the graph or the project starts using the advised
runtime path.
