from aurora.backends.datafusion.adapter import DataFusionAdapter
from aurora.core.backend.capabilities import BackendCertificationLevel


def test_collect_routes_query_with_explicit_policy_and_profile() -> None:
    from aurora.api.session import AuroraSession

    session = AuroraSession()
    session.register_backend(
        DataFusionAdapter(),
        certification=BackendCertificationLevel.CERTIFIED,
    )

    result = session.sql("select 1").collect(
        engine="auto",
        device="auto",
        fallback_policy="auto",
        execution_profile="single_node_max",
    )

    assert result["route"]["engine"] == "datafusion"
    assert result["route"]["device"] == "cpu"
    assert result["result"]["engine"] == "datafusion"
    assert result["profile"]["attributes"]["fallback_policy"] == "auto"
    assert result["profile"]["attributes"]["execution_profile"] == "single_node_max"
