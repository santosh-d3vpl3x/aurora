from aurora.core.backend.capabilities import (
    BackendCertificationLevel,
    FallbackReasonCode,
    FallbackSeverity,
    SupportLevel,
    severity_for_reason,
)


def test_capability_classification_and_reason_codes() -> None:
    assert SupportLevel.NATIVE.value == "native"
    assert FallbackReasonCode.MISSING_OPERATOR.value == "missing_operator"
    assert severity_for_reason(FallbackReasonCode.MISSING_OPERATOR) is FallbackSeverity.ERROR
    assert BackendCertificationLevel.CERTIFIED.value == "certified"
