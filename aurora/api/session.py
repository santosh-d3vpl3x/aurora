from __future__ import annotations

from dataclasses import dataclass
from itertools import count

from aurora.api.dataframe import DataFrame
from aurora.api.errors import BackendExecutionError, RouteSelectionError
from aurora.core.backend.capabilities import BackendCertificationLevel
from aurora.core.backend.registry import BackendRegistry
from aurora.core.backend.traits import ExecutionBackend
from aurora.core.observability.events import EventFamily, QueryEvent
from aurora.core.routing.planner import RouteCandidate, RoutePlanner


@dataclass(frozen=True)
class _RegisteredBackend:
    backend: ExecutionBackend
    certification: BackendCertificationLevel


class AuroraSession:
    _counter = count(1)

    def __init__(self) -> None:
        self._registry = BackendRegistry()
        self._registered: list[_RegisteredBackend] = []

    def register_backend(
        self,
        backend: ExecutionBackend,
        *,
        certification: BackendCertificationLevel = BackendCertificationLevel.CERTIFIED,
    ) -> None:
        self._registry.register(backend)
        self._registered.append(_RegisteredBackend(backend=backend, certification=certification))

    def sql(self, query: str) -> DataFrame:
        return DataFrame(self, query)

    def collect_query(
        self,
        query: str,
        *,
        engine: str,
        device: str,
        fallback_policy: str,
        execution_profile: str,
    ) -> dict[str, object]:
        candidates = [
            RouteCandidate(
                engine=item.backend.engine,
                device=item.backend.device,
                certification=item.certification,
            )
            for item in self._registered
            if (engine == "auto" or item.backend.engine == engine)
            and (device == "auto" or item.backend.device == device)
        ]
        if not candidates:
            raise RouteSelectionError("no route candidates matched requested controls")

        planner = RoutePlanner(fallback_policy=fallback_policy)
        selected_route = planner.plan(candidates)
        backend = self._registry.get(selected_route.engine, selected_route.device)
        if backend is None:
            raise RouteSelectionError("selected route is not registered")

        query_id = f"q-{next(self._counter)}"
        event = QueryEvent(
            query_id=query_id,
            job_id=f"{query_id}-j1",
            stage_id=f"{query_id}-s1",
            task_id=f"{query_id}-t1",
            fragment_id=f"{query_id}-f1",
            name="query.routed",
            family=EventFamily.ROUTING,
            attributes={
                "engine": selected_route.engine,
                "device": selected_route.device,
                "fallback_policy": fallback_policy,
                "execution_profile": execution_profile,
                "single_backend": True,
                "candidate_count": len(candidates),
            },
        )

        try:
            adapter_result = backend.execute({"sql": query, "query_id": query_id})
        except Exception as exc:
            raise BackendExecutionError("backend execution failed") from exc

        return {
            "route": {
                "engine": selected_route.engine,
                "device": selected_route.device,
            },
            "profile": event.to_payload(),
            "result": adapter_result,
        }
