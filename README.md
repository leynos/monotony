# Monotony

*Ticking away the moments that make up a dull day.*

Monotony is a small Rust crate for injecting monotonic clocks. It gives
production code a narrow `MonotonicClock` trait over `std::time::Instant`, and
gives tests deterministic clocks without reaching for `#[cfg(test)]` internals.

______________________________________________________________________

## Why monotony?

- **Keep time injectable:** Measure elapsed time without coupling business
  logic directly to `Instant::now()`.
- **Mock time downstream:** Enable the `test-util` feature from your own
  crate's dev-dependencies and use deterministic clocks in integration tests.
- **Bring your own sleeper:** Pair clocks with application-owned sleep, timer,
  async runtime, or logical-time policy instead of making those choices part of
  Monotony.
- **Stay small:** The production crate surface is dependency-free and focused
  on one job.

______________________________________________________________________

## Quick start

### Installation

```toml
[dependencies]
monotony = "0.1.0"
```

Enable deterministic clocks for tests:

```toml
[dev-dependencies]
monotony = { version = "0.1.0", features = ["test-util"] }
```

### Basic usage

```rust
use std::time::Duration;

use monotony::{MonotonicClock, StdMonotonicClock};

fn measure(clock: &dyn MonotonicClock) -> Duration {
    let started_at = clock.now();
    clock.now().duration_since(started_at)
}

let elapsed = measure(&StdMonotonicClock);
assert!(elapsed >= Duration::ZERO);
```

### Mocking time

```rust
use std::time::Duration;

use monotony::{MonotonicClock, test_util::FixedMonotonicClock};

fn measure(clock: &dyn MonotonicClock) -> Duration {
    let started_at = clock.now();
    clock.now().duration_since(started_at)
}

let clock = FixedMonotonicClock::with_elapsed(Duration::from_secs(3));

assert_eq!(measure(&clock), Duration::from_secs(3));
```

______________________________________________________________________

## Features

- `MonotonicClock` trait for elapsed-time measurement.
- `MonotonicClockExt` convenience methods for narrow elapsed-time helpers.
- `StdMonotonicClock` production adapter backed by `Instant::now()`.
- `FixedMonotonicClock` for exactly two `now()` calls in simple tests.
- `QueuedMonotonicClock` for deterministic sequences of instants.
- `ManualMonotonicClock` for tests that explicitly advance time.
- `SharedManualMonotonicClock` for cloneable manual time control across owned
  test handles.
- `trybuild`, doctest, property, and runtime coverage for the public contract.

Monotony intentionally does not own sleep, timers, async runtimes, retry
policy, timeout policy, or logical-time scaling. Downstream crates should pair
`MonotonicClock` with their own sleeper or timer adapter when they need waiting
behaviour.

______________________________________________________________________

## Learn more

- [Documentation contents](docs/contents.md) — full documentation index.
- [Users' guide](docs/users-guide.md) — usage, test helpers, and Make targets.
- [Developers' guide](docs/developers-guide.md) — contributor workflow and
  local tooling.
- [Repository layout](docs/repository-layout.md) — path responsibilities and
  ownership boundaries.

______________________________________________________________________

## Licence

ISC — see [LICENSE](LICENSE) for details.

______________________________________________________________________

## Contributing

Contributions are welcome. Please see [AGENTS.md](AGENTS.md) and the
[developers' guide](docs/developers-guide.md) for the local workflow and
quality gates.
