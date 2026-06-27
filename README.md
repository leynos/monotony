# Monotony

Monotony is a low-dependency Rust microcrate for injecting monotonic clocks. It
exposes a small `MonotonicClock` trait over `std::time::Instant`, a production
`StdMonotonicClock`, and feature-gated deterministic clocks for downstream
tests.

```rust
use monotony::{MonotonicClock, StdMonotonicClock};

let clock = StdMonotonicClock;
let started_at = clock.now();
let elapsed = clock.now().duration_since(started_at);
```

Enable the `test-util` feature to use deterministic clocks:

```toml
[dev-dependencies]
monotony = { version = "0.1.0", features = ["test-util"] }
```

## Documentation

- [Documentation contents](docs/contents.md)
- [User guide](docs/users-guide.md)
- [Developer guide](docs/developers-guide.md)
