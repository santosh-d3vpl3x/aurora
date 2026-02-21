# Aurora: Spark Connect server on distributed Polars

This repository contains a Spark Connect-style gRPC server implemented in Rust using:

- `tonic` for gRPC service exposure.
- `polars` for query planning/execution.
- A lightweight distributed-table registration abstraction that can scan partitioned parquet datasets.

## Implemented API surface

- `ExecutePlan` (streaming)
- `AnalyzePlan`
- `Config`
- `AddArtifacts` (client-streaming upload; stores batch/chunked artifacts in-memory)
- `ArtifactStatus` (returns per-artifact existence from in-memory registry)
- `Interrupt` (marks one/all operations as interrupted)
- `ReattachExecute` (replays cached execution frames after `last_response_id`)
- `ReleaseExecute` (drops cached operation frames)

## Supported commands

`ExecutePlanRequest.command` currently supports:

- `sql_command`: execute SQL over mapped parquet table paths.
- `projection_command`: perform projection (`value + add_scalar`) over inline values.
- `range_repartition_count_command`: compute `range(start, end, step).repartition(n).count()` using Polars-backed execution semantics.
- Spark Connect `plan` payloads for range/project/aggregate flows are decoded via `scripts/plan_eval.py` to support PySpark `spark.range(...).show()/count()/selectExpr(...)` execution.

`ExecutePlan` executes the DataFrame command with Polars and streams: schema, JSON rows, metrics, and `result_complete`.

Execution frames are cached by `operation_id` so `ReattachExecute`, `Interrupt`, and `ReleaseExecute` work against real operation state.

## Running

```bash
cargo run
```

Environment variables:

- `SPARK_CONNECT_ADDR` (default: `0.0.0.0:15002`)

## SQL request model

`SqlCommand.table_paths` should map logical table names to parquet glob patterns.

Example:

- table: `trips`
- path: `/data/trips/*.parquet`

Then query with:

```sql
select vendor_id, count(*) from trips group by vendor_id
```

If `table_paths` is empty, the server registers a built-in in-memory table `numbers(id, value)` for MVP usage.
