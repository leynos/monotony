//! Executable examples from the users' guide.

use std::time::Duration;
#[cfg(feature = "test-util")]
use std::time::Instant;

#[cfg(feature = "test-util")]
use monotony::test_util::{FixedMonotonicClock, ManualMonotonicClock, QueuedMonotonicClock};
use monotony::{MonotonicClock, StdMonotonicClock};

fn measure_elapsed(clock: &dyn MonotonicClock) -> Duration {
    let started_at = clock.now();
    clock.now().duration_since(started_at)
}

#[test]
fn guide_measures_elapsed_time_with_standard_clock() {
    let elapsed = measure_elapsed(&StdMonotonicClock);

    assert!(elapsed >= Duration::ZERO);
}

#[cfg(feature = "test-util")]
#[test]
fn guide_measures_fixed_elapsed_time() {
    let clock = FixedMonotonicClock::with_elapsed(Duration::from_millis(250));

    assert_eq!(measure_elapsed(&clock), Duration::from_millis(250));
}

#[cfg(feature = "test-util")]
#[test]
fn guide_collects_queued_instants() {
    let first = Instant::now();
    let second = first + Duration::from_millis(10);
    let third = second + Duration::from_millis(20);
    let clock = QueuedMonotonicClock::from_instants([first, second, third]);

    assert_eq!(collect_ticks(&clock, 3), vec![first, second, third]);
}

#[cfg(feature = "test-util")]
fn collect_ticks(clock: &dyn MonotonicClock, count: usize) -> Vec<Instant> {
    (0..count).map(|_| clock.now()).collect()
}

#[cfg(feature = "test-util")]
#[test]
fn guide_advances_manual_time() {
    let started_at = Instant::now();
    let clock = ManualMonotonicClock::new(started_at);

    assert!(!has_timed_out(&clock, started_at));

    clock.advance(Duration::from_secs(5));

    assert!(has_timed_out(&clock, started_at));
}

#[cfg(feature = "test-util")]
fn has_timed_out(clock: &dyn MonotonicClock, started_at: Instant) -> bool {
    clock.now().duration_since(started_at) >= Duration::from_secs(5)
}
