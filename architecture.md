# Unified Execution Architecture (Python-First, Arrow-Centric, Local-First)

## 1) Goals and Non-Goals

### Goals
- Python is the first-class interface and stability contract.
- One backend abstraction with pluggable engines: DataFusion, Polars, Velox, DuckDB, and future engines.
- Arrow is the only cross-backend data contract.
- Single-node performance is the default optimization target.
- Distributed scaling is available without changing user-facing semantics.
- Spark-compatible semantics and Spark-UI-grade observability.
- Lakehouse baseline support: Parquet, Delta Lake, Iceberg.
- Catalog baseline support: Unity Catalog, HMS (and compatible implementations).
- Storage baseline support: object stores (S3/ABFS/GCS) and HDFS.

### Non-Goals
- No backend-specific APIs exposed to users.
- No backend-specific in-memory format crossing adapter boundaries.
- No forced distributed execution when local execution is faster and safe.

---

## 2) High-Level System Shape

```
                          +--------------------------------+
                          |        Python API Surface      |
                          | SparkSession/DataFrame/SQL/UDF |
                          +----------------+---------------+
                                           |
                                           v
                          +--------------------------------+
                          | Semantic Layer (Spark-like)    |
                          | Analyzer, Logical Plan, Rules  |
                          +----------------+---------------+
                                           |
                                           v
                          +--------------------------------+
                          |  Backend Router + Planner      |
                          |  Capabilities + Selection Rule |
                          +----------------+---------------+
                                           |
                   +-----------------------+-----------------------+
                   |                                               |
                   v                                               v
        +------------------------+                     +------------------------+
        | Profile: single_node   |                     | Profile: distributed   |
        | maximize locality       |                     | maximize throughput    |
        +------------+-----------+                     +------------+-----------+
                     |                                                |
                     +-------------------+----------------------------+
                                         |
                                         v
                          +--------------------------------+
                          | Arrow Data Plane Contract      |
                          | RecordBatch/Schema/Streams     |
                          +----------------+---------------+
                                           |
          +----------------+---------------+----------------+----------------+
          |                |                                |                |
          v                v                                v                v
 +----------------+ +----------------+            +----------------+ +----------------+
 | DataFusion     | | Polars         |            | Velox          | | DuckDB         |
 | adapter        | | adapter        |            | adapter        | | adapter        |
 | CPU first      | | CPU + GPU      |            | CPU + GPU      | | CPU first      |
 +----------------+ +----------------+            +----------------+ +----------------+
```

Core principle: users target one semantic API; internal routing chooses backend/device per policy.
Core selection rule: user-selected engine/device is authoritative in v0.

---

## 3) Core Abstractions

### 3.1 ExecutionBackend
Each backend adapter implements one interface and advertises capabilities.

Key responsibilities:
- `execute(logical_fragment, context) -> ArrowStream`
- `explain(logical_fragment, context) -> PhysicalPlanDescription`
- `capabilities() -> CapabilitySet`
- `supported_devices() -> {cpu, gpu}`

Note: fallback policy is a router concern, not a backend concern. Backends report capabilities; the router applies policy.

#### Adapter integration strategies

Backends differ in their internal APIs. Each adapter chooses the appropriate strategy:

- **Plan-native** (DataFusion, Velox): adapter translates logical fragment to backend physical plan, executes directly.
- **SQL generation** (DuckDB): adapter translates logical fragment to SQL text, executes via backend SQL interface, wraps result as ArrowStream.
- **API mapping** (Polars): adapter translates logical fragment to backend DataFrame API calls, collects result as ArrowStream.

Integration strategy is internal to the adapter. The platform sees only `execute() -> ArrowStream` and `explain() -> PhysicalPlanDescription`.

### 3.2 BackendRegistry
Registry maps `(engine, device)` to concrete adapters and readiness probes.

Examples:
- `(datafusion, cpu)`
- `(polars, cpu)`
- `(polars, gpu)`
- `(velox, cpu)`
- `(velox, gpu)`
- `(duckdb, cpu)`

### 3.3 Capability Model
Capability resolution occurs at operator-expression-type level:
- Operator kind (scan, join, agg, window, sort, write)
- Expression coverage
- Data type coverage
- Runtime constraints (memory, spill, file format feature)

Support classification:
- `Native`: backend executes directly
- `FallbackCpu`: backend cannot execute on selected device, can run on CPU path
- `Unsupported`: must route to another backend or fail in strict mode

---

## 4) Execution Profiles

### 4.1 `single_node_max` (default for powerful machines)
Optimize for:
- In-memory pipelines
- High CPU parallelism
- Vectorized operators
- Fewer exchanges and serialization boundaries
- Aggressive locality and cache-aware execution

Profile selection rule:
- Execution profile is explicitly selected by user input.
- Router must not auto-switch between `single_node_max` and `distributed_scale`.

### 4.2 `distributed_scale`
Optimize for:
- Fault containment
- Controlled shuffles
- Task retry
- Throughput and fairness across workloads

### 4.2.1 Execution Semantics

Lazy evaluation:
- DataFrame operations build a logical plan without executing.
- Execution triggers only on terminal actions: `collect`, `write`, `show`, `explain`, `profile`.

Short-circuit:
- `LIMIT` and `head(n)` terminate the execution pipeline once N rows are produced.
- Backend adapters must support early termination of Arrow streams (cancel remaining work when consumer stops pulling).

Streaming within a query:
- Execution is pipelined: backends stream Arrow RecordBatches through operators without requiring full materialization of intermediate results.
- Blocking operators (sort, hash aggregate) may buffer internally; this is backend-managed and reported via spill metrics.

Incremental/tailing reads:
- Not supported in v0. All reads are batch snapshot reads.
- Streaming append tailing (e.g., Delta table CDC) is deferred to a future phase.

---

## 4.3 Memory Management

Model: **platform-level memory budget with per-backend allocation.**

Budget:
- Platform defines a total memory budget (configurable, defaults to 75% of system memory).
- Each active backend receives a memory allocation from the platform budget.
- v0: only one backend executes at a time (whole-query routing), so the active backend gets the full budget.
- Future (multi-backend): budget is split proportionally based on active fragment count.

Per-backend integration:
- DataFusion: `MemoryPool` trait accepts external budget.
- DuckDB: `memory_limit` setting at connection level.
- Polars: streaming engine respects batch size limits; no formal memory pool — platform controls batch size and monitors RSS.
- Velox: `MemoryManager` accepts external budget.

Spill policy:
- Spill-to-disk is backend-managed (each engine has its own spill implementation).
- Platform configures spill directory and max spill size.
- Spill events are reported through observability pipeline (`spill.bytes`, `spill.events`).

Concurrent query isolation:
- v0: single-query execution assumed; no query-level memory arbitration needed.
- Future: fair-share or priority-based allocation across concurrent queries.

---

## 5) Routing and Fallback

```
Logical Plan
    |
    v
Capability Annotation Pass
    |
    +--> all Native? -------------------------> single backend/device path
    |
    +--> some FallbackCpu? -------------------> insert device transfer boundaries
    |
    +--> Unsupported? --> strict mode: fail with reason
                        warn/auto: reroute fragment/backend
```

Fallback policy:
- `strict`: fail fast if selected backend/device cannot fully execute.
- `warn`: fallback allowed, emit diagnostics and profile counters.
- `auto`: planner can reroute whole-query only when selected target is unavailable, to satisfy the explicit user request.

v0 routing scope:
- One backend per query by default.
- Cross-backend fragment splitting is deferred to later phases.
- Device fallback (GPU -> CPU) within same backend is allowed by policy.

Cross-backend policy:
- If cross-backend fragment execution is enabled in future phases, it is fallback-only.
- Cross-backend fragmenting is not used as a performance optimization strategy.

Selection behavior:
- `engine` and `device` are explicit user controls.
- `engine=auto` uses fixed backend priority list, not a cost model.
- `device=auto` defaults to CPU in v0.

---

## 6) Python-First Contract

Python API is canonical and semantically stable:
- Spark-like DataFrame and SQL semantics
- Explicit controls: `engine`, `device`, `fallback_policy`, `execution_profile`
- Stable explain/profile API regardless of backend

Python runtime design:
- Build logical plans in Python-facing semantic layer
- Execute in Rust adapters
- Exchange data through Arrow streams
- Minimize Python/Rust boundary crossings

---

## 7) Arrow as the Glue Layer

Hard rule: all inter-component and inter-backend data exchange uses Arrow.

Implications:
- Adapter boundaries are Arrow-only.
- Transfer operators are explicit in plans.
- Zero-copy where possible.
- Conversion hotspots measured and reported.

Transfer node examples:
- `DeviceTransferExec(CPU->GPU)`
- `DeviceTransferExec(GPU->CPU)`
- `BackendTransferExec(engine_a->engine_b)`

---

## 8) Lakehouse, Catalog, and Storage Integration

Platform layer is independent of backend engines.

Minimum supported matrix:
- Formats: Parquet, Delta Lake, Iceberg
- Catalogs: Unity Catalog, HMS-compatible
- Storage: S3/ABFS/GCS, HDFS

Design principle:
- Resolve metadata once in platform layer.
- Pass normalized scan/write plans to backend adapters.
- Keep backend adapters focused on execution, not catalog semantics.

---

## 9) Observability and Spark-UI Compatibility

Expose job/stage/task/operator event stream from execution runtime.

```
Runtime Events -> Event Normalizer -> Spark-Compatible Event Log -> UI/API
      |                  |                        |
      |                  +-> OpenTelemetry        +-> Spark UI bridge/custom UI
      +-> Backend/device/fallback counters
```

Required metrics:
- selected engine/device
- fallback count and reasons
- transfer bytes and transfer time
- operator-level CPU/GPU time
- spill and memory pressure indicators

User-facing artifacts:
- explain plan with backend/device tags
- query profile summary
- stage/task timeline views

---

## 10) Security and Governance

Control points:
- Catalog authorization hooks (Unity/HMS policies)
- Credential management abstraction for storage and catalogs
- Audit event stream (read/write/ddl, backend choice, fallback events)

---

## 11) Failure and Degradation Model

- Missing backend at startup: registry marks unavailable.
- GPU unavailable: device route degrades to CPU under policy.
- Unsupported operation: reroute or fail by policy.
- Storage/catalog transient failures: retry policy at connector layer.
- Strict mode guarantees deterministic failure over silent fallback.

---

## 12) Suggested Delivery Phases

```
Phase 0: Abstractions + Registry + DataFusion baseline adapter
Phase 1: Single-node optimization profile + Arrow contract hardening
Phase 2: DuckDB CPU adapter (local-first acceleration)
Phase 3: Polars CPU adapter
Phase 4: Polars GPU adapter (policy-gated)
Phase 5: Velox CPU adapter
Phase 6: Velox GPU adapter (experimental feature flag)
Phase 7: Extended routing options and full Spark-UI compatibility bridge
```

Acceptance gates per phase:
- semantic parity tests
- explain/profile correctness
- fallback reason coverage
- performance regression checks

v0 runtime choice:
- Local runtime only, with Spark-like job/stage/task events enabled from day one.
- Distributed runtime interface is defined, but multi-node execution is feature-gated until post-v0.
- When distributed mode becomes available, it is enabled only when user passes `execution_profile=distributed_scale`.

---

## 13) Key Decisions Locked

- Python is first-class and stable.
- Arrow is the universal glue format.
- One backend abstraction; many engines behind it.
- Local-first performance with scale-on-demand.
- Spark semantics and Spark-UI-grade telemetry are first-class requirements.
