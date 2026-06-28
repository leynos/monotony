# User Guide

This guide explains how to use Monotony in Rust crates that need deterministic
elapsed-time measurement.

## Clock Abstraction

Monotony exposes a narrow `MonotonicClock` trait over `std::time::Instant`.
Inject the trait into code that measures durations, so production code can use
`StdMonotonicClock` while tests provide deterministic time.

```rust
use monotony::{MonotonicClock, StdMonotonicClock};

fn measure(clock: &dyn MonotonicClock) -> std::time::Duration {
    let started_at = clock.now();
    clock.now().duration_since(started_at)
}

let elapsed = measure(&StdMonotonicClock);
assert!(elapsed >= std::time::Duration::ZERO);
```

## Test Utilities

Test helpers are available behind the `test-util` feature. They are not hidden
behind `#[cfg(test)]`, so downstream crates can enable them in their own
`dev-dependencies`.

```toml
[dev-dependencies]
monotony = { version = "0.1.0", features = ["test-util"] }
```

Use `FixedMonotonicClock::with_elapsed(...)` for code that calls `now()`
exactly twice. Use `QueuedMonotonicClock::from_instants(...)` when a test needs
several pre-seeded instants. Use `ManualMonotonicClock::advance(...)` for
polling loops and timeout code where the test should explicitly move time
between observations.

## Tooling

Monotony uses Rust 2024, a pinned nightly toolchain, strict lint settings, and
documented library code.

Development builds use Cranelift for debug code generation. On Linux targets,
`.cargo/config.toml` configures clang to link with `mold` so local debug builds
link quickly. Coverage generation uses `lld` instead because LLVM coverage
tools expect LLVM-compatible linker behaviour.

## Makefile Targets

The generated `Makefile` exposes these public targets:

- `make all` runs formatting checks, linting, and tests.
- `make check-fmt` verifies Rust formatting.
- `make lint` runs rustdoc, Clippy, and Whitaker with warnings denied.
- `make test` runs `cargo nextest run` when cargo-nextest is installed and
  falls back to `cargo test` otherwise. All projects also run doctests.
- `make build` builds the debug target.
- `make release` builds the release target.
- `make coverage` writes `lcov.info` using `cargo llvm-cov` and `lld`.
- `make audit` derives the Rust workspace root with `cargo metadata` and runs
  `cargo audit` once from that root.
- `make markdownlint` checks Markdown files.
- `make nixie` validates Mermaid diagrams.

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.
