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

use std::time::{Duration, Instant};

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

/// Convenience methods for [`MonotonicClock`] implementations.
///
/// Import this trait when code wants to calculate elapsed monotonic time
/// without adding sleep, deadline, timer, runtime, or logical-time policy to
/// the clock abstraction.
///
/// ```
/// use std::time::{Duration, Instant};
///
/// use monotony::{MonotonicClock, MonotonicClockExt};
///
/// struct FrozenClock {
///     instant: Instant,
/// }
///
/// impl MonotonicClock for FrozenClock {
///     fn now(&self) -> Instant { self.instant }
/// }
///
/// let started_at = Instant::now();
/// let clock = FrozenClock {
///     instant: started_at + Duration::from_secs(2),
/// };
///
/// assert_eq!(clock.elapsed_since(started_at), Duration::from_secs(2));
/// ```
pub trait MonotonicClockExt: MonotonicClock {
    /// Returns the duration between `started_at` and the current instant.
    ///
    /// This helper is intentionally only elapsed-time measurement. Callers keep
    /// ownership of sleeping, deadlines, retries, async runtimes, and any
    /// accelerated logical-time policy.
    #[must_use]
    fn elapsed_since(&self, started_at: Instant) -> Duration {
        self.now().duration_since(started_at)
    }
}

impl<T: MonotonicClock + ?Sized> MonotonicClockExt for T {}

/// Monotonic clock backed by [`Instant::now`].
///
/// This is the production adapter for code that should use the process'
/// native monotonic clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdMonotonicClock;

impl MonotonicClock for StdMonotonicClock {
    fn now(&self) -> Instant { Instant::now() }
}
