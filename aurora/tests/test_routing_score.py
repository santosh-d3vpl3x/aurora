from aurora.core.backend.capabilities import BackendCertificationLevel
from aurora.core.routing.cost_model import score_route
from aurora.core.routing.planner import RouteCandidate, select_route


def test_deterministic_route_score_prefers_certified_cpu_path() -> None:
    certified_cpu = score_route(
        engine="datafusion",
        device="cpu",
        certification=BackendCertificationLevel.CERTIFIED,
    )
    experimental_gpu = score_route(
        engine="datafusion",
        device="gpu",
        certification=BackendCertificationLevel.EXPERIMENTAL,
    )

    assert certified_cpu > experimental_gpu


def test_select_route_is_deterministic_for_equal_scores() -> None:
    candidates = [
        RouteCandidate(
            engine="duckdb",
            device="cpu",
            certification=BackendCertificationLevel.CERTIFIED,
        ),
        RouteCandidate(
            engine="datafusion",
            device="cpu",
            certification=BackendCertificationLevel.CERTIFIED,
        ),
    ]

    winner_a = select_route(candidates)
    winner_b = select_route(list(reversed(candidates)))

    assert winner_a == winner_b
    assert winner_a.engine == "datafusion"
    assert winner_a.device == "cpu"
