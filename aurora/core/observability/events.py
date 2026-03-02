"""Canonical event model for Aurora observability."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any

REQUIRED_CORRELATION_KEYS: tuple[str, ...] = (
    "query_id",
    "job_id",
    "stage_id",
    "task_id",
    "fragment_id",
)


class EventFamily(StrEnum):
    """Top-level event categories emitted by the execution runtime."""

    LIFECYCLE = "lifecycle"
    EXECUTION = "execution"
    OPERATOR = "operator"
    ROUTING = "routing"
    FALLBACK = "fallback"
    TRANSFER = "transfer"


@dataclass(frozen=True, slots=True)
class QueryEvent:
    """Canonical event envelope carrying required correlation keys."""

    query_id: str
    job_id: str
    stage_id: str
    task_id: str
    fragment_id: str
    name: str
    family: EventFamily = EventFamily.LIFECYCLE
    attributes: dict[str, Any] = field(default_factory=dict)

    def to_payload(self) -> dict[str, Any]:
        """Serialize the event to the normalized payload shape."""
        return {
            "query_id": self.query_id,
            "job_id": self.job_id,
            "stage_id": self.stage_id,
            "task_id": self.task_id,
            "fragment_id": self.fragment_id,
            "name": self.name,
            "family": self.family.value,
            "attributes": dict(self.attributes),
        }
