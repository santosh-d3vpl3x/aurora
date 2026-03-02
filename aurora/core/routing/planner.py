from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable

from aurora.core.backend.capabilities import BackendCertificationLevel
from aurora.core.routing.cost_model import score_route


@dataclass(frozen=True)
class RouteCandidate:
    engine: str
    device: str
    certification: BackendCertificationLevel


def select_route(candidates: Iterable[RouteCandidate]) -> RouteCandidate:
    ranked = sorted(
        candidates,
        key=lambda candidate: (
            -score_route(
                engine=candidate.engine,
                device=candidate.device,
                certification=candidate.certification,
            ),
            candidate.engine,
            candidate.device,
        ),
    )
    if not ranked:
        raise ValueError("at least one route candidate is required")
    return ranked[0]


class RoutePlanner:
    def plan(self, candidates: Iterable[RouteCandidate]) -> RouteCandidate:
        return select_route(candidates)
