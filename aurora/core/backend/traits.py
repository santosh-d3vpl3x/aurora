from __future__ import annotations

from typing import Protocol


class ExecutionBackend(Protocol):
    """Minimal backend protocol for registration and route lookup."""

    engine: str
    device: str

    def execute(self, query: object) -> dict[str, object]: ...
