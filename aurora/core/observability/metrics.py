"""Canonical metric names for observability telemetry."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class MetricDefinition:
    """Simple metric descriptor with optional tag keys."""

    name: str
    tags: tuple[str, ...] = ()


QUERY_DURATION_MS = MetricDefinition("query.duration_ms")
QUERY_ROWS_OUT = MetricDefinition("query.rows_out")
BACKEND_SELECTED = MetricDefinition("backend.selected", tags=("engine", "device"))
FALLBACK_COUNT = MetricDefinition("fallback.count", tags=("reason",))
TRANSFER_BYTES = MetricDefinition("transfer.bytes", tags=("src", "dst"))
TRANSFER_DURATION_MS = MetricDefinition("transfer.duration_ms", tags=("src", "dst"))
OPERATOR_CPU_MS = MetricDefinition("operator.cpu_ms")
OPERATOR_GPU_MS = MetricDefinition("operator.gpu_ms")
SPILL_BYTES = MetricDefinition("spill.bytes")
SPILL_EVENTS = MetricDefinition("spill.events")

CORE_METRICS: tuple[MetricDefinition, ...] = (
    QUERY_DURATION_MS,
    QUERY_ROWS_OUT,
    BACKEND_SELECTED,
    FALLBACK_COUNT,
    TRANSFER_BYTES,
    TRANSFER_DURATION_MS,
    OPERATOR_CPU_MS,
    OPERATOR_GPU_MS,
    SPILL_BYTES,
    SPILL_EVENTS,
)
