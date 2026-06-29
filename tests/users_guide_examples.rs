//! Executable examples from the users' guide.

use std::time::Duration;
#[cfg(feature = "test-util")]
use std::time::Instant;

#[cfg(feature = "test-util")]
use monotony::test_util::{
    FixedMonotonicClock,
    ManualMonotonicClock,
    QueuedMonotonicClock,
    SharedManualMonotonicClock,
};
use monotony::{MonotonicClock, StdMonotonicClock};
#[cfg(feature = "test-util")]
use proptest::prelude::*;

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

#[cfg(feature = "test-util")]
trait Sleeper {
    fn sleep(&mut self, duration: Duration);
}

#[cfg(feature = "test-util")]
struct AdvancingSleeper {
    clock: SharedManualMonotonicClock,
    total_slept: Duration,
    sleep_durations: Vec<Duration>,
}

#[cfg(feature = "test-util")]
impl AdvancingSleeper {
    const fn new(clock: SharedManualMonotonicClock) -> Self {
        Self {
            clock,
            total_slept: Duration::ZERO,
            sleep_durations: Vec::new(),
        }
    }
}

#[cfg(feature = "test-util")]
impl Sleeper for AdvancingSleeper {
    fn sleep(&mut self, duration: Duration) {
        self.clock.advance(duration);
        self.total_slept += duration;
        self.sleep_durations.push(duration);
    }
}

#[cfg(feature = "test-util")]
#[derive(Clone, Copy, Debug)]
struct WaitPolicy {
    timeout: Duration,
    interval: Duration,
}

#[cfg(feature = "test-util")]
fn wait_until_timeout(
    clock: &dyn MonotonicClock,
    sleeper: &mut dyn Sleeper,
    started_at: Instant,
    policy: WaitPolicy,
) {
    assert!(
        policy.interval > Duration::ZERO,
        "wait interval must be greater than zero"
    );

    loop {
        let elapsed = clock.now().duration_since(started_at);

        if elapsed >= policy.timeout {
            break;
        }

        let remaining = policy.timeout.saturating_sub(elapsed);

        sleeper.sleep(remaining.min(policy.interval));
    }
}

#[cfg(feature = "test-util")]
#[test]
fn guide_pairs_clock_with_consumer_owned_sleeper() {
    let started_at = Instant::now();
    let observed_clock = SharedManualMonotonicClock::new(started_at);
    let mut sleeper = AdvancingSleeper::new(observed_clock.clone());

    wait_until_timeout(
        &observed_clock,
        &mut sleeper,
        started_at,
        WaitPolicy {
            timeout: Duration::from_secs(5),
            interval: Duration::from_secs(1),
        },
    );

    assert_eq!(sleeper.total_slept, Duration::from_secs(5));
    assert!(has_timed_out(&observed_clock, started_at));
}

#[cfg(feature = "test-util")]
#[test]
fn guide_clamps_final_sleep_to_remaining_timeout() {
    let started_at = Instant::now();
    let observed_clock = SharedManualMonotonicClock::new(started_at);
    let mut sleeper = AdvancingSleeper::new(observed_clock.clone());

    wait_until_timeout(
        &observed_clock,
        &mut sleeper,
        started_at,
        WaitPolicy {
            timeout: Duration::from_secs(5),
            interval: Duration::from_secs(2),
        },
    );

    assert_eq!(
        sleeper.sleep_durations,
        vec![
            Duration::from_secs(2),
            Duration::from_secs(2),
            Duration::from_secs(1)
        ]
    );
    assert_eq!(sleeper.total_slept, Duration::from_secs(5));
}

#[cfg(feature = "test-util")]
#[test]
#[should_panic(expected = "wait interval must be greater than zero")]
fn guide_rejects_zero_sleep_interval() {
    let started_at = Instant::now();
    let observed_clock = SharedManualMonotonicClock::new(started_at);
    let mut sleeper = AdvancingSleeper::new(observed_clock.clone());

    wait_until_timeout(
        &observed_clock,
        &mut sleeper,
        started_at,
        WaitPolicy {
            timeout: Duration::from_secs(5),
            interval: Duration::ZERO,
        },
    );
}

#[cfg(feature = "test-util")]
proptest! {
    #[test]
    fn guide_waiter_reaches_timeout_without_oversleeping(
        timeout_millis in 0_u64..10_000,
        interval_millis in 1_u64..1_000,
    ) {
        let started_at = Instant::now();
        let observed_clock = SharedManualMonotonicClock::new(started_at);
        let mut sleeper = AdvancingSleeper::new(observed_clock.clone());
        let timeout = Duration::from_millis(timeout_millis);
        let interval = Duration::from_millis(interval_millis);

        wait_until_timeout(
            &observed_clock,
            &mut sleeper,
            started_at,
            WaitPolicy { timeout, interval },
        );

        prop_assert_eq!(sleeper.total_slept, timeout);
        prop_assert!(sleeper
            .sleep_durations
            .iter()
            .all(|duration| *duration > Duration::ZERO && *duration <= interval));
        prop_assert!(observed_clock.now().duration_since(started_at) >= timeout);
    }
}
