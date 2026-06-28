use std::time::{Duration, Instant};

use monotony::{
    MonotonicClock,
    test_util::{FixedMonotonicClock, ManualMonotonicClock, QueuedMonotonicClock},
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

    manual.advance(Duration::from_millis(1));

    assert_clock(&fixed);
    assert_clock(&queued);
    assert_clock(&manual);
}
