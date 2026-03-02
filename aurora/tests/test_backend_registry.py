from aurora.core.backend.registry import BackendRegistry


class _DummyBackend:
    engine = "datafusion"
    device = "cpu"


def test_registry_resolves_backend_by_engine_device() -> None:
    registry = BackendRegistry()
    backend = _DummyBackend()

    registry.register(backend)

    assert registry.get("datafusion", "cpu") is backend
