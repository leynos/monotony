# Repository layout

This document describes the generated Monotony repository layout. It is the
canonical reference for where source code, tests, configuration, automation,
and long-lived documentation belong.

## Top-level tree

The tree below shows the generated repository structure. It is intentionally
compact and omits build output such as `target/`.

```plaintext
.
├── .cargo/
│   └── config.toml
├── .github/
│   ├── dependabot.yml
│   └── workflows/
│       ├── ci.yml

├── docs/
│   ├── adr-001-main-owned-coverage-publication.md
│   ├── contents.md
│   ├── clock-design.md
│   ├── developers-guide.md
│   ├── execplans/
│   ├── repository-layout.md
│   ├── users-guide.md
│   └── ...
├── src/

│   ├── lib.rs
│   └── test_util.rs

├── tests/
│   ├── clock.rs
│   ├── compile_time.rs
│   ├── users_guide_examples.rs
│   ├── workflow_suite_contract.rs
│   ├── workflow_suite/
│   └── trybuild/
├── AGENTS.md
├── CHANGELOG.md
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── clippy.toml
├── codecov.yml
└── rust-toolchain.toml
```

## Path responsibilities

- `.cargo/config.toml`: Configures Cargo defaults for local development,
  including Linux linker and code-generation settings.
- `.github/dependabot.yml`: Configures automated dependency update checks.
- `.github/workflows/ci.yml`: Runs the generated project's continuous
  integration checks.

- `docs/`: Holds long-lived reference documentation, guides, style rules, and
  design material.
- `docs/contents.md`: Indexes the documentation set and should be updated when
  documentation files are added, renamed, or removed.
- `docs/adr-001-main-owned-coverage-publication.md`: Records the decision that
  `main` owns coverage publication and the ratchet baseline.
- `docs/clock-design.md`: Records the architectural rationale for the clock
  abstraction and `test-util` feature boundary.
- `docs/execplans/`: Holds living execution plans for substantial repository
  changes that should be reviewed before implementation.
- `docs/users-guide.md`: Explains how to use the generated project and its
  public build and test commands.
- `docs/developers-guide.md`: Explains the contributor workflow and local
  tooling used to work on the generated project.
- `docs/repository-layout.md`: Documents the repository tree and path
  responsibilities.

- `src/lib.rs`: Contains the library crate root and exported production public
  API surface.
- `src/test_util.rs`: Contains deterministic clock helpers exposed by the
  `test-util` feature for downstream tests.

- `tests/`: Holds integration, behavioural, and compile-time contract tests
  that exercise public behaviour.
- `tests/clock.rs`: Exercises the public monotonic clock API and feature-gated
  test utilities.
- `tests/compile_time.rs`: Runs `trybuild` compile-time API contract tests.
- `tests/users_guide_examples.rs`: Exercises code examples from the users'
  guide.
- `tests/workflow_suite_contract.rs`: Holds the contract that each pull
  request runs the test suite once, in the coverage step, with the doctests in
  a step of their own.
- `tests/workflow_suite/`: Contains the workflow and manifest readers that
  contract uses.
- `tests/trybuild/`: Contains downstream crate fixtures for compile-time API
  contracts.
- `AGENTS.md`: Provides repository-specific working instructions for agents and
  contributors.
- `CHANGELOG.md`: Records notable changes in each published release.
- `Cargo.toml`: Defines package metadata, dependencies, lint policy, and Cargo
  configuration.
- `LICENSE`: Records the project licence text.
- `Makefile`: Provides the public build, lint, test, coverage, and
  documentation validation commands.
- `README.md`: Introduces the project and gives the shortest useful
  getting-started path.
- `clippy.toml`: Configures Clippy lint behaviour that is not expressed
  directly in `Cargo.toml`.
- `codecov.yml`: Configures coverage reporting behaviour.
- `rust-toolchain.toml`: Pins the Rust toolchain channel and required
  components.

## Ownership boundaries

Production code must remain dependency-free. The core crate surface is limited
to the `MonotonicClock` trait and the `StdMonotonicClock` adapter.

Reusable deterministic clocks live in `src/test_util.rs` and are exposed only
through the `test-util` feature. Keep helpers in that module when they are
intended for downstream crate tests; keep private test-only fixtures inside
individual test modules when they are useful only to Monotony's own tests.

- Keep generated source code under `src/`. Add modules below `src/` when a
  feature grows beyond a small entrypoint or crate root.
- Keep reusable deterministic test helpers behind the `test-util` feature, so
  downstream crates can opt into them without relying on private `#[cfg(test)]`
  items.
- Keep black-box integration tests and externally observable workflow tests
  under `tests/`.
- Keep reusable documentation under `docs/`. Update `docs/contents.md` whenever
  a documentation file is added, renamed, or removed.
- Keep build and validation entrypoints in `Makefile`; prefer adding or
  extending a Make target over documenting an ad hoc command.
- Keep continuous integration workflow changes under `.github/workflows/` and
  dependency-update policy under `.github/dependabot.yml`.
- Do not commit generated build output such as `target/`, coverage artefacts,
  or local editor state.

## Updating this document

Update this document when the repository gains a new top-level directory, a new
long-lived documentation category, a new workflow file, or a changed ownership
boundary that would otherwise make the tree misleading.
