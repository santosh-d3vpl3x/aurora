# Aurora Spark Connect + Distributed Polars (MVP)

This repository contains a Rust gRPC server that implements a **Spark Connect-like** API surface and executes plans with a **Polars-backed execution engine**.

## What is implemented

- `SparkConnectService/ExecutePlan` gRPC endpoint.
- Two plan variants:
  - `SqlPlan`: executes a constrained SQL subset over an in-memory `numbers` relation.
  - `ProjectionPlan`: demonstrates distributed-style projection using Polars lazy execution.
- Response payload includes JSON-encoded rows and row counts.

## Why this structure

This is an MVP skeleton intended to be expanded toward fuller Spark Connect compatibility:

- `src/server.rs`: Spark Connect service implementation and request dispatch.
- `src/engine.rs`: Polars execution engine abstraction (replaceable with a real distributed scheduler).
- `proto/spark/connect/base.proto`: protocol contract for the service.

## Run

```bash
cargo run
```

Server listens on `0.0.0.0:15002`.

## Test

```bash
cargo test
```
