from aurora.core.observability.events import QueryEvent


def test_query_event_contains_required_correlation_keys() -> None:
    event = QueryEvent(
        query_id="q-1",
        job_id="j-1",
        stage_id="s-1",
        task_id="t-1",
        fragment_id="f-1",
        name="query.started",
    )

    payload = event.to_payload()

    assert payload["query_id"] == "q-1"
    assert payload["job_id"] == "j-1"
    assert payload["stage_id"] == "s-1"
    assert payload["task_id"] == "t-1"
    assert payload["fragment_id"] == "f-1"
