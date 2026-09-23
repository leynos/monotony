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
available. Use `make test-fast` to run the same test entrypoint with the opt-in
`mold` linker route for local test builds. Compile-time API contracts live under
`tests/trybuild/` and run through the same test entrypoint with `trybuild`.
`make audit` derives the Rust workspace root with `cargo metadata`, logs
workspace member manifests, and runs `cargo audit` once from the workspace root.
`make coverage` uses `cargo llvm-cov` with `lld`.

GitHub Actions Act validation lives in `.github/workflows/act-validation.yml`.
The main `.github/workflows/ci.yml` workflow deliberately does not run
`make test WITH_ACT=1`; the separate Act workflow runs those slower
container-backed checks in parallel.

## Coverage publication

Main owns both persistent coverage outputs, following concordat's CV-005 rule.
Pull requests measure coverage in `ci.yml` with `with-ratchet: 'true'` and
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
never cancels, so runs never overlap and, for triggered runs (push and
dispatch), uploads land in commit order and the newest baseline wins. A manual
"Re-run jobs" on an older `main` run is an operator action: it keeps its old
SHA and republishes that commit's coverage and baseline until the next push
supersedes it. Two gaps are known and accepted. A Dependabot pull request
merged by the automerge workflow with `GITHUB_TOKEN` fires no push, so it
publishes nothing until the next push to `main` (shared-actions #518). A
dispatch that replaces a pending push uploads the same or a newer commit, but
`generate-coverage` saves the baseline only on a push, so the baseline stays
one commit behind until the next push (shared-actions #518).

`tests/coverage_workflows.rs` holds the rule. Its readers and judgements live
under `tests/cv005/`, and it proves each clause against breaching fixtures as
well as against the real workflows: the pull-request clauses run over every
workflow a pull request can reach through local `uses:` calls, the host and
token clauses read every scalar in each document, the upload condition is split
on `&&` with any `||` refused, and workflows are parsed with duplicate keys
refused.

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
`.cargo/config.toml` configures clang with the repository's LLD baseline.
`make test-fast` opts into `mold` for faster local test linking. Coverage
generation uses `lld` because LLVM coverage tooling expects LLVM-compatible
linker behaviour.

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.

### Security audit ignores

Security audit jobs may set `CARGO_AUDIT_IGNORES` for narrowly scoped RustSec
advisories that affect unused or tooling-only dependency paths. Keep each
ignore tied to a documented runtime impact analysis, and remove it when the
affected dependency leaves the graph or the project starts using the advised
runtime path.
