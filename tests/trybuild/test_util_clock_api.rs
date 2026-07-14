//! Compile-time contract for the feature-gated deterministic clock API.

use std::time::{Duration, Instant};

use monotony::{
    MonotonicClock,
    test_util::{
        FixedMonotonicClock, ManualMonotonicClock, QueuedMonotonicClock, SharedManualMonotonicClock,
    },
};

fn assert_clock(clock: &impl MonotonicClock) {
    let _instant = clock.now();
}

fn main() {
    let started_at = Instant::now();
    let later = started_at + Duration::from_millis(1);

    let fixed = FixedMonotonicClock::with_elapsed(Duration::from_millis(1));
    let queued = QueuedMonotonicClock::from_instants([started_at, later]);
    let manual = ManualMonotonicClock::new(started_at);
    let shared = SharedManualMonotonicClock::new(started_at);
    let shared_controller = shared.clone();

    manual.advance(Duration::from_millis(1));
    shared_controller.advance(Duration::from_millis(1));

    assert_clock(&fixed);
    assert_clock(&queued);
    assert_clock(&manual);
    assert_clock(&shared);
}
