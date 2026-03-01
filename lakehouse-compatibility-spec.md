# Lakehouse Compatibility Specification

## 1) Scope

Define minimum production compatibility for:
- formats: Parquet, Delta Lake, Iceberg
- catalogs: Unity Catalog, HMS-compatible
- storage: S3/ABFS/GCS/HDFS

---

## 2) Compatibility Principles

- Semantics first: preserve Spark-like read/write/DDL expectations.
- One semantic model, many connectors.
- Backend adapters consume normalized metadata only.

---

## 3) Format Conformance

### 3.1 Parquet
Required:
- projection pushdown
- predicate pushdown
- row group pruning
- schema evolution handling (documented limits)

### 3.2 Delta Lake
Required:
- snapshot resolution
- transaction log replay
- atomic commit behavior
- conflict detection and clear error reporting

### 3.3 Iceberg
Required:
- snapshot and manifest planning
- partition transform awareness
- atomic commit lifecycle
- metadata table compatibility (where supported)

---

## 4) Catalog Conformance

Catalog operations required:
- create/drop/alter table
- namespace operations
- table metadata resolution
- auth policy integration hooks

Unity/HMS normalization contract:
- normalize to a shared `InternalTableMetadata` model before planning.

---

## 5) Storage Conformance

Connectors must support:
- list/read/write/delete
- range reads
- retries with bounded backoff
- credential refresh paths

HDFS support requirements:
- secure auth mode compatibility
- large file streaming behavior
- read-after-write and consistency caveats documented

---

## 6) Cross-Matrix Certification

Certification matrix axes:
- format x catalog x storage x backend x device

Example baseline certification targets:
- Delta + Unity + S3 + DataFusion CPU
- Iceberg + HMS + ABFS + DataFusion CPU
- Parquet + HMS + HDFS + DuckDB CPU

Any uncertified combination must be marked experimental.

---

## 7) Compatibility Test Suites

Suite categories:
- read correctness
- write correctness
- concurrent write conflict behavior
- schema evolution behavior
- metadata refresh and cache invalidation
- authz enforcement

---

## 8) User-Facing Guarantees

- Unsupported combinations fail early with actionable error text.
- Fallback does not alter table semantics.
- Explain/profile show format/catalog/storage context for debugging.

---

## 9) Release Gating

Release requires:
- all baseline matrix tests pass
- no unresolved critical data correctness issues
- migration notes for behavior changes
