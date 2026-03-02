from __future__ import annotations

from aurora.core.backend.traits import ExecutionBackend


class BackendRegistry:
    """Registry keyed by (engine, device)."""

    def __init__(self) -> None:
        self._backends: dict[tuple[str, str], ExecutionBackend] = {}

    def register(self, backend: ExecutionBackend) -> None:
        key = (backend.engine, backend.device)
        self._backends[key] = backend

    def get(self, engine: str, device: str) -> ExecutionBackend | None:
        return self._backends.get((engine, device))
