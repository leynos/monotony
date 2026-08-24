# Changelog

## [1.0.0] - 2026-08-24

### Added

- Add `MonotonicClock` and `StdMonotonicClock` for injection ([#6]).
- Add `FixedMonotonicClock` and `QueuedMonotonicClock` test helpers ([#6]).
- Add manually advanced and shared test clocks ([#6], [#7]).
- Add `MonotonicClockExt::elapsed_since` for measuring elapsed time ([#7]).
- Document composition with application-owned sleepers and timers ([#7]).

[1.0.0]: https://github.com/leynos/monotony/releases/tag/v1.0.0
[#6]: https://github.com/leynos/monotony/pull/6
[#7]: https://github.com/leynos/monotony/pull/7
