from __future__ import annotations


class AuroraApiError(Exception):
    pass


class RouteSelectionError(AuroraApiError):
    pass


class BackendExecutionError(AuroraApiError):
    pass
