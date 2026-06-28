//! Monotonic clock abstractions for deterministic elapsed-time measurement.
//!
//! `monotony` provides a narrow [`MonotonicClock`] trait over [`Instant`] plus
//! a production [`StdMonotonicClock`] adapter. Consumers can inject the trait
//! wherever elapsed time matters and avoid coupling business logic to
//! [`Instant::now`].
//!
//! ```
//! use monotony::{MonotonicClock, StdMonotonicClock};
//!
//! let clock = StdMonotonicClock;
//! let started_at = clock.now();
//! let elapsed = clock.now().duration_since(started_at);
//!
//! assert!(elapsed >= std::time::Duration::ZERO);
//! ```
//!
//! Enable the `test-util` feature to use deterministic clocks from the
//! `test_util` module in downstream crate tests.

#[cfg(feature = "test-util")]
pub mod test_util;

use std::time::Instant;

/// Clock abstraction for monotonic elapsed-time measurements.
///
/// Implement this trait when code needs to measure elapsed time without
/// depending directly on the host wall clock. The trait is intentionally
/// small so it can be used across library boundaries without pulling in
/// runtime or async dependencies.
pub trait MonotonicClock: Send + Sync {
    /// Returns the current monotonic instant.
    #[must_use]
    fn now(&self) -> Instant;
}

/// Monotonic clock backed by [`Instant::now`].
///
/// This is the production adapter for code that should use the process'
/// native monotonic clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdMonotonicClock;

impl MonotonicClock for StdMonotonicClock {
    fn now(&self) -> Instant { Instant::now() }
}
