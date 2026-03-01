# Aurora Development Guide

## Project
Unified query execution engine. Python-first, Arrow-centric, local-first.
Spark-compatible semantics across pluggable backends (DataFusion, DuckDB, Polars, Velox).

## Tech Stack
- Python 3.12+ (user API, tests)
- Rust (planned execution core, PyO3 bridge)
- Arrow C Data/C Stream interfaces
- pytest for testing

## Setup
make setup    # create venv, install deps
make test     # run full test suite
make lint     # ruff check + ruff format

## Architecture
Read README-architecture-index.md for the full doc index and team-specific reading order.

## Current Phase
Phase 0 — see docs/plans/2026-03-01-aurora-phase0-implementation.md
Tasks: `gh issue list --milestone "Phase 0"`

## Picking Up Work
1. Find available work: `gh issue list --milestone "Phase 0" --search "no:assignee -label:blocked"`
2. Read the issue body — spec references, acceptance criteria, dependencies
3. Verify dependencies are resolved (no `blocked` label)
4. Claim: `gh issue edit N --add-assignee @me` and add `in-progress` label
5. Branch: `phase0/issue-N-short-description`
6. Implement: TDD — failing test first, then implementation, then passing test
7. Verify: `make test && make lint`
8. PR: include `closes #N`, spec section references, test summary
9. After merge: remove `blocked` label from downstream issues if applicable

## Rules
- One issue per PR. One branch per issue.
- Branch must contain the issue number: `phase0/issue-N-...`
- Do not modify architecture docs (root *.md) without `spec-change` label and human approval.
- All tests must pass. CI enforces this.
- If blocked, comment on the issue and pick another.
- Stale claims (no PR within 24h) may be reclaimed by others.
