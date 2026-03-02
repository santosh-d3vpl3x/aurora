import importlib


def test_package_imports() -> None:
    modules = [
        "aurora.api",
        "aurora.core.backend",
        "aurora.core.routing",
        "aurora.core.observability",
        "aurora.core.metadata",
        "aurora.backends.datafusion",
    ]

    for module in modules:
        importlib.import_module(module)
