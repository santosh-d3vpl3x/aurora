# Distributed Runtime Strategy

## 1) Goal

Define the runtime path from local-first execution to distributed-scale execution without changing Python semantics.

---

## 2) v0 Decision

- v0 ships with LocalRuntime only.
- LocalRuntime still emits Spark-like job/stage/task events.
- `execution_profile=distributed_scale` is accepted but feature-gated.
- Runtime profile remains user-controlled; no auto-switching from local to distributed.

This keeps observability and scheduling contracts stable while reducing early implementation risk.

---

## 3) Target Runtime Shape (post-v0)

```
Client (Python)
   |
   v
Driver
  - logical planning
  - routing
  - stage graph build
   |
   +--> Scheduler
         - task assignment
         - retry policy
         - resource accounting
              |
              v
          Workers
          - fragment execution
          - local shuffle write/read
          - metrics + event emission
```

---

## 4) Stage and Task Model

- Stage boundary occurs at exchange/shuffle points.
- Tasks are partition-scoped stage units.
- Retry occurs at task granularity.
- Stage completion requires all task outputs materialized and registered.

---

## 5) Shuffle Contract

Shuffle data contract:
- Arrow IPC stream format
- partitioned by exchange partition key
- checksum and size metadata per partition artifact

v0: local shuffle implementation only.
post-v0: remote shuffle service or peer transfer model.

---

## 6) Scheduling Policy (initial)

- Pull-based worker task acquisition.
- Fair scheduling by query and stage.
- Resource hints: CPU cores, memory budget, GPU availability.
- Backpressure when spill/queue pressure crosses thresholds.

---

## 7) Fault Tolerance

- Driver persists stage/task state transitions.
- Worker failure triggers task reschedule.
- Lost shuffle partition triggers re-run of producing task.
- Query-level retry is opt-in for idempotent workloads.

---

## 8) Integration Options

Candidate paths:
- Integrate Apache Ballista scheduler/executor model for faster distributed bootstrap.
- Build custom scheduler aligned to Spark-compatibility requirements.

v0 recommendation:
- Keep distributed abstraction boundary clean, defer hard integration choice to post-v0 ADR.

---

## 9) Exit Criteria for Enabling Distributed Mode

- LocalRuntime parity tests green.
- Stage/task/event model stable across 3 releases.
- Shuffle correctness and retry tests green.
- Performance crossover thresholds defined and validated.
