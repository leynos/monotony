# Clock design

This document records the architectural rationale for Monotony's clock
abstraction and feature-gated deterministic test helpers.

## Context

Rust code that measures elapsed time often reaches directly for
`std::time::Instant::now()`. That is simple in production code, but it makes
tests depend on the host clock and scheduler. Tests for timeout logic, polling
loops, and elapsed-time measurement then become slower, less precise, or more
fragile than the behaviour being exercised.

Monotony exists to make monotonic time injectable without bringing a runtime,
mocking framework, or async dependency into production builds.

## Production API

The production API is deliberately small:

- `MonotonicClock` is a `Send + Sync` trait with one method, `now()`.
- `StdMonotonicClock` is the production adapter backed by
  `std::time::Instant::now()`.
- `MonotonicClockExt` provides narrow convenience methods, such as
  `elapsed_since`, without changing the core trait contract.
- The production dependency set is empty.

This keeps the public boundary easy to implement in downstream crates and
avoids coupling application logic to a concrete clock source. Consumers can use
`&dyn MonotonicClock`, generic parameters, or their own clock implementations
depending on the surrounding design.

## Clock, not sleeper

Monotony intentionally measures monotonic elapsed time. It does not own sleep,
timers, async runtimes, retry policy, timeout policy, or logical-time scaling.
Those behaviours belong in downstream application code because blocking,
asynchronous, and accelerated test sleepers each carry different operational
trade-offs.

Consumers that need both measurement and waiting should inject two boundaries: a
`MonotonicClock` for observation and a local sleeper or timer adapter for
policy. This keeps crates such as `catnap` free to choose blocking sleep, async
runtime sleep, or test-time acceleration without making those choices part of
Monotony's public API.

## Test utilities

Reusable deterministic clocks live in `src/test_util.rs` and are exported only
behind the `test-util` feature:

- `FixedMonotonicClock` supports code that reads `now()` exactly twice.
- `QueuedMonotonicClock` supports tests that need a known sequence of instants.
- `ManualMonotonicClock` supports polling and timeout tests that advance time
  explicitly.
- `SharedManualMonotonicClock` supports tests where code under test owns one
  clock handle and the test advances time from another cloned handle.

These helpers are not hidden behind `#[cfg(test)]` because downstream crates
cannot use a dependency's private test-only items in their own integration
tests. A Cargo feature keeps the helpers available to downstream test suites
without adding them to the default production surface.

## Feature boundary

The default feature set exposes only production clock abstractions. The
`test-util` feature exposes deterministic helpers for tests and examples.
Compile-time `trybuild` contracts assert both sides of this boundary:

- the public clock API compiles for downstream crates by default,
- `monotony::test_util` is unavailable without `test-util`, and
- deterministic clocks compile when `test-util` is enabled.

This boundary keeps normal dependency use small while preserving a stable,
documented test API for crates that need deterministic elapsed-time behaviour.
