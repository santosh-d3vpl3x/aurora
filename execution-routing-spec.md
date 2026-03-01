# Execution Routing Specification

## 1) Objective

Specify deterministic routing from logical plans to backend/device execution using explicit user selection while preserving Python semantic stability.

v0 safety principle: explicit user intent beats automatic optimization.

---

## 2) Inputs

Routing inputs:
- logical plan
- user hints (`engine`, `device`, `fallback_policy`, `execution_profile`)
- backend capability matrix
- runtime state (GPU availability, memory pressure)
- format and catalog context

Execution profile rule:
- `execution_profile` is authoritative user input.
- Router does not auto-promote/demote profile based on observed pressure.

Additional mandatory inputs:
- backend certification level (`certified_auto|opt_in|experimental`)
- capability snapshot version (used in reproducibility key)

---

## 3) Routing Pipeline

```
Logical Plan
   |
   v
Fragmenter
   |
   v
Capability Annotator
   |
   v
Selection Resolver (user-selected engine/device)
   |
   v
Router Decision
   |
   v
Transfer/Fallback Inserter
   |
   v
Executable Physical Plan
```

---

## 4) Profiles

### `single_node_max`
Priorities:
- minimize transfers
- minimize exchange boundaries
- maximize in-memory locality
- maximize vectorized operator coverage

### `distributed_scale`
Priorities:
- throughput under concurrency
- bounded failure domains
- deterministic task scheduling and retries

Selection policy:
- Used only when explicitly requested by user.

---

## 5) Selection Model (No Cost Scoring)

v0 does not use a cost model.

Selection algorithm:
1. Read `engine` from user input.
2. Read `device` from user input.
3. Validate backend/device availability and certification.
4. Validate capability support under `fallback_policy`.
5. Build plan for the selected backend/device, or fail with actionable reason.

Cross-backend rule:
- Cross-backend fragment routing is not an optimization path.
- It is permitted only as fallback/degradation when the selected backend cannot execute required fragments under policy.

Auto semantics:
- `engine=auto`: choose from a fixed, documented priority list (not score-based).
- `device=auto`: choose `cpu` by default in v0.

Default engine priority for `engine=auto` (v0):
1. datafusion
2. duckdb
3. polars
4. velox

Selection is deterministic and reproducible for identical inputs.

---

## 6) Policy Semantics

`fallback_policy=strict`
- No reroute/fallback allowed if selected target is non-native.

`fallback_policy=warn`
- Fallback allowed with mandatory reason and profile annotation.

`fallback_policy=auto`
- Router may reroute only to satisfy explicit user request when selected target is unavailable.

v0 scope restriction:
- Reroute is whole-query only.
- Mixed-backend execution within one query is disabled by default in v0.

Future-phase rule (when mixed-backend is enabled):
- `BackendTransferExec` insertion must be justified by fallback reason codes.
- Planner must never insert cross-backend splits purely for speed heuristics.

---

## 7) Transfer Insertion Rules

Insert explicit transfer operators when crossing:
- backend boundary
- device boundary

Transfer operators:
- `BackendTransferExec`
- `DeviceTransferExec`

Transfer nodes must include estimated and actual bytes in profile output.

v0 restriction:
- `BackendTransferExec` is reserved for future phases and disabled by default.
- `DeviceTransferExec` is allowed only for GPU -> CPU fallback within same backend.

When `BackendTransferExec` is enabled in later phases:
- each transfer node must include `fallback_reason` and `source_capability_state`.

---

## 8) Determinism and Reproducibility

Router output must be reproducible for same inputs.

Plan reproducibility key:
- query hash
- config hash
- capability snapshot version
- runtime environment fingerprint

---

## 9) Failure Behavior

If no executable route exists:
- fail with consolidated reason set
- include top candidate routes and rejection reasons

---

## 10) Test Requirements

- golden routing tests by profile and policy
- transfer insertion tests
- strict mode failure tests
- runtime-state adaptive tests (GPU present/absent)

Additional v0 tests:
- deterministic-selection snapshot tests (same inputs -> same winner)
- no-mixed-backend invariant tests
