# Architecture Docs Index

This index links all architecture docs and suggests reading order by audience.

## Document Set

1. `architecture.md`
   - Unified platform and execution architecture.
2. `data-architecture.md`
   - Data-plane architecture: formats, catalogs, storage, read/write lifecycle.
3. `python-api-architecture.md`
   - Python-first contract, semantic guarantees, UDF model, and error model.
4. `semantic-layer-spec.md`
   - Semantic IR, analyzer pipeline, SQL parsing strategy, and Spark-compatibility boundary.
5. `backend-capability-matrix.md`
   - Backend/device support model, support levels, and fallback reasons.
6. `execution-routing-spec.md`
   - Routing pipeline, explicit selection rules, transfer insertion, and policy semantics.
7. `observability-and-ui-architecture.md`
   - Event model, metrics taxonomy, and Spark-UI mapping.
8. `lakehouse-compatibility-spec.md`
   - Conformance requirements for Parquet/Delta/Iceberg, Unity/HMS, and storage.
9. `performance-benchmark-plan.md`
   - Benchmark tiers, KPI definitions, CI gates, and regression triage workflow.

## Recommended Reading Order

### Platform/Engine Team
1. `architecture.md`
2. `semantic-layer-spec.md`
3. `backend-capability-matrix.md`
4. `execution-routing-spec.md`
5. `observability-and-ui-architecture.md`
6. `performance-benchmark-plan.md`

### Data/Lakehouse Team
1. `data-architecture.md`
2. `lakehouse-compatibility-spec.md`
3. `execution-routing-spec.md`
4. `performance-benchmark-plan.md`

### Python/Product API Team
1. `python-api-architecture.md`
2. `semantic-layer-spec.md`
3. `architecture.md`
4. `observability-and-ui-architecture.md`
5. `backend-capability-matrix.md`

### SRE/Performance Team
1. `observability-and-ui-architecture.md`
2. `performance-benchmark-plan.md`
3. `execution-routing-spec.md`
4. `architecture.md`

## Decision Flow (Quick Start)

```
Need the big picture? ------------------> architecture.md
Need API guarantees? -------------------> python-api-architecture.md
Need semantic rules and SQL behavior? --> semantic-layer-spec.md
Need backend/device behavior? ----------> backend-capability-matrix.md
Need routing details? ------------------> execution-routing-spec.md
Need metrics and UI mapping? -----------> observability-and-ui-architecture.md
Need format/catalog/storage guarantees? -> lakehouse-compatibility-spec.md
Need perf gates and benchmarks? --------> performance-benchmark-plan.md
```

## Maintenance Notes

- Keep this file updated whenever a new architecture document is added or renamed.
- Version-sensitive policy or matrix changes should be reflected in this index and in the target document.
- Prefer linking from this index in PR descriptions for architecture-impacting changes.
