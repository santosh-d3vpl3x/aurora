# Observability and UI Architecture

## 1) Goal

Provide Spark-UI-grade visibility for jobs, stages, tasks, SQL plans, and storage behavior across multiple backends.

---

## 2) Event Pipeline

```
Execution Runtime
   |
   +--> Job/Stage/Task events
   +--> Operator metrics
   +--> Backend routing events
   +--> Fallback/transfer events
   |
   v
Event Normalizer
   |
   +--> Spark-compatible event log
   +--> OpenTelemetry metrics/traces/logs
   +--> Query profile API store
```

---

## 3) Canonical Event Model

Event families:
- lifecycle: query submitted, started, finished, failed
- execution: stage/task start/end, retries
- operator: per-node timing and row/batch counters
- routing: selected backend/device, selection_reason, policy
- fallback: reason codes and boundary locations
- transfer: bytes/time across backend/device boundaries

Transfer policy annotation:
- backend-boundary transfer events must include whether transfer occurred due to fallback.
- cross-backend transfer without fallback reason is invalid.

Required correlation keys:
- query_id
- job_id
- stage_id
- task_id
- fragment_id

---

## 4) Spark UI Mapping

Mapping table:
- Query lifecycle -> SQL tab
- Stage/task timeline -> Stages tab
- task metrics and retries -> Executors/Tasks view
- storage and cache metrics -> Storage view
- DAG + boundary nodes -> Jobs/SQL graph views

Additional extensions:
- backend/device badges per stage
- fallback counts and reason drill-down
- transfer cost overlays on DAG edges

---

## 5) Metrics Taxonomy

Core metrics:
- `query.duration_ms`
- `query.rows_out`
- `backend.selected{engine,device}`
- `fallback.count{reason}`
- `transfer.bytes{src,dst}`
- `transfer.duration_ms{src,dst}`
- `operator.cpu_ms` / `operator.gpu_ms`
- `spill.bytes` / `spill.events`

---

## 6) Explain and Profile Integration

Explain output includes:
- backend/device annotations
- transfer boundaries
- fallback markers

Profile output includes:
- stage/task rollups
- operator hotspots
- top fallback reasons
- top transfer edges

---

## 7) Storage and Retention

Retention tiers:
- hot: last N queries for UI
- warm: event log files for audit and replay
- long-term: compressed telemetry export

---

## 8) Reliability and Privacy

- Event emission must be non-blocking.
- Telemetry pipeline failure must not fail queries.
- Sensitive fields are redacted by policy at emitter and sink layers.

---

## 9) Testing and Validation

- event schema contract tests
- UI mapping snapshot tests
- load tests for high event volume
- redaction tests for sensitive fields
