# Data Architecture (Arrow-Centric Lakehouse Platform)

## 1) Purpose

This document defines the data-plane architecture that supports:
- Parquet, Delta Lake, Iceberg
- Unity Catalog and HMS-compatible catalogs
- Object stores and HDFS
- One semantic layer across multiple execution backends
- Python-first user experience

Arrow is the canonical in-memory contract and interoperability boundary.

---

## 2) Data Plane Layers

```
 +-----------------------------------------------------------------------+
 |                           Python / SQL Surface                        |
 +--------------------------------------+--------------------------------+
                                        |
                                        v
 +-----------------------------------------------------------------------+
 |                  Semantic Plan + Table Intent Layer                   |
 |   (reads/writes/ddl/dml; Spark-compatible semantics)                  |
 +--------------------------------------+--------------------------------+
                                        |
                                        v
 +-----------------------------------------------------------------------+
 |            Metadata Resolution + Connector Abstraction Layer          |
 |  Catalogs (Unity/HMS) + Format Resolvers (Parquet/Delta/Iceberg)     |
 +--------------------------------------+--------------------------------+
                                        |
                                        v
 +-----------------------------------------------------------------------+
 |                   Arrow Scan/Write Contract Layer                      |
 |    partition pruning | projection pushdown | schema normalization      |
 +--------------------------------------+--------------------------------+
                                        |
                                        v
 +-------------------+-------------------+-------------------+------------+
 | Object Stores     | HDFS              | Local FS          | Cache Tier |
 | S3/ABFS/GCS       | namenode+datanode | ephemeral/dev     | metadata   |
 +-------------------+-------------------+-------------------+------------+
```

---

## 3) Canonical Data Contracts

### 3.1 In-Memory Contract
- Arrow `Schema`
- Arrow `RecordBatch`
- Arrow stream semantics for execution and UDF bridges

### 3.2 Logical Table Contract
- Stable table identifier: catalog.namespace.table
- Snapshot/version pointers for Delta/Iceberg
- Partition spec abstraction
- Metadata location abstraction

`InternalTableMetadata` minimum schema:
- `table_id`
- `catalog_type`
- `format`
- `schema`
- `partition_spec`
- `location_uri`
- `properties`
- `snapshot_pointer`
- `capabilities` (read/write/merge/optimize flags)
- `format_metadata` (opaque, format-specific; see below)

#### Two-tier metadata model

Normalized tier (fields above except `format_metadata`):
- Used by routing, planning, observability, and all format-agnostic code.
- Sufficient for scan planning, capability checks, and explain output.

Format-specific tier (`format_metadata`):
- Carries format-native metadata that has no cross-format equivalent.
- Delta examples: generated columns, check constraints, column mapping mode, deletion vector config.
- Iceberg examples: partition transforms (bucket, truncate, day/hour/year), sort orders, equality delete schemas.
- Parquet examples: bloom filter columns, encoding overrides.

Access rules:
- Format-agnostic consumers (router, observability, explain) must never inspect `format_metadata`.
- Format-aware consumers (write path, schema evolution, merge planner) downcast `format_metadata` to the concrete format type.

#### Format-derived capabilities

The `capabilities` field is populated by the `TableFormat` implementation, not by the caller.

Examples:
- Iceberg: `row_delete=native`, `partition_evolution=native`, `sort_order=native`
- Delta: `row_delete=via_merge`, `partition_evolution=unsupported`, `generated_columns=native`
- Parquet (standalone): `row_delete=unsupported`, `atomic_commit=unsupported`

Capability flags are consumed by the semantic planner to validate operations before routing.

### 3.3 Write Contract
- Staged write files
- Validation and conflict checks
- Atomic commit through format-specific transaction managers

---

## 4) Unified Table Format Model

Treat formats as specialized implementations under one table-format interface.

```
                    +----------------------+
                    |  TableFormat API     |
                    | read/write/commit    |
                    +----------+-----------+
                               |
             +-----------------+-----------------+
             |                                   |
             v                                   v
   +---------------------+             +---------------------+
   | Delta Implementation|             | Iceberg Implementation
   | log + checkpoint    |             | manifest + snapshot |
   +---------------------+             +---------------------+
             |
             v
   +---------------------+
   | Parquet Data Files  |
   +---------------------+
```

Notes:
- Parquet is both standalone format and underlying data file format for lakehouse tables.
- Delta and Iceberg transaction semantics remain format-native, exposed via unified lifecycle hooks.

---

## 5) Catalog Architecture

Catalog abstraction separates metadata governance from execution engines.

### 5.1 Catalog SPI
Responsibilities:
- namespace and table discovery
- table metadata retrieval
- authn/authz hooks
- table create/alter/drop lifecycle
- external location policy checks

### 5.2 Implementations
- Unity Catalog adapter
- HMS-compatible adapter
- optional memory/test adapter

### 5.3 Catalog Normalization
Catalog responses normalize into platform-internal metadata model.

```
Unity/HMS response --> CatalogNormalizer --> InternalTableMetadata
                                            (schema, partitions,
                                             format, location,
                                             properties, version)
```

---

## 6) Storage Architecture

### 6.1 Storage Connector SPI

Foundation: platform builds on the `object_store` Rust crate for S3/ABFS/GCS/local FS connectors.

The Storage Connector SPI wraps `object_store` to add:
- credential provider integration (pluggable, isolated from execution backends)
- platform-level retry policy
- observability hooks (read/write byte counters, latency, error classification)
- URI resolution and storage context mapping

HDFS connector is custom (not covered by `object_store`) and uses `hdfs-native` or equivalent.

Common operations (provided by underlying `object_store`):
- list/read/write/delete
- multipart uploads
- range reads
- consistency and retry handling

### 6.2 Supported Backends
- S3-compatible
- ABFS
- GCS
- HDFS
- local file system for development

### 6.3 URI and Credential Model
- Storage URI parser maps to connector and security context.
- Credential providers are pluggable and isolated from execution backend logic.

---

## 7) Metadata and Cache Strategy

Two cache classes:
- Metadata cache (catalog/schema/snapshot pointers)
- File statistics cache (Parquet footers, partition stats, manifest-derived stats)

```
Request
  |
  +--> metadata cache hit? ----yes--> use cached metadata
  |                          no
  v
Catalog/Format fetch --> validate staleness --> populate cache --> continue
```

Cache requirements:
- TTL and size controls
- invalidation on commit when possible
- session and global scopes

---

## 8) Read Path

```
SQL/DataFrame Read
    |
    v
Resolve table + snapshot + schema + partition spec
    |
    v
Build scan plan with projection/predicate pushdown
    |
    v
Prune files/partitions/manifests
    |
    v
Create Arrow scan streams
    |
    v
Dispatch to execution backend adapter
```

Key optimizations:
- partition pruning
- data skipping from metadata/statistics
- column projection pushdown
- predicate pushdown where safe

---

## 9) Write Path

```
DataFrame/SQL Write
    |
    v
Normalize schema + partitioning + mode
    |
    v
Generate staged files (Arrow -> Parquet)
    |
    v
Format transaction prepare (Delta/Iceberg)
    |
    v
Conflict detection + validation
    |
    v
Atomic commit
    |
    v
Catalog/metadata refresh + cache invalidation
```

Write modes:
- append
- overwrite (partition-aware and full)
- merge/upsert where supported by format semantics

### 9.1 Concurrency and transaction semantics (v0)

Isolation guarantees:
- Reads: snapshot-consistent.
- Writes: single-statement atomicity.
- Multi-table atomic transactions: not supported in v0.

Concurrency model:
- Optimistic concurrency per table.
- Commit validates against resolved snapshot.
- On conflict: fail with stable conflict error; no silent retry.

Safe retries:
- Append writes may retry automatically when idempotence key is present.
- Overwrite and merge do not auto-retry by default.

### 9.2 Parquet file ownership

Decision: **the format library writes Parquet files.**

- Delta writes via delta-rs; Iceberg writes via iceberg-rust; standalone Parquet writes via arrow-rs/parquet crate.
- Platform provides Arrow RecordBatch streams to format write APIs.
- Execution backends handle compute (filter, join, agg). The format layer handles persistence.

Rationale:
- Format libraries embed Parquet writer config (row group sizing, compression, statistics) tuned for their transaction models.
- Cross-backend write consistency is guaranteed because the same format library writes regardless of which engine computed the result.
- Format-specific features (Delta deletion vectors, Iceberg position deletes) require format-aware writers.

Platform-level write hints (advisory; format library may override for correctness):
- Compression codec preference (zstd default, configurable)
- Target file size hint
- Row group size hint

---

## 10) Schema Evolution and Compatibility

Support matrix should include:
- additive columns
- type widening where legal
- column rename semantics (format-dependent)
- nullability rules

Rules:
- enforce format-native constraints
- emit explicit compatibility diagnostics
- avoid silent coercion that breaks Spark-like expectations

---

## 11) Data Governance and Audit

Audit event types:
- read/write/ddl operations
- commit and rollback outcomes
- catalog auth decisions
- backend/device chosen and fallback reasons

Lineage artifacts:
- source tables and snapshots
- output table/version
- transformation plan hash

---

## 12) Reliability Model

Failure classes:
- connector transient errors
- commit conflicts
- stale metadata
- partial write failures

Recovery expectations:
- idempotent retries where safe
- explicit conflict surfaces for concurrent writers
- checkpoint/log replay for recovery in Delta/Iceberg flows

Commit lifecycle states:
- `staged`
- `validated`
- `committed`
- `aborted`

Every failed write must report last successful state transition.

---

## 13) Data Architecture Quality Gates

Minimum production gates:
- format correctness tests (Parquet/Delta/Iceberg)
- catalog contract tests (Unity/HMS)
- storage interoperability tests (S3/ABFS/GCS/HDFS)
- snapshot/commit conflict tests
- schema evolution compatibility tests
- read/write performance baselines on single-node profile

---

## 14) Capacity Planning (Local-First)

Single-node first principles:
- use high memory for caching and reduced re-scan
- keep hot intermediate data in Arrow batches
- avoid distributed shuffle unless thresholds require it

Execution profile policy:
- Profile is user-selected (`single_node_max` or `distributed_scale`).
- Capacity signals are advisory diagnostics only; they do not change execution profile automatically.

---

## 15) Locked Decisions

- Arrow is the cross-component data contract.
- Catalog and storage are platform concerns, not backend adapter concerns.
- Delta/Iceberg semantics stay format-native behind a unified lifecycle API.
- Data architecture must preserve Spark-compatible expectations for Python users.
