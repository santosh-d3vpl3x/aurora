from aurora.core.observability.metrics import CORE_METRICS


def test_core_metrics_include_canonical_names() -> None:
    metric_names = {metric.name for metric in CORE_METRICS}

    assert metric_names == {
        "query.duration_ms",
        "query.rows_out",
        "backend.selected",
        "fallback.count",
        "transfer.bytes",
        "transfer.duration_ms",
        "operator.cpu_ms",
        "operator.gpu_ms",
        "spill.bytes",
        "spill.events",
    }
