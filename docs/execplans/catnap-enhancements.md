# Clarify monotonic clock boundaries for catnap-style consumers

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT

## Purpose / Big Picture

After this change, users of `monotony` can see the feature-gated test clocks on
docs.rs, measure elapsed time through a minor-version convenience API, share a
manual test clock across owned handles, and understand the boundary between
`monotony` and sleeper/runtime policy.

The visible result is that generated documentation includes
`monotony::test_util::{FixedMonotonicClock, QueuedMonotonicClock, ManualMonotonicClock}`,
the public API includes a narrow `MonotonicClockExt` extension trait with
`elapsed_since`, and the users' guide shows how an application such as `catnap`
can pair `MonotonicClock` with its own `Sleeper` abstraction without moving
sleep, timers, async runtimes, or logical-time scaling into this crate.

This plan targets a SemVer-compatible `1.1.0` release shape. It must not make
existing `1.0.0` implementors of `MonotonicClock` change their code.

## Constraints

Hard invariants that must hold throughout implementation:

- Keep production code dependency-free. Do not add external crates.
- Keep `test-util` opt-in. The default feature set must continue to expose only
  the production clock API.
- Do not remove or rename `ManualMonotonicClock`, `FixedMonotonicClock`,
  `QueuedMonotonicClock`, `MonotonicClock`, or `StdMonotonicClock`.
- Do not change the signature of `MonotonicClock::now`.
- Do not add `elapsed_since` directly to `MonotonicClock`; expose it through a
  blanket extension trait so existing implementors remain unaffected.
- Keep all Rust source files below 400 lines.
- Every Rust module must retain a module-level `//!` comment.
- Public APIs must have Rustdoc examples where useful.
- Documentation changes must follow `docs/documentation-style-guide.md`,
  including 80-column Markdown wrapping.
- Follow Red-Green-Refactor for code behaviour where practical.
- Stop after drafting this plan until explicit approval is given.

If satisfying the objective requires violating a constraint, do not proceed.
Document the conflict in `Decision Log` and escalate.

## Tolerances

These thresholds trigger escalation rather than workaround:

- Scope: if implementation requires changing more than 8 files or more than
  250 net lines, stop and escalate.
- Interface: if satisfying the work requires changing an existing public API
  signature or making `test-util` part of default features, stop and escalate.
- Dependencies: if any new external dependency appears necessary, stop and
  escalate.
- Shared state: if the cloneable manual clock requires anything broader than
  `Arc<Mutex<Instant>>` or equivalent standard-library shared ownership, stop
  and present options.
- Tests: if focused tests still fail after 3 implementation attempts, stop and
  document the failure.
- Ambiguity: if `ManualMonotonicClock` must itself become cloneable instead of
  adding a separate shared type, stop and ask for direction because that
  changes existing type semantics.

## Risks

- Risk: `cargo doc` may not precisely simulate docs.rs feature selection.
  Severity: low. Likelihood: medium. Mitigation: add
  `[package.metadata.docs.rs] features = ["test-util"]` and validate locally
  with `cargo doc --no-deps --features test-util`.

- Risk: adding `elapsed_since` directly to `MonotonicClock` could create a
  SemVer hazard for existing `1.0.0` implementors or downstream extension
  traits. Severity: medium. Likelihood: medium. Mitigation: add
  `MonotonicClockExt` with a blanket implementation instead of changing the
  core trait.

- Risk: a cloneable manual clock can blur ownership semantics.
  Severity: medium. Likelihood: medium. Mitigation: add a separate
  `SharedManualMonotonicClock` behind `test-util` instead of making
  `ManualMonotonicClock` itself `Clone`.

- Risk: a `Sleeper` example could make readers think `monotony` owns sleeping.
  Severity: medium. Likelihood: low. Mitigation: place the example under a
  boundary section that says the sleeper trait is consumer-owned application
  code.

- Risk: `elapsed_since` could become policy creep.
  Severity: low. Likelihood: medium. Mitigation: implement only
  elapsed-duration calculation and do not add timeout, sleep, deadline, async,
  or scaling helpers.

## Progress

- [x] (2026-06-29 00:00Z) Loaded `rust-router` and routed the task to
  crate-boundary/API design.
- [x] (2026-06-29 00:00Z) Loaded `execplans` and observed the approval gate.
- [x] (2026-06-29 00:00Z) Inspected `Cargo.toml`, `src/lib.rs`,
  `src/test_util.rs`, `docs/clock-design.md`, `docs/users-guide.md`,
  `README.md`, and current tests.
- [x] (2026-06-29 00:00Z) Revised the plan for a `1.1.0` SemVer-compatible
  release by replacing the core-trait helper with `MonotonicClockExt`.
- [ ] Await explicit approval to implement.
- [ ] Add red tests and compile-time contracts.
- [ ] Implement Cargo metadata, extension trait, and shared manual clock.
- [ ] Update README and documentation.
- [ ] Run quality gates and record evidence.
- [ ] Commit only if all quality gates pass and the user requests a commit.

## Surprises & Discoveries

- Observation: `ManualMonotonicClock` already stores its instant in a
  `Mutex<Instant>`, so the shared version only needs shared ownership around
  the existing state model. Evidence: `src/test_util.rs` defines
  `ManualMonotonicClock { current: Mutex<Instant> }`. Impact: prefer a separate
  cloneable wrapper using `Arc<Mutex<Instant>>` rather than changing the
  existing manual clock.

- Observation: the repository already has executable users' guide examples.
  Evidence: `tests/users_guide_examples.rs` mirrors examples from
  `docs/users-guide.md`. Impact: add or update tests there for the `Sleeper`
  adapter recipe.

- Observation: adding a default method to a trait is not the cleanest
  post-`1.0.0` compatibility story in Rust. Evidence: downstream crates may
  already provide an `elapsed_since` extension trait, and method resolution can
  become ambiguous or change. Impact: this plan uses `MonotonicClockExt` for
  the convenience helper.

## Decision Log

- Decision: add `[package.metadata.docs.rs] features = ["test-util"]` to
  `Cargo.toml`. Rationale: this is the documented docs.rs mechanism for
  enabling optional feature documentation without changing default features.
  Date/Author: 2026-06-29, Codex.

- Decision: add `MonotonicClockExt::elapsed_since` rather than adding a method
  to `MonotonicClock`. Rationale: this preserves the `1.0.0` core trait
  contract while allowing a `1.1.0` minor release to add a useful convenience
  API. Date/Author: 2026-06-29, Codex.

- Decision: add a separate `SharedManualMonotonicClock` behind `test-util`.
  Rationale: a separate type gives tests cloneable shared ownership while
  preserving the existing `ManualMonotonicClock` API and semantics.
  Date/Author: 2026-06-29, Codex.

- Decision: document the clock/sleeper boundary in `docs/clock-design.md`,
  `docs/users-guide.md`, and a short README note. Rationale: the design
  document records the architectural boundary, the users' guide teaches
  application composition, and the README gives quick orientation. Date/Author:
  2026-06-29, Codex.

## Outcomes & Retrospective

No implementation has started. The expected outcome is a crate that remains
small and dependency-free while making its intended composition boundary
clearer for downstream crates such as `catnap`.

At completion, update this section with the validation evidence, the final
changed file list, and any lessons from the Red-Green-Refactor cycle.

## Context and Orientation

`monotony` is a Rust library crate. Its production API lives in `src/lib.rs`.
The crate currently exposes `MonotonicClock`, a `Send + Sync` trait with
`now() -> Instant`, and `StdMonotonicClock`, a production adapter backed by
`Instant::now()`.

Feature-gated deterministic test helpers live in `src/test_util.rs` and are
compiled only when the `test-util` feature is enabled. The existing helpers are
`FixedMonotonicClock`, `QueuedMonotonicClock`, and `ManualMonotonicClock`.

The design rationale lives in `docs/clock-design.md`. User-facing examples live
in `docs/users-guide.md` and `README.md`. Executable guide coverage lives in
`tests/users_guide_examples.rs`. Public runtime behaviour is tested in
`tests/clock.rs`, and downstream compile-time API contracts are tested through
`tests/compile_time.rs` and fixtures under `tests/trybuild/`.

A "sleeper" means application-owned code that pauses execution for a duration.
Examples include blocking `std::thread::sleep`, async runtime sleep functions,
or accelerated fake sleeps in tests. This crate should measure monotonic time,
not own that sleeping policy.

## Plan of Work

Stage A is orientation and red tests. Add the smallest tests that express the
missing behaviour before production changes.

In `tests/clock.rs`, add a test for `MonotonicClockExt::elapsed_since` using a
deterministic test clock behind `#[cfg(feature = "test-util")]`. The red
failure should be a missing `MonotonicClockExt` import or unresolved trait.

In `tests/clock.rs`, add tests for `SharedManualMonotonicClock` behind
`#[cfg(feature = "test-util")]`. One test should clone the clock, advance one
handle, and observe the new instant through the other handle. Another test
should prove advances accumulate across handles.

In `tests/trybuild/test_util_clock_api.rs`, add a downstream-style compile
fixture that imports and uses `SharedManualMonotonicClock`.

In `tests/trybuild/public_clock_api.rs`, add a downstream-style compile fixture
that imports and uses `MonotonicClockExt` with `StdMonotonicClock`.

In `tests/users_guide_examples.rs`, add an executable example for a
consumer-owned `Sleeper` trait paired with `MonotonicClock`. Keep the sleeper
implementation local to the test file. The example should prove that measuring
elapsed time and sleeping are composed by the consumer, not by `monotony`.

Stage B is implementation. In `Cargo.toml`, add:

```toml
[package.metadata.docs.rs]
features = ["test-util"]
```

In `src/lib.rs`, import `Duration` alongside `Instant`, then add this public
extension trait:

```rust
pub trait MonotonicClockExt: MonotonicClock {
    fn elapsed_since(&self, started_at: Instant) -> Duration {
        self.now().duration_since(started_at)
    }
}

impl<T: MonotonicClock + ?Sized> MonotonicClockExt for T {}
```

Add Rustdoc explaining that the helper measures elapsed monotonic time and does
not sleep, enforce deadlines, or interact with any runtime.

In `src/test_util.rs`, add:

```rust
#[derive(Clone, Debug)]
pub struct SharedManualMonotonicClock {
    current: Arc<Mutex<Instant>>,
}
```

Implement `new`, `advance`, and `MonotonicClock` for it using the same
poison-tolerant mutex helper as the existing manual clock. Add Rustdoc examples
showing one handle passed to code under test while another handle advances time.

Stage C is documentation. Update `docs/clock-design.md` with a short "Clock,
not sleeper" section stating that `monotony` measures elapsed monotonic time
and intentionally does not own sleep, timers, async runtimes, or logical-time
scaling.

Update `docs/users-guide.md` with an "Application sleeper policy" section that
shows:

```rust
trait Sleeper {
    fn sleep(&mut self, duration: Duration);
}
```

Pair that consumer-owned trait with `MonotonicClock`. Explain that downstream
crates can implement blocking, async-adapted, or accelerated test sleepers
locally.

Update `README.md` with a concise boundary note and mention
`SharedManualMonotonicClock` and `MonotonicClockExt` in the feature list.

Stage D is refactor and validation. Keep implementation small. If
`src/test_util.rs` approaches 400 lines or the shared/manual implementations
duplicate enough code to obscure behaviour, extract only a private helper that
does not change the public API. If such an abstraction is needed, document its
scope in `docs/clock-design.md` or `docs/developers-guide.md` before keeping
it, per repository policy.

## Concrete Steps

Run all commands from `/data/leynos/Projects/monotony`.

1. Add red tests only, then run:

```sh
cargo test --all-targets --all-features elapsed_since
cargo test --all-targets --all-features shared_manual
cargo test --all-targets --all-features test_util_clock_api_compiles_for_downstream_crates
cargo test --all-targets --all-features public_clock_api_compiles_for_downstream_crates
```

Expected red output includes missing items such as:

```plaintext
error[E0432]: unresolved import `monotony::MonotonicClockExt`
error[E0432]: unresolved import `monotony::test_util::SharedManualMonotonicClock`
```

1. Implement the narrow code changes in `Cargo.toml`, `src/lib.rs`, and
`src/test_util.rs`.

2. Run focused green checks:

```sh
cargo test --all-targets --all-features elapsed_since
cargo test --all-targets --all-features shared_manual
cargo test --all-targets --all-features test_util_clock_api_compiles_for_downstream_crates
cargo test --all-targets --all-features public_clock_api_compiles_for_downstream_crates
```

Expected green output is successful test completion.

1. Update `docs/clock-design.md`, `docs/users-guide.md`, `README.md`, and
`tests/users_guide_examples.rs`.

2. Run documentation checks:

```sh
cargo doc --no-deps --features test-util
cargo test --doc --workspace --all-features
```

Expected output is successful documentation generation and passing doctests.

1. Run repository gates:

```sh
make check-fmt
make lint
make test
make markdownlint
```

If Markdown diagrams are touched or added, also run:

```sh
make nixie
```

## Validation and Acceptance

Acceptance criteria:

- `Cargo.toml` contains `[package.metadata.docs.rs]` with
  `features = ["test-util"]`.
- Local docs generated with `cargo doc --no-deps --features test-util` include
  `monotony::test_util`.
- `MonotonicClock` remains source-compatible with existing implementors.
- `MonotonicClockExt::elapsed_since(started_at)` returns
  `self.now().duration_since(started_at)`.
- `SharedManualMonotonicClock` is available only with `test-util`, is
  cloneable, and shares time across cloned handles.
- `ManualMonotonicClock` remains available and keeps its existing API.
- The users' guide and README explain that `monotony` is a clock abstraction,
  not a sleeper/timer/runtime abstraction.
- The consumer-owned `Sleeper` recipe is covered by
  `tests/users_guide_examples.rs`.
- The default feature build still keeps `monotony::test_util` unavailable, as
  covered by the existing compile-fail trybuild test.

Red-Green-Refactor evidence to record during implementation:

- Red: missing `MonotonicClockExt` and missing `SharedManualMonotonicClock`
  after adding focused tests.
- Green: focused tests pass after minimal implementation.
- Refactor: full `make check-fmt`, `make lint`, `make test`, and
  `make markdownlint` pass after documentation and cleanup.

Quality criteria:

- Tests: focused tests plus `make test` pass.
- Lint/typecheck: `make lint` passes with warnings denied.
- Formatting: `make check-fmt` passes.
- Documentation: `cargo doc --no-deps --features test-util`,
  `cargo test --doc --workspace --all-features`, and `make markdownlint` pass.
- Dependencies: no new dependencies are added.

## Idempotence and Recovery

All planned edits are ordinary text changes and are safe to re-run. If a test
fixture update for `trybuild` produces changed stderr unexpectedly, inspect the
compiler output before accepting it. Do not update trybuild stderr snapshots
unless the failure is an intentional public contract change.

If formatting changes Markdown wrapping, review the affected sections to ensure
the examples still match `tests/users_guide_examples.rs`.

Rollback is a normal Git revert or patch reversal of the touched files:
`Cargo.toml`, `src/lib.rs`, `src/test_util.rs`, `tests/clock.rs`,
`tests/trybuild/public_clock_api.rs`, `tests/trybuild/test_util_clock_api.rs`,
`tests/users_guide_examples.rs`, `docs/clock-design.md`, `docs/users-guide.md`,
and `README.md`.

## Artifacts and Notes

Current relevant files:

```plaintext
Cargo.toml
src/lib.rs
src/test_util.rs
tests/clock.rs
tests/compile_time.rs
tests/trybuild/public_clock_api.rs
tests/trybuild/test_util_clock_api.rs
tests/users_guide_examples.rs
docs/clock-design.md
docs/users-guide.md
README.md
```

Current source sizes are below the 400-line limit:

```plaintext
src/lib.rs       47 lines
src/test_util.rs 216 lines
```

## Interfaces and Dependencies

The final production trait must remain:

```rust
pub trait MonotonicClock: Send + Sync {
    #[must_use]
    fn now(&self) -> Instant;
}
```

The final production extension trait should be:

```rust
pub trait MonotonicClockExt: MonotonicClock {
    #[must_use]
    fn elapsed_since(&self, started_at: Instant) -> Duration {
        self.now().duration_since(started_at)
    }
}

impl<T: MonotonicClock + ?Sized> MonotonicClockExt for T {}
```

The final test utility API behind `test-util` should include:

```rust
#[derive(Clone, Debug)]
pub struct SharedManualMonotonicClock {
    current: Arc<Mutex<Instant>>,
}

impl SharedManualMonotonicClock {
    #[must_use]
    pub fn new(instant: Instant) -> Self;

    pub fn advance(&self, duration: Duration);
}

impl MonotonicClock for SharedManualMonotonicClock {
    fn now(&self) -> Instant;
}
```

No external dependencies should be introduced. Use only
`std::sync::{Arc, Mutex}` and `std::time::{Duration, Instant}`.

## Revision Note

Initial file written to `docs/execplans/catnap-enhancements.md`. Compared with
the first draft discussed in conversation, this revision changes
`elapsed_since` from a default method on `MonotonicClock` to a
`MonotonicClockExt` extension trait so the enhancement fits a `1.1.0`
SemVer-compatible release without requiring existing `MonotonicClock`
implementors to change.
