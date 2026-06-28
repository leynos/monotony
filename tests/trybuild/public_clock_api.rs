use std::time::Instant;

use monotony::{MonotonicClock, StdMonotonicClock};

#[derive(Clone, Copy, Debug)]
struct FrozenClock {
    instant: Instant,
}

impl FrozenClock {
    const fn new(instant: Instant) -> Self { Self { instant } }
}

impl MonotonicClock for FrozenClock {
    fn now(&self) -> Instant { self.instant }
}

fn read_generic_clock(clock: &impl MonotonicClock) -> Instant { clock.now() }

fn read_trait_object(clock: &dyn MonotonicClock) -> Instant { clock.now() }

fn main() {
    let instant = Instant::now();
    let frozen = FrozenClock::new(instant);
    let standard = StdMonotonicClock;

    let _generic_read = read_generic_clock(&frozen);
    let _standard_read = read_generic_clock(&standard);
    let _trait_object_read = read_trait_object(&standard);
}
