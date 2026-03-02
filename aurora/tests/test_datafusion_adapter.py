from aurora.core.backend.registry import BackendRegistry


def test_datafusion_adapter_registers_cpu_capability() -> None:
    from aurora.backends.datafusion.adapter import DataFusionAdapter

    adapter = DataFusionAdapter()
    registry = BackendRegistry()
    registry.register(adapter)

    assert "cpu" in adapter.supported_devices()
    assert registry.get("datafusion", "cpu") is adapter
