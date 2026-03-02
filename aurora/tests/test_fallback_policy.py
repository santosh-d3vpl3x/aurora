import pytest

from aurora.core.backend.capabilities import BackendCertificationLevel
from aurora.core.routing.planner import RouteCandidate, RoutePlanner


def test_v0_blocks_mixed_backend_plan_when_auto_policy_enabled() -> None:
    planner = RoutePlanner(fallback_policy="auto")
    candidates = [
        RouteCandidate(
            engine="datafusion",
            device="cpu",
            certification=BackendCertificationLevel.CERTIFIED,
        ),
        RouteCandidate(
            engine="duckdb",
            device="cpu",
            certification=BackendCertificationLevel.CERTIFIED,
        ),
    ]

    with pytest.raises(ValueError, match="mixed-backend"):
        planner.plan(candidates)


def test_v0_allows_device_fallback_within_same_engine() -> None:
    planner = RoutePlanner(fallback_policy="auto")
    candidates = [
        RouteCandidate(
            engine="datafusion",
            device="gpu",
            certification=BackendCertificationLevel.CERTIFIED,
        ),
        RouteCandidate(
            engine="datafusion",
            device="cpu",
            certification=BackendCertificationLevel.CERTIFIED,
        ),
    ]

    winner = planner.plan(candidates)
    assert winner.engine == "datafusion"
    assert winner.device == "cpu"
