# Developer Guide

This guide explains the contributor workflow for the Monotony project.

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The generated
`typos.toml` starts from the shared estate dictionary, refreshes its untracked
local cache only when the authority is newer, and then applies the narrow
repository policy in `typos.local.toml`. Edit the local policy and regenerate
the configuration rather than changing generated entries by hand.

When an HTTPS authority is unreachable, the generator may reuse the existing
tracked `typos.toml`. That connectivity-only fallback deliberately does not
apply `typos.local.toml`, so local policy edits remain unapplied until a
successful refresh regenerates the tracked configuration. HTTP status and
local persistence failures still fail the gate.

`scripts/typos_rollout_http.py` owns spelling-cache freshness, HTTPS transport
security, and refresh persistence coordination. Only `scripts/typos_rollout.py`
may compose that helper with dictionary validation; other project code must
not reuse its infrastructure internals. This boundary keeps the public
dictionary-rendering API stable while each Python source remains below the
repository's 400-line limit.

Each refresh call carries metadata, offline mode, and an optional test opener
in one immutable `RefreshOptions` value. Construct that value at the generator
or test call site; do not pass the HTTP helper's private local-source state
across the dictionary-rendering boundary.

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
