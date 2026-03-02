from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from aurora.api.session import AuroraSession


class DataFrame:
    def __init__(self, session: AuroraSession, query: str) -> None:
        self._session = session
        self._query = query

    def collect(
        self,
        *,
        engine: str = "auto",
        device: str = "auto",
        fallback_policy: str = "auto",
        execution_profile: str = "single_node_max",
    ) -> dict[str, object]:
        return self._session.collect_query(
            self._query,
            engine=engine,
            device=device,
            fallback_policy=fallback_policy,
            execution_profile=execution_profile,
        )
