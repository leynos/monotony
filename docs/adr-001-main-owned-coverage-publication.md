# Architectural decision record (ADR) 001: main-owned coverage publication

## Status

Accepted (2026-09-23): `main` owns both persistent coverage outputs, the
CodeScene upload and the ratchet baseline, through one publisher workflow. Pull
requests measure coverage only for their own ratchet check.

## Date

2026-09-24.

## Context and problem statement

Monotony measures test coverage in continuous integration (CI) for two
purposes. A pull request checks that coverage has not fallen below a stored
baseline (the ratchet), and `main` reports coverage to CodeScene so that the
service can track it over time.

Before this decision, the pull-request lane in `.github/workflows/ci.yml` also
ran CodeScene's `cs-coverage` in check mode, which needed `CS_ACCESS_TOKEN` and
a call to `codescene.io`. That arrangement had three problems:

- CodeScene accepts an upload only for an analysed branch, which a pull
  request head is not.
- Check mode fails on every project whose coverage gates are switched off,
  which is now every project in the estate, so the pull-request lane was red
  for a reason no branch could fix.
- A pull request received a secret and contacted an external service for no
  result it could use.

The question was which workflow should own coverage publication and the
baseline, and what must hold for that ownership to stay safe.

## Decision drivers

- A pull request must not receive `CS_ACCESS_TOKEN` or contact `codescene.io`,
  directly or through a reusable workflow it calls.
- The ratchet baseline must come from `main`, because caches written on `main`
  are readable by every pull-request run.
- There must be exactly one publisher, so two workflows cannot race to write
  the baseline or upload different reports for one commit.
- The rule must be enforced by tests, not by convention.

## Options considered

### Option A: keep check mode on pull requests

This keeps the red lane and the secret on pull requests. Nothing a branch does
can make check mode pass while the coverage gates are off.

### Option B: main-owned publication (chosen)

The pull-request lane runs `generate-coverage` with `with-ratchet: 'true'` and
`publish-artefact: 'false'`, and nothing else.
`.github/workflows/coverage-main.yml` runs on a push to `main` and on dispatch,
writes the baseline, and uploads to CodeScene.

| Topic                           | Option A    | Option B    |
| ------------------------------- | ----------- | ----------- |
| Pull request receives token     | Yes         | No          |
| Pull request contacts CodeScene | Yes         | No          |
| Lane passes with gates off      | No          | Yes         |
| Baseline writer                 | Unspecified | `main` only |

_Table 1: Comparison of coverage publication options._

## Decision outcome / proposed direction

Adopt option B, following concordat's CV-005 rule. The publisher has the
following shape:

- A `Check CodeScene token` step whose sole command, with no `if:`, is
  `echo "available=${{ secrets.CS_ACCESS_TOKEN != '' }}" >> "$GITHUB_OUTPUT"`.
  The expression is evaluated before the shell runs, so the token enters no
  process and sits in no `env`.
- The upload step runs only when that output is `true` and
  `github.ref == 'refs/heads/main'`, since a dispatch may name any branch. The
  token reaches the upload action only through its `access-token` input: the
  action is composite and hands its step's `env` to the nested steps it runs.
- The concurrency group is `${{ github.workflow }}-${{ github.ref }}` with
  `cancel-in-progress: false`. A cancelled publisher would abandon both its
  upload and its baseline write. Runs for the same ref never overlap, and a
  newer trigger replaces any pending run. Keying the group on the event as well
  would let a dispatch and a push on `main` run side by side.

`tests/coverage_workflows.rs`, with its readers and judgements under
`tests/cv005/`, owns the rule. It checks the real workflows and proves each
clause against breaching fixtures. The pull-request clauses cover every
workflow a pull request can reach through local `uses:` calls. The host and
token clauses read every scalar in each document. The upload condition is split
on `&&` with any `||` refused, and workflows are parsed with duplicate keys
refused. The developers' guide keeps the operational summary.

## Goals and non-goals

- Goals: keep secrets and external calls off pull requests; keep one baseline
  writer on `main`; enforce the shape by tests.
- Non-goals: a freshness check comparing the publisher's SHA with the current
  `main`. The concurrency group already keeps triggered runs from publishing
  out of order.

## Known risks and limitations

- A Dependabot pull request merged by the automerge workflow with
  `GITHUB_TOKEN` fires no push, so it publishes nothing until the next push to
  `main` (shared-actions #518).
- A dispatch that replaces a pending push uploads the same or a newer commit,
  but `generate-coverage` saves the baseline only on a push, so the baseline
  stays one commit behind until the next push (shared-actions #518).
- A manual "Re-run jobs" on an older `main` run keeps its old SHA and
  republishes that commit's coverage and baseline until the next push
  supersedes it. This is an operator action, not a race.
