//! Compile-fail contract for using `test_util` without the `test-util` feature.

use std::time::Instant;

use monotony::test_util;

fn main() {
    let instant = Instant::now();
    let _clock = test_util::QueuedMonotonicClock::from_instants([instant]);
}
