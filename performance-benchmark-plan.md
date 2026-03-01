# Performance Benchmark Plan

## 1) Objective

Measure and guard performance for a local-first architecture that scales when needed.

Primary success criteria:
- best-in-class single-node performance on modern high-memory, high-core machines
- predictable scale-out behavior
- no semantic drift across backends

---

## 2) Benchmark Tiers

```
Tier A: Microbenchmarks (operator-level)
Tier B: Query benchmarks (end-to-end analytical SQL)
Tier C: Lakehouse workflows (read/write/merge)
Tier D: Concurrency and mixed workloads
Tier E: Scale-out stress benchmarks
```

---

## 3) Environment Matrix

Hardware profiles:
- `local_xlarge_cpu` (high core + high memory)
- `local_gpu` (single GPU + high memory)
- `cluster_small`
- `cluster_medium`

Software matrix:
- backend: datafusion, duckdb, polars, velox
- device: cpu, gpu (where supported)
- execution profile: single_node_max, distributed_scale

---

## 4) Workload Suites

### Suite 1: Single-node OLAP
- wide scans
- selective filters
- multi-join analytics
- heavy group-by and windows

### Suite 2: Lakehouse operations
- Delta append/merge patterns
- Iceberg snapshot scan and commits
- Parquet read/write with partition pruning

### Suite 3: Python-heavy workflows
- DataFrame chaining
- SQL + DataFrame mixed workloads
- UDF-intensive jobs

### Suite 4: Scale-out
- high shuffle queries
- skew scenarios
- failure and retry scenarios

---

## 5) Metrics and KPIs

Performance KPIs:
- p50/p95 query latency
- throughput (queries/hour)
- bytes scanned per second
- CPU/GPU utilization
- spill bytes
- transfer bytes/time

Correctness KPI:
- result equivalence across backends within tolerance rules

Stability KPI:
- run-to-run variance bounds

---

## 6) Baselines and Budgets

Define baseline by:
- stable reference branch
- fixed dataset snapshots
- fixed hardware profile

Regression budgets (example):
- latency regression > 5% blocks merge for critical suites
- spill increase > 10% requires investigation
- transfer byte increase > 10% requires routing review

---

## 7) CI/CD Integration

CI layers:
- PR: smoke and targeted benchmarks
- nightly: full Tier A-C suites
- weekly: Tier D-E scale and stress suites

Output artifacts:
- benchmark report markdown
- metrics JSON
- explain/profile snapshots for top regressions

---

## 8) Regression Triage Workflow

```
Detect regression
   |
   v
Classify: routing / operator / storage / catalog / runtime
   |
   v
Compare explain + profile deltas
   |
   v
Assign owner and mitigation plan
   |
   v
Re-run benchmark gate
```

---

## 9) Launch Readiness Gates

Before enabling a backend/device in `engine=auto`:
- correctness suite green
- benchmark suite within budget
- fallback reasons stable and explainable
- no critical reliability issues

---

## 10) Reporting

Weekly report includes:
- fastest backend/device by workload class
- top regressions and fixes
- fallback trendline
- single-node vs distributed crossover analysis (for user guidance, not automatic profile switching)
