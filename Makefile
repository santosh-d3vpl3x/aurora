.PHONY: setup test lint clean

setup:
	python3 -m venv .venv
	.venv/bin/pip install -e ".[dev]"

test:
	.venv/bin/pytest aurora/tests -v

lint:
	.venv/bin/ruff check aurora
	.venv/bin/ruff format --check aurora
