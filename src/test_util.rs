//! Deterministic monotonic clocks for downstream tests.
//!
//! These helpers are feature-gated instead of hidden behind `#[cfg(test)]` so
//! crates that depend on `monotony` can use the same deterministic clocks in
//! their own test suites.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::MonotonicClock;

/// Deterministic clock that reports one configured elapsed duration.
///
/// The clock is useful for code paths that call [`MonotonicClock::now`] exactly
/// twice, once before work starts and once after it ends.
///
/// ```
/// use std::time::Duration;
///
/// use monotony::{MonotonicClock, test_util::FixedMonotonicClock};
///
/// let clock = FixedMonotonicClock::with_elapsed(Duration::from_secs(2));
/// let started_at = clock.now();
/// let finished_at = clock.now();
///
/// assert_eq!(
///     finished_at.duration_since(started_at),
///     Duration::from_secs(2)
/// );
/// ```
#[derive(Debug)]
pub struct FixedMonotonicClock {
    clock: QueuedMonotonicClock,
}

impl FixedMonotonicClock {
    /// Creates a fixed clock that reports `elapsed` between two `now` calls.
    ///
    /// # Panics
    ///
    /// Panics if `elapsed` cannot be represented as an [`Instant`] offset from
    /// the captured start instant.
    #[must_use]
    pub fn with_elapsed(elapsed: Duration) -> Self {
        let started_at = Instant::now();
        let finished_at = add_duration(started_at, elapsed);
        Self {
            clock: QueuedMonotonicClock::from_instants([started_at, finished_at]),
        }
    }
}

impl MonotonicClock for FixedMonotonicClock {
    fn now(&self) -> Instant { self.clock.now() }
}

/// Deterministic clock that returns a queue of pre-seeded instants.
///
/// Use this helper for tests that need several precise `now` values. Exhausted
/// queues panic immediately so under-provisioned tests fail at the call site
/// that consumed too many instants.
///
/// ```
/// use std::time::{Duration, Instant};
///
/// use monotony::{MonotonicClock, test_util::QueuedMonotonicClock};
///
/// let first = Instant::now();
/// let second = first + Duration::from_millis(50);
/// let clock = QueuedMonotonicClock::from_instants([first, second]);
///
/// assert_eq!(clock.now(), first);
/// assert_eq!(clock.now(), second);
/// ```
#[derive(Debug)]
pub struct QueuedMonotonicClock {
    instants: Mutex<VecDeque<Instant>>,
}

impl QueuedMonotonicClock {
    /// Creates a clock that returns `instants` in iteration order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use monotony::{MonotonicClock, test_util::QueuedMonotonicClock};
    ///
    /// let first = Instant::now();
    /// let second = first + Duration::from_millis(50);
    /// let clock = QueuedMonotonicClock::from_instants([first, second]);
    ///
    /// assert_eq!(clock.now(), first);
    /// assert_eq!(clock.now(), second);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `instants` contains an instant earlier than the instant before
    /// it.
    #[must_use]
    pub fn from_instants(instants: impl IntoIterator<Item = Instant>) -> Self {
        let queued_instants = instants.into_iter().collect::<VecDeque<_>>();
        assert_instants_are_monotonic(&queued_instants);
        Self {
            instants: Mutex::new(queued_instants),
        }
    }

    fn with_instants<Output>(
        &self,
        operation: impl FnOnce(&mut VecDeque<Instant>) -> Output,
    ) -> Output {
        with_mutex(&self.instants, operation)
    }
}

impl MonotonicClock for QueuedMonotonicClock {
    fn now(&self) -> Instant {
        let next_instant = self.with_instants(VecDeque::pop_front);
        let Some(instant) = next_instant else {
            panic!("queued monotonic clock exhausted: no instant available");
        };
        instant
    }
}

/// Deterministic clock that advances only when tests explicitly move it.
///
/// This helper is useful for polling loops, timeout code, and other tests that
/// need one clock handle to be observed repeatedly while the test controls time
/// from the outside.
///
/// ```
/// use std::time::{Duration, Instant};
///
/// use monotony::{MonotonicClock, test_util::ManualMonotonicClock};
///
/// let started_at = Instant::now();
/// let clock = ManualMonotonicClock::new(started_at);
///
/// clock.advance(Duration::from_secs(3));
///
/// assert_eq!(
///     clock.now().duration_since(started_at),
///     Duration::from_secs(3)
/// );
/// ```
#[derive(Debug)]
pub struct ManualMonotonicClock {
    current: Mutex<Instant>,
}

impl ManualMonotonicClock {
    /// Creates a manual clock starting at `instant`.
    #[must_use]
    pub const fn new(instant: Instant) -> Self {
        Self {
            current: Mutex::new(instant),
        }
    }

    /// Advances the current instant by `duration`.
    ///
    /// # Panics
    ///
    /// Panics if `duration` cannot be represented as an [`Instant`] offset from
    /// the current instant.
    pub fn advance(&self, duration: Duration) {
        self.with_current(|current| {
            *current = add_duration(*current, duration);
        });
    }

    fn with_current<Output>(&self, operation: impl FnOnce(&mut Instant) -> Output) -> Output {
        with_mutex(&self.current, operation)
    }
}

impl MonotonicClock for ManualMonotonicClock {
    fn now(&self) -> Instant { self.with_current(|current| *current) }
}

/// Cloneable deterministic clock advanced explicitly by tests.
///
/// Use this helper when code under test needs to own one clock handle while the
/// test keeps another handle for controlling time.
///
/// ```
/// use std::time::{Duration, Instant};
///
/// use monotony::{MonotonicClock, test_util::SharedManualMonotonicClock};
///
/// fn has_timed_out(clock: &dyn MonotonicClock, started_at: Instant) -> bool {
///     clock.now().duration_since(started_at) >= Duration::from_secs(5)
/// }
///
/// let started_at = Instant::now();
/// let observed_clock = SharedManualMonotonicClock::new(started_at);
/// let controller = observed_clock.clone();
///
/// assert!(!has_timed_out(&observed_clock, started_at));
///
/// controller.advance(Duration::from_secs(5));
///
/// assert!(has_timed_out(&observed_clock, started_at));
/// ```
#[derive(Clone, Debug)]
pub struct SharedManualMonotonicClock {
    current: Arc<Mutex<Instant>>,
}

impl SharedManualMonotonicClock {
    /// Creates a shared manual clock starting at `instant`.
    #[must_use]
    pub fn new(instant: Instant) -> Self {
        Self {
            current: Arc::new(Mutex::new(instant)),
        }
    }

    /// Advances the current instant by `duration`.
    ///
    /// # Panics
    ///
    /// Panics if `duration` cannot be represented as an [`Instant`] offset from
    /// the current instant.
    pub fn advance(&self, duration: Duration) {
        self.with_current(|current| {
            *current = add_duration(*current, duration);
        });
    }

    fn with_current<Output>(&self, operation: impl FnOnce(&mut Instant) -> Output) -> Output {
        with_mutex(&self.current, operation)
    }
}

impl MonotonicClock for SharedManualMonotonicClock {
    fn now(&self) -> Instant { self.with_current(|current| *current) }
}

fn with_mutex<T, Output>(mutex: &Mutex<T>, operation: impl FnOnce(&mut T) -> Output) -> Output {
    match mutex.lock() {
        Ok(mut guard) => operation(&mut guard),
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            operation(&mut guard)
        }
    }
}

fn assert_instants_are_monotonic(instants: &VecDeque<Instant>) {
    let mut previous_instant = None;
    for instant in instants {
        if let Some(previous) = previous_instant {
            assert!(
                *instant >= previous,
                "queued monotonic clock instants must be non-decreasing"
            );
        }
        previous_instant = Some(*instant);
    }
}

fn add_duration(instant: Instant, duration: Duration) -> Instant {
    let Some(advanced) = instant.checked_add(duration) else {
        panic!("monotonic clock instant overflowed while advancing");
    };
    advanced
}
