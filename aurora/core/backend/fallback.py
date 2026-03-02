from __future__ import annotations

from collections.abc import Sequence
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from aurora.core.routing.planner import RouteCandidate


def enforce_v0_scope(candidates: Sequence[RouteCandidate], fallback_policy: str) -> None:
    if fallback_policy != "auto":
        return

    engines = {candidate.engine for candidate in candidates}
    if len(engines) > 1:
        raise ValueError("mixed-backend plan is disabled in v0")
