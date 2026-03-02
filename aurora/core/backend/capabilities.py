from __future__ import annotations

from enum import Enum


class SupportLevel(str, Enum):
    """Capability support classification for an operation."""

    NATIVE = "native"
    FALLBACK = "fallback"
    UNSUPPORTED = "unsupported"


class FallbackSeverity(str, Enum):
    """Severity level associated with a fallback reason code."""

    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


class FallbackReasonCode(str, Enum):
    """Canonical reason taxonomy for why fallback occurred."""

    MISSING_OPERATOR = "missing_operator"
    UNSUPPORTED_DATATYPE = "unsupported_datatype"
    RESOURCE_LIMIT = "resource_limit"


class BackendCertificationLevel(str, Enum):
    """Backend certification state for deterministic router preferences."""

    EXPERIMENTAL = "experimental"
    VERIFIED = "verified"
    CERTIFIED = "certified"


_REASON_SEVERITY_MAP: dict[FallbackReasonCode, FallbackSeverity] = {
    FallbackReasonCode.MISSING_OPERATOR: FallbackSeverity.ERROR,
    FallbackReasonCode.UNSUPPORTED_DATATYPE: FallbackSeverity.WARNING,
    FallbackReasonCode.RESOURCE_LIMIT: FallbackSeverity.INFO,
}


def severity_for_reason(reason: FallbackReasonCode) -> FallbackSeverity:
    """Resolve fallback severity for a canonical reason code."""

    return _REASON_SEVERITY_MAP[reason]
