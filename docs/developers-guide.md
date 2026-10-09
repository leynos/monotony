# Developer Guide

This guide explains the contributor workflow for the Monotony project.

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The gate
regenerates `typos.toml` on every run from the live shared estate dictionary
and the narrow repository policy in `typos.local.toml`, so a word added to the
shared dictionary needs no change here. Because the dictionary is live,
`typos.toml` is never drift checked in continuous integration. Edit the local
policy rather than changing generated entries by hand; any such edit is
overwritten on the next run.

`TYPOS_CONFIG_BUILDER_VERSION` in the `Makefile` pins the
`typos-config-builder` release the gate runs (currently `v0.1.3`). Raise it
together with the regenerated `typos.toml`, never on its own. The builder
requires Python 3.14 or newer, so the target passes `--python 3.14` and `uv`
fetches that interpreter when the host lacks one.

Fenced code blocks are ignored wholesale, but inline backtick spans are not:
the shared dictionary checks their contents like any other prose. An
intentionally US-spelled identifier quoted inline therefore needs a narrow
pattern in the `typos.local.toml` `[patterns] ignore` list, scoped to the exact
span, for example:

```toml
[patterns]
ignore = [
  "`color`",
  "`mold`",
]
```

Prefer that over a word-level entry in `[words] accepted`, which would also
excuse the same US spelling in ordinary prose. Move a long or repeatedly quoted
example into a fenced block instead of broadening the pattern.

Architectural rationale for the clock abstraction and the `test-util` feature
boundary lives in [clock design](clock-design.md). Path ownership and
repository boundaries live in [repository layout](repository-layout.md).

## Local Workflow

Use `make all` as the public entrypoint for formatting, linting, and tests.
`make lint` runs rustdoc, Clippy, and Whitaker. `make test` prefers
`cargo nextest run` and falls back to `cargo test` when cargo-nextest is not
available. `make test-fast` is an alias for `make test`, which links with
`mold` by default on Linux. Compile-time API contracts live under
`tests/trybuild/` and run through the same test entrypoint with `trybuild`.
`make audit` derives the Rust workspace root with `cargo metadata`, logs
workspace member manifests, and runs `cargo audit` once from the workspace root.
`make coverage` uses `cargo llvm-cov` with `lld`.

The test suite runs once per pull request, in `ci.yml`'s coverage step. That
step runs the same tests `make test` runs except the doctests, which
`build-test` runs in a step of its own with
`cargo test --doc --workspace --all-features`. The repository used to carry an
`act-validation.yml` workflow that ran `make test WITH_ACT=1`, but nothing reads
`WITH_ACT` and no test is gated on Act, so that workflow ran the whole suite a
second time and was removed. The coverage steps in `ci.yml` and
`coverage-main.yml` pass `features: test-util`, the crate's one feature, so
`make test`'s `--all-features` selects the same tests as the coverage run.
`tests/workflow_suite_contract.rs` holds the split.

## Coverage publication

Main owns both persistent coverage outputs, following concordat's CV-005 rule.
[ADR 001](adr-001-main-owned-coverage-publication.md) records the decision and
its rationale; this section is the operational summary. Pull requests measure
coverage in `ci.yml` with `with-ratchet: 'true'` and
`publish-artefact: 'false'`, so they check the ratchet against the stored
baseline and do nothing else: no pull request uploads a report, runs
`cs-coverage`, receives `CS_ACCESS_TOKEN`, or contacts `codescene.io`.
CodeScene accepts an upload only for an analysed branch, which a pull request
head is not, and its check mode fails on every project whose coverage gates are
off. What the split takes off the pull request is the call to the service; the
CLI archive is already pinned by digest.

`.github/workflows/coverage-main.yml` is the one publisher. It runs on a push to
`main` and on dispatch, writes the ratchet baseline, and uploads to CodeScene
only when both hold:

- a `Check CodeScene token` step, whose sole command is
  `echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT"`,
  reports the token as set; the expression is evaluated before the shell runs,
  so the step binds nothing;
- `github.ref == 'refs/heads/main'`, since a dispatch may name any branch.

The upload passes the token only as `access-token`, never through an `env`: the
upload action is composite and hands its step's `env` to the nested steps it
runs. The concurrency group is `${{ github.workflow }}-${{ github.ref }}` and
never cancels, so runs for the same ref never overlap and, for triggered runs
(push and dispatch), uploads land in commit order and the newest baseline wins.
Runs on other refs may overlap a `main` run, but the upload's ref conjunct
keeps them from publishing. A manual "Re-run jobs" on an older `main` run is an
operator action: it keeps its old SHA and republishes that commit's coverage
and baseline until the next push supersedes it. Two gaps are known and
accepted. A Dependabot pull request merged by the automerge workflow with
`GITHUB_TOKEN` fires no push, so it publishes nothing until the next push to
`main` (shared-actions #518). A dispatch that replaces a pending push uploads
the same or a newer commit, but `generate-coverage` saves the baseline only on
a push, so the baseline stays one commit behind until the next push
(shared-actions #518).

`make test-workflow-contracts` holds the rule by running
`cv005-contracts check`, the shared contract library in `leynos/shared-actions`
(`packages/cv005-contracts`), from a full commit named by `CV005_CONTRACTS_REF`
in the Makefile, and CI runs it in a "Check the CV-005 contracts" step. A fix
to the rules is therefore a pin bump. The target needs `uv`, which fetches the
Python 3.13 the library runs under. The repository's only parameter is
`repository` in `.github/cv005.toml`. The library's own suite proves each rule
refuses the shape it exists to refuse, so this repository keeps no copy of the
readers or the refusal cases. The pull-request clauses run over every workflow
a pull request can reach through local `uses:` calls, the host and token
clauses read every scalar in each document, the upload condition is split on
`&&` with any `||` refused, and workflows are read strictly: a duplicate key is
refused rather than silently resolved, and a reading failure exits 2 rather
than passing.

## Clock extension boundary

`MonotonicClock` remains the stable one-method production trait. Add
elapsed-time conveniences to `MonotonicClockExt` instead of adding default
methods to `MonotonicClock`, so downstream implementors do not need to update
their core trait implementations for minor releases.

Keep `MonotonicClockExt` limited to monotonic measurement. Sleeping, timers,
timeouts, retry cadence, async runtime integration, and accelerated logical
time are consumer-owned policy and must stay outside Monotony's production API.

## Test utility boundary

Deterministic clocks live behind the opt-in `test-util` feature so downstream
integration tests can use them without changing Monotony's default production
surface. `ManualMonotonicClock` is the single-owner manual test clock, while
`SharedManualMonotonicClock` is the cloneable test helper for cases where code
under test owns one clock handle and the test advances time through another.

Do not make sleeper policy part of `SharedManualMonotonicClock`. Tests that
need waiting behaviour should define a local sleeper or timer adapter and use
the shared manual clock only to observe and advance monotonic time.

## Tooling

Development builds use Cranelift for debug code generation. On Linux targets,
`.cargo/config.toml` configures clang to link with `mold`. `make test-fast` is
an alias for `make test`. Coverage generation uses `lld` because LLVM coverage
tooling expects LLVM-compatible linker behaviour.

Install `clang`, `lld`, `mold`, `python3`, `uv`, and `cargo-audit` before
running the full generated workflow locally on Linux.

### Security audit ignores

Security audit jobs may set `CARGO_AUDIT_IGNORES` for narrowly scoped RustSec
advisories that affect unused or tooling-only dependency paths. Keep each
ignore tied to a documented runtime impact analysis, and remove it when the
affected dependency leaves the graph or the project starts using the advised
runtime path.

## The build standard

Development, test, lint, and typecheck builds use the parallel `rustc` frontend
(`-Zthreads=8`) and, on Linux, the `mold` linker (`-Clink-arg=-fuse-ld=mold`).
These are defaults in `.cargo/config.toml`, which Cargo discovers on its own,
so a bare `cargo build` gets them. `mold` ships for Linux only, so the linker
flag lives in a Linux-only table and macOS and Windows keep their platform
linker. Cargo selects one `rustflags` source rather than merging them, so every
source repeats the same flags apart from the linker.

An assigned `RUSTFLAGS` replaces the configuration's flags, so the Makefile
recipes that set it compose the standard's flags onto any inherited value (CI's
`setup-rust` exports one). Two builds are deliberately excluded: coverage
assigns `RUSTFLAGS` without the fast flags, because a measurement should not
depend on them, and the release recipe and workflow keep the platform linker,
because they assign `RUSTFLAGS` (even an empty value displaces the
configuration). Cargo has no per-profile `rustflags`, so a direct
`cargo build --release` takes the configuration's flags unless `RUSTFLAGS` is
assigned too.

On Linux, install `mold` before building: the configuration names it, so a
build without it fails at link time. CI installs it through `setup-rust`'s
`install-mold` input. `tests/build_standard_contract.rs` holds the standard. It
reads the configuration sources, the commands `make -n` prints for each
development target on a Linux host and a macOS host (each keeping the caller's
own `RUSTFLAGS`) and for each coverage and release target on a Linux host, and
the `setup-rust` steps of the CI workflows (each must pass `install-mold`), so
a flag lost through a recipe or workflow edit fails there. The decision is
recorded in [ADR 002](adr-002-rust-build-standard.md). The contract runs
`make -n`, so a direct `cargo test` needs GNU make on the `PATH`. It fails when
`make` is missing instead of skipping, so a missing tool cannot read as a pass.

### Cranelift

Cranelift is the development-profile codegen backend. The full suite was
measured under it on the pinned `nightly-2026-05-28` on 2026-09-28: all 175
nextest tests and the doctests pass. Coverage selects LLVM explicitly
(`CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm`), because instrumentation needs it,
and release builds use the release profile, which Cranelift does not touch.
Re-measure the whole suite on the next toolchain bump; if it fails, record the
failing tests here as an exception and remove the backend from
`.cargo/config.toml`.
