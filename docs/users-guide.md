# User Guide

This guide explains how to use Monotony in Rust crates that need deterministic
elapsed-time measurement.

## Clock Abstraction

Monotony exposes a narrow `MonotonicClock` trait over `std::time::Instant`.
Inject the trait into code that measures durations, so production code can use
`StdMonotonicClock` while tests provide deterministic time.

```rust
use std::time::Duration;

use monotony::{MonotonicClock, StdMonotonicClock};

fn measure_elapsed(clock: &dyn MonotonicClock) -> Duration {
    let started_at = clock.now();
    clock.now().duration_since(started_at)
}

let elapsed = measure_elapsed(&StdMonotonicClock);

assert!(elapsed >= Duration::ZERO);
```

Import `MonotonicClockExt` when code wants the common elapsed-time helper
without changing the core trait contract.

```rust
use std::time::{Duration, Instant};

use monotony::{MonotonicClock, MonotonicClockExt};

fn has_elapsed(clock: &dyn MonotonicClock, started_at: Instant) -> bool {
    clock.elapsed_since(started_at) >= Duration::from_secs(5)
}
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
between observations. Use `SharedManualMonotonicClock` when code under test
must own one clock handle while the test advances time through another cloned
handle.

### Fixed elapsed time

Use `FixedMonotonicClock` when the code under test reads the clock once before
work starts and once after it finishes.

```rust
use std::time::Duration;

use monotony::{MonotonicClock, test_util::FixedMonotonicClock};

fn measure_elapsed(clock: &dyn MonotonicClock) -> Duration {
    let started_at = clock.now();
    clock.now().duration_since(started_at)
}

let clock = FixedMonotonicClock::with_elapsed(Duration::from_millis(250));

assert_eq!(measure_elapsed(&clock), Duration::from_millis(250));
```

### Queued instants

Use `QueuedMonotonicClock` when the code under test needs a sequence of known
clock readings.

```rust
use std::time::{Duration, Instant};

use monotony::{MonotonicClock, test_util::QueuedMonotonicClock};

fn collect_ticks(clock: &dyn MonotonicClock, count: usize) -> Vec<Instant> {
    (0..count).map(|_| clock.now()).collect()
}

let first = Instant::now();
let second = first + Duration::from_millis(10);
let third = second + Duration::from_millis(20);
let clock = QueuedMonotonicClock::from_instants([first, second, third]);

assert_eq!(collect_ticks(&clock, 3), vec![first, second, third]);
```

### Manual time

Use `ManualMonotonicClock` when the test needs to move time between repeated
observations, such as timeout or polling code.

```rust
use std::time::{Duration, Instant};

use monotony::{MonotonicClock, test_util::ManualMonotonicClock};

fn has_timed_out(clock: &dyn MonotonicClock, started_at: Instant) -> bool {
    clock.now().duration_since(started_at) >= Duration::from_secs(5)
}

let started_at = Instant::now();
let clock = ManualMonotonicClock::new(started_at);

assert!(!has_timed_out(&clock, started_at));

clock.advance(Duration::from_secs(5));

assert!(has_timed_out(&clock, started_at));
```

### Shared manual time

Use `SharedManualMonotonicClock` when one component observes time through an
owned clock handle and the test needs a separate handle for advancing time.

```rust
use std::time::{Duration, Instant};

use monotony::{MonotonicClock, test_util::SharedManualMonotonicClock};

fn has_timed_out(clock: &dyn MonotonicClock, started_at: Instant) -> bool {
    clock.now().duration_since(started_at) >= Duration::from_secs(5)
}

let started_at = Instant::now();
let observed_clock = SharedManualMonotonicClock::new(started_at);
let controller = observed_clock.clone();

assert!(!has_timed_out(&observed_clock, started_at));

controller.advance(Duration::from_secs(5));

assert!(has_timed_out(&observed_clock, started_at));
```

## Application Sleeper Policy

Monotony is a clock abstraction, not a sleeper abstraction. It does not own
blocking sleep, async runtime timers, retry policy, timeout policy, or
logical-time scaling. Applications that need both elapsed-time measurement and
waiting should compose `MonotonicClock` with their own sleeper boundary.

```rust
use std::time::{Duration, Instant};

use monotony::{MonotonicClock, test_util::SharedManualMonotonicClock};

trait Sleeper {
    fn sleep(&mut self, duration: Duration);
}

struct AdvancingSleeper {
    clock: SharedManualMonotonicClock,
    total_slept: Duration,
}

impl AdvancingSleeper {
    const fn new(clock: SharedManualMonotonicClock) -> Self {
        Self {
            clock,
            total_slept: Duration::ZERO,
        }
    }
}

impl Sleeper for AdvancingSleeper {
    fn sleep(&mut self, duration: Duration) {
        self.clock.advance(duration);
        self.total_slept += duration;
    }
}

#[derive(Clone, Copy, Debug)]
struct WaitPolicy {
    timeout: Duration,
    interval: Duration,
}

fn wait_until_timeout(
    clock: &dyn MonotonicClock,
    sleeper: &mut dyn Sleeper,
    started_at: Instant,
    policy: WaitPolicy,
) {
    loop {
        if clock.now().duration_since(started_at) >= policy.timeout {
            break;
        }

        sleeper.sleep(policy.interval);
    }
}

let started_at = Instant::now();
let observed_clock = SharedManualMonotonicClock::new(started_at);
let mut sleeper = AdvancingSleeper::new(observed_clock.clone());

wait_until_timeout(
    &observed_clock,
    &mut sleeper,
    started_at,
    WaitPolicy {
        timeout: Duration::from_secs(5),
        interval: Duration::from_secs(1),
    },
);

assert_eq!(sleeper.total_slept, Duration::from_secs(5));
assert!(observed_clock.now().duration_since(started_at) >= Duration::from_secs(5));
```

In production, the same application-owned `Sleeper` trait could call
`std::thread::sleep`, delegate to an async runtime, or apply a test-time
acceleration policy. Monotony stays responsible only for monotonic time
observation.

## Tooling

Monotony uses Rust 2024, a pinned nightly toolchain, strict lint settings, and
documented library code.

Development builds use Cranelift for debug code generation. On Linux targets,
`.cargo/config.toml` configures clang with the repository's LLD baseline. Use
`make test-fast` to opt into `mold` for faster local test linking. Coverage
generation uses `lld` because LLVM coverage tools expect LLVM-compatible linker
behaviour.

## Makefile Targets

The generated `Makefile` exposes these public targets:

- `make all` runs formatting checks, linting, and tests.
- `make check-fmt` verifies Rust formatting.
- `make lint` runs rustdoc, Clippy, and Whitaker with warnings denied.
- `make test` runs `cargo nextest run` when cargo-nextest is installed and
  falls back to `cargo test` otherwise. All projects also run doctests.
- `make test-fast` runs the same tests with the opt-in `mold` linker route.
- `make build` builds the debug target.
- `make release` builds the release target.
- `make coverage` writes `lcov.info` using `cargo llvm-cov` and `lld`.
- `make audit` derives the Rust workspace root with `cargo metadata` and runs
  `cargo audit` once from that root.
- `make markdownlint` checks Markdown files.
- `make nixie` validates Mermaid diagrams.

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.
