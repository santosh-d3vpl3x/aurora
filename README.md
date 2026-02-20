# Aurora: Spark Connect server on distributed Polars

This repository contains a minimal Spark Connect-compatible gRPC server implemented in Rust using:

- `tonic` for gRPC service exposure.
- `polars` for query planning/execution.
- A lightweight distributed-table registration abstraction that can scan partitioned parquet datasets.

## What is implemented

- `SparkConnectService/ExecutePlan` endpoint.
- SQL command execution (`sql_command`) over user-provided table path mappings.
- Streamed responses containing:
  1. output schema,
  2. JSON rows,
  3. execution metrics.

## Running

```bash
cargo run
```

Environment variables:

- `SPARK_CONNECT_ADDR` (default: `0.0.0.0:15002`)

## Example request model

`SqlCommand.table_paths` should map logical table names to parquet glob patterns.

Example:

- table: `trips`
- path: `/data/trips/*.parquet`

Then query with:

```sql
select vendor_id, count(*) from trips group by vendor_id
```
