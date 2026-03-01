# Python API Architecture

## 1) Objective

Define Python as the primary and stable user interface for the platform while execution engines remain internal and replaceable.

Core rule: users code against Python semantics, not backend semantics.

---

## 2) API Surface

Primary entry points:
- `SparkSession`-like session object
- DataFrame API
- SQL API (`session.sql(...)`)
- UDF/UDAF registration API
- Explain/profile APIs

Proposed execution controls:
- `engine`: `auto|datafusion|duckdb|polars|velox`
- `device`: `auto|cpu|gpu`
- `fallback_policy`: `strict|warn|auto`
- `execution_profile`: `single_node_max|distributed_scale`

Control semantics:
- `execution_profile` is explicit user intent.
- Runtime must not override profile automatically.

Example:
```python
df = session.read.table("main.sales")
result = (
    df.filter("amount > 0")
      .groupBy("region")
      .sum("amount")
      .collect(
          engine="auto",
          device="auto",
          fallback_policy="warn",
          execution_profile="single_node_max",
      )
)
```

---

## 3) Semantic Contract

The Python contract guarantees:
- Stable Spark-like behavior for DataFrame and SQL semantics
- Stable error classes and messages for user-level failures
- Stable schema and type display conventions
- Stable explain and profile shape

The Python contract does not guarantee:
- Which backend executes each operator
- Exact physical plan details across versions

---

## 4) Runtime Architecture

```
Python Call
   |
   v
Semantic Planner (Spark-like logical rules)
   |
   v
Backend Router (engine/device/fallback)
   |
   v
Rust Adapter Execution
   |
   v
Arrow stream -> Python result materialization
```

Design goals:
- Minimize Python <-> Rust crossing count
- Prefer Arrow-native batch transfer
- Keep data movement visible in profile outputs

### 4.1 FFI contract (v0 locked)

FFI stack:
- Binding: PyO3
- Arrow bridge: Arrow C Data Interface / C Stream Interface
- Python-side table objects: PyArrow-compatible

Boundary rules:
- Plan construction may call into Rust frequently but remains metadata-only.
- Execution actions (`collect`, `write`, `profile`) cross boundary once per action.
- Execution runs with GIL released; GIL is reacquired only for Python object materialization.

Memory ownership:
- Rust owns execution buffers until exported through Arrow C interfaces.
- Exported Arrow batches carry explicit release callbacks.
- Python must not retain released buffers beyond batch lifetime.

Error mapping:
- Rust error categories map to Python error families without backend-specific leak.
- Every exception includes `query_id` and `error_class`.

---

## 5) UDF Architecture

UDF tiers:
- Tier 1: SQL/native expression mapping where possible
- Tier 2: Arrow-batch vectorized Python UDF
- Tier 3: row UDF fallback (warned, slower)

UDF execution policy:
- Preserve deterministic behavior over speed
- Show UDF cost and fallback impact in query profile

### 5.1 UDF execution protocol

Tier 1 (`native`):
- UDF is rewritten to backend-native expression node.
- No Python callback on execution path.

Tier 2 (`vectorized_arrow`):
- UDF receives Arrow batches through Arrow C stream bridge.
- Default batch size: 8192 rows (configurable).
- UDF executes in dedicated Python worker context.

Tier 3 (`row_fallback`):
- Row-at-a-time bridge only when vectorized path unavailable.
- Always emits warning-level profile annotation.

Serialization and shipping:
- v0 supports in-process Python only.
- Distributed UDF shipping is deferred until distributed runtime phase.

GPU UDF policy:
- Not supported in v0.
- GPU route with Python UDF triggers CPU fallback unless `strict`.

---

## 6) Compatibility Matrix (Python View)

Compatibility categories:
- `fully_compatible`
- `compatible_with_warning`
- `unsupported`

This matrix is versioned and published with each release so users know what works under each backend/device.

---

## 7) Explain and Profile API

Required Python methods:
- `df.explain(mode="logical|physical|analyzed")`
- `df.explain(engine_details=True)`
- `df.profile()`

Profile must include:
- selected backend/device
- fallback reasons
- transfer bytes/time
- operator-level runtime metrics

---

## 8) Error Model

Error families:
- `AnalysisError`
- `ExecutionError`
- `CapabilityError`
- `CatalogError`
- `StorageError`

Strict rule:
- `fallback_policy=strict` must fail with actionable reason, never silently degrade.

---

## 9) Versioning Policy

- Minor versions can add options and metrics fields.
- Major versions may change defaults only with migration notes.
- Backend additions must not break existing Python code.

---

## 10) Acceptance Criteria

- Python examples run unchanged across supported backends under `engine=auto`.
- Explain/profile outputs are stable and documented.
- Error messages include mitigation hints.
- UDF behavior is deterministic and tested across backends.
- GIL release and Arrow bridge behavior are covered by integration tests.
