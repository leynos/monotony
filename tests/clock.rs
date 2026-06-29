//! Public monotonic clock behaviour.

#[cfg(feature = "test-util")]
use std::time::{Duration, Instant};

#[cfg(feature = "test-util")]
use monotony::MonotonicClockExt;
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
#[cfg(feature = "test-util")]
use rstest::rstest;

#[cfg(feature = "test-util")]
#[rstest]
#[case(Duration::ZERO)]
#[case(Duration::from_millis(250))]
#[case(Duration::from_secs(5))]
fn fixed_clock_reports_configured_elapsed_duration(#[case] elapsed: Duration) {
    let clock = FixedMonotonicClock::with_elapsed(elapsed);
    let started_at = clock.now();
    let finished_at = clock.now();

    assert_eq!(finished_at.duration_since(started_at), elapsed);
}

#[cfg(feature = "test-util")]
#[test]
#[should_panic(expected = "queued monotonic clock exhausted")]
fn fixed_clock_panics_after_its_second_read() {
    let clock = FixedMonotonicClock::with_elapsed(Duration::ZERO);

    let _started_at = clock.now();
    let _finished_at = clock.now();
    let _exhausted = clock.now();
}

#[cfg(feature = "test-util")]
#[test]
fn queued_clock_returns_instants_in_insertion_order() {
    let first = Instant::now();
    let second = first + Duration::from_millis(10);
    let third = second + Duration::from_millis(25);
    let clock = QueuedMonotonicClock::from_instants([first, second, third]);

    assert_eq!(clock.now(), first);
    assert_eq!(clock.now(), second);
    assert_eq!(clock.now(), third);
}

#[cfg(feature = "test-util")]
#[test]
#[should_panic(expected = "queued monotonic clock exhausted")]
fn queued_clock_panics_when_no_instants_remain() {
    let clock = QueuedMonotonicClock::from_instants([]);

    let _exhausted = clock.now();
}

#[cfg(feature = "test-util")]
#[test]
#[should_panic(expected = "queued monotonic clock instants must be non-decreasing")]
fn queued_clock_rejects_descending_instants() {
    let first = Instant::now();
    let second = first + Duration::from_millis(10);

    let _clock = QueuedMonotonicClock::from_instants([second, first]);
}

#[cfg(feature = "test-util")]
#[rstest]
#[case(Duration::ZERO)]
#[case(Duration::from_nanos(1))]
#[case(Duration::from_mins(1))]
fn manual_clock_advances_from_initial_instant(#[case] elapsed: Duration) {
    let started_at = Instant::now();
    let clock = ManualMonotonicClock::new(started_at);

    clock.advance(elapsed);

    assert_eq!(clock.now().duration_since(started_at), elapsed);
}

#[cfg(feature = "test-util")]
#[test]
fn elapsed_since_reports_duration_from_start() {
    let started_at = Instant::now();
    let finished_at = started_at + Duration::from_millis(275);
    let clock = QueuedMonotonicClock::from_instants([finished_at]);

    assert_eq!(clock.elapsed_since(started_at), Duration::from_millis(275));
}

#[cfg(feature = "test-util")]
#[test]
fn shared_manual_clock_observes_advances_from_cloned_handle() {
    let started_at = Instant::now();
    let observer = SharedManualMonotonicClock::new(started_at);
    let controller = observer.clone();

    controller.advance(Duration::from_secs(4));

    assert_eq!(
        observer.now().duration_since(started_at),
        Duration::from_secs(4)
    );
}

#[cfg(feature = "test-util")]
#[test]
fn shared_manual_clock_accumulates_advances_across_handles() {
    let started_at = Instant::now();
    let first = SharedManualMonotonicClock::new(started_at);
    let second = first.clone();

    first.advance(Duration::from_secs(2));
    second.advance(Duration::from_secs(3));

    assert_eq!(
        first.now().duration_since(started_at),
        Duration::from_secs(5)
    );
    assert_eq!(
        second.now().duration_since(started_at),
        Duration::from_secs(5)
    );
}

#[test]
fn standard_clock_can_be_used_through_the_trait() {
    let clock: &dyn MonotonicClock = &StdMonotonicClock;
    let started_at = clock.now();
    let finished_at = clock.now();

    assert!(finished_at >= started_at);
}

#[cfg(feature = "test-util")]
proptest! {
    #[test]
    fn manual_clock_accumulates_advances(first_millis in 0_u64..1_000_000, second_millis in 0_u64..1_000_000) {
        let started_at = Instant::now();
        let first_elapsed = Duration::from_millis(first_millis);
        let second_elapsed = Duration::from_millis(second_millis);
        let expected_elapsed = Duration::from_millis(first_millis + second_millis);
        let clock = ManualMonotonicClock::new(started_at);

        clock.advance(first_elapsed);
        clock.advance(second_elapsed);

        prop_assert_eq!(clock.now().duration_since(started_at), expected_elapsed);
    }

    #[test]
    fn queued_clock_preserves_monotonic_offsets(offset_millis in proptest::collection::vec(0_u64..1_000_000, 1..32)) {
        let started_at = Instant::now();
        let mut total_elapsed = Duration::ZERO;
        let instants = offset_millis
            .into_iter()
            .map(|offset| {
                total_elapsed += Duration::from_millis(offset);
                started_at + total_elapsed
            })
            .collect::<Vec<_>>();
        let instant_count = instants.len();
        let clock = QueuedMonotonicClock::from_instants(instants);
        let mut previous = started_at;

        for _read in 0..instant_count {
            let current = clock.now();
            prop_assert!(current >= previous);
            previous = current;
        }
    }

    #[test]
    fn shared_manual_clock_preserves_elapsed_time_across_clone_interleavings(
        actions in proptest::collection::vec((any::<bool>(), 0_u64..1_000), 0..32),
    ) {
        let started_at = Instant::now();
        let observed_clock = SharedManualMonotonicClock::new(started_at);
        let mut handles = vec![observed_clock.clone()];
        let mut total_elapsed = Duration::ZERO;

        for (should_clone, advance_millis) in actions {
            if should_clone && let Some(handle) = handles.last().cloned() {
                handles.push(handle);
            }

            let elapsed = Duration::from_millis(advance_millis);

            if let Some(handle) = handles.last() {
                handle.advance(elapsed);
                total_elapsed += elapsed;
            }
        }

        for handle in handles {
            prop_assert_eq!(handle.now().duration_since(started_at), total_elapsed);
        }
        prop_assert_eq!(
            observed_clock.now().duration_since(started_at),
            total_elapsed
        );
    }
}
