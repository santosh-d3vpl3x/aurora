from aurora.api.session import AuroraSession
from aurora.backends.datafusion.adapter import DataFusionAdapter
from aurora.core.backend.capabilities import BackendCertificationLevel


def test_phase0_query_uses_single_backend_and_emits_profile() -> None:
    session = AuroraSession()
    session.register_backend(
        DataFusionAdapter(),
        certification=BackendCertificationLevel.CERTIFIED,
    )

    output = session.sql("select amount from sales").collect(
        engine="auto",
        device="auto",
        fallback_policy="auto",
        execution_profile="single_node_max",
    )

    assert output["route"] == {"engine": "datafusion", "device": "cpu"}
    assert output["result"]["status"] == "stub"
    assert output["profile"]["attributes"]["single_backend"] is True
    assert output["profile"]["attributes"]["execution_profile"] == "single_node_max"
