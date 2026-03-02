from __future__ import annotations

from aurora.core.backend.capabilities import BackendCertificationLevel

_CERTIFICATION_WEIGHTS: dict[BackendCertificationLevel, int] = {
    BackendCertificationLevel.CERTIFIED: 30,
    BackendCertificationLevel.VERIFIED: 20,
    BackendCertificationLevel.EXPERIMENTAL: 10,
}

_DEVICE_WEIGHTS: dict[str, int] = {
    "cpu": 5,
    "gpu": 0,
}

_ENGINE_PRIORITY: dict[str, int] = {
    "datafusion": 4,
    "duckdb": 3,
    "polars": 2,
    "velox": 1,
}


def score_route(*, engine: str, device: str, certification: BackendCertificationLevel) -> int:
    certification_score = _CERTIFICATION_WEIGHTS[certification]
    device_score = _DEVICE_WEIGHTS.get(device, 0)
    engine_score = _ENGINE_PRIORITY.get(engine, 0)
    return certification_score + device_score + engine_score
