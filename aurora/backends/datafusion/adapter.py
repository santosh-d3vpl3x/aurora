from __future__ import annotations


class DataFusionAdapter:
    engine = "datafusion"
    device = "cpu"

    def supported_devices(self) -> tuple[str, ...]:
        return ("cpu",)

    def execute(self, query: object) -> dict[str, object]:
        return {
            "status": "stub",
            "engine": self.engine,
            "device": self.device,
            "query": query,
        }
