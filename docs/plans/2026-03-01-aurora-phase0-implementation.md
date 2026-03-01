# Aurora Phase 0 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the Phase 0 local-first execution baseline with one backend abstraction, deterministic router, DataFusion CPU adapter, Arrow-boundary execution, and Spark-like observability events.

**Architecture:** Implement a thin Python API over a Rust-backed execution core boundary, with deterministic route selection and strict capability gating. Phase 0 enforces one-backend-per-query, no mixed-backend split, and local runtime only while preserving extension seams for GPU and distributed profiles.

**Tech Stack:** Python, Rust (planned runtime), PyO3, Arrow C Data/C Stream interfaces, DataFusion (initial backend), PyTest.

---

### Task 1: Bootstrap package skeleton

**Files:**
- Create: `aurora/__init__.py`
- Create: `aurora/api/__init__.py`
- Create: `aurora/core/__init__.py`
- Create: `aurora/core/backend/__init__.py`
- Create: `aurora/core/routing/__init__.py`
- Create: `aurora/core/observability/__init__.py`
- Create: `aurora/core/metadata/__init__.py`
- Create: `aurora/backends/__init__.py`
- Create: `aurora/backends/datafusion/__init__.py`
- Create: `aurora/tests/__init__.py`

**Step 1: Write the failing test**

```python
def test_package_imports():
    import aurora
    assert aurora is not None
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_bootstrap.py::test_package_imports -v`
Expected: FAIL with module import/path error

**Step 3: Write minimal implementation**

Create package directories and `__init__.py` files.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_bootstrap.py::test_package_imports -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora
git commit -m "chore: bootstrap aurora package skeleton"
```

### Task 2: Define backend abstraction and registry

**Files:**
- Create: `aurora/core/backend/traits.py`
- Create: `aurora/core/backend/registry.py`
- Test: `aurora/tests/test_backend_registry.py`

**Step 1: Write the failing test**

```python
def test_registry_resolves_backend_by_engine_device():
    from aurora.core.backend.registry import BackendRegistry
    registry = BackendRegistry()
    assert registry.get("datafusion", "cpu") is not None
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_backend_registry.py::test_registry_resolves_backend_by_engine_device -v`
Expected: FAIL with missing class/method

**Step 3: Write minimal implementation**

Implement `ExecutionBackend` protocol and `BackendRegistry` with `register()` and `get()`.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_backend_registry.py::test_registry_resolves_backend_by_engine_device -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/core/backend aurora/tests/test_backend_registry.py
git commit -m "feat: add backend abstraction and registry"
```

### Task 3: Implement capability model and reason taxonomy

**Files:**
- Create: `aurora/core/backend/capabilities.py`
- Test: `aurora/tests/test_capabilities.py`

**Step 1: Write the failing test**

```python
def test_capability_classification_and_reason_codes():
    from aurora.core.backend.capabilities import SupportLevel
    assert SupportLevel.NATIVE.value == "native"
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_capabilities.py::test_capability_classification_and_reason_codes -v`
Expected: FAIL due to missing enum

**Step 3: Write minimal implementation**

Add `SupportLevel`, reason code enums, severity mapping, and backend certification levels.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_capabilities.py::test_capability_classification_and_reason_codes -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/core/backend/capabilities.py aurora/tests/test_capabilities.py
git commit -m "feat: define capability model and fallback reason taxonomy"
```

### Task 4: Implement deterministic route scorer

**Files:**
- Create: `aurora/core/routing/cost_model.py`
- Create: `aurora/core/routing/planner.py`
- Test: `aurora/tests/test_routing_score.py`

**Step 1: Write the failing test**

```python
def test_deterministic_route_score_prefers_certified_cpu_path():
    from aurora.core.routing.cost_model import score_route
    assert score_route(...) > score_route(...)
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_routing_score.py::test_deterministic_route_score_prefers_certified_cpu_path -v`
Expected: FAIL due to missing scorer

**Step 3: Write minimal implementation**

Implement fixed-weight v0 scoring and deterministic tie-breakers.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_routing_score.py::test_deterministic_route_score_prefers_certified_cpu_path -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/core/routing aurora/tests/test_routing_score.py
git commit -m "feat: add deterministic v0 route scorer"
```

### Task 5: Enforce v0 routing scope and fallback policy

**Files:**
- Modify: `aurora/core/routing/planner.py`
- Create: `aurora/core/backend/fallback.py`
- Test: `aurora/tests/test_fallback_policy.py`

**Step 1: Write the failing test**

```python
def test_v0_blocks_mixed_backend_plan_when_auto_policy_enabled():
    # expected: single-backend plan or explicit failure
    assert ...
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_fallback_policy.py::test_v0_blocks_mixed_backend_plan_when_auto_policy_enabled -v`
Expected: FAIL due to missing enforcement

**Step 3: Write minimal implementation**

Enforce one-backend-per-query in v0; allow only device fallback within same backend.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_fallback_policy.py::test_v0_blocks_mixed_backend_plan_when_auto_policy_enabled -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/core/routing/planner.py aurora/core/backend/fallback.py aurora/tests/test_fallback_policy.py
git commit -m "feat: enforce v0 fallback policy and routing scope"
```

### Task 6: Add DataFusion CPU adapter stub

**Files:**
- Create: `aurora/backends/datafusion/adapter.py`
- Test: `aurora/tests/test_datafusion_adapter.py`

**Step 1: Write the failing test**

```python
def test_datafusion_adapter_registers_cpu_capability():
    from aurora.backends.datafusion.adapter import DataFusionAdapter
    assert "cpu" in DataFusionAdapter().supported_devices()
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_datafusion_adapter.py::test_datafusion_adapter_registers_cpu_capability -v`
Expected: FAIL because adapter missing

**Step 3: Write minimal implementation**

Implement adapter surface methods with placeholder execution.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_datafusion_adapter.py::test_datafusion_adapter_registers_cpu_capability -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/backends/datafusion/adapter.py aurora/tests/test_datafusion_adapter.py
git commit -m "feat: add datafusion cpu adapter skeleton"
```

### Task 7: Add observability event model

**Files:**
- Create: `aurora/core/observability/events.py`
- Create: `aurora/core/observability/metrics.py`
- Test: `aurora/tests/test_observability_events.py`

**Step 1: Write the failing test**

```python
def test_query_event_contains_required_correlation_keys():
    event = ...
    assert {"query_id", "job_id", "stage_id", "task_id", "fragment_id"}.issubset(event)
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_observability_events.py::test_query_event_contains_required_correlation_keys -v`
Expected: FAIL due to missing event model

**Step 3: Write minimal implementation**

Define canonical event schema and metrics names.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_observability_events.py::test_query_event_contains_required_correlation_keys -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/core/observability aurora/tests/test_observability_events.py
git commit -m "feat: add canonical observability event model"
```

### Task 8: Wire Python session API

**Files:**
- Create: `aurora/api/session.py`
- Create: `aurora/api/dataframe.py`
- Create: `aurora/api/errors.py`
- Test: `aurora/tests/test_session_routing.py`

**Step 1: Write the failing test**

```python
def test_collect_routes_query_with_explicit_policy_and_profile():
    # ensures controls are accepted and routed
    assert ...
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_session_routing.py::test_collect_routes_query_with_explicit_policy_and_profile -v`
Expected: FAIL due to missing API

**Step 3: Write minimal implementation**

Provide `SparkSession`-like entry point and route invocation flow.

**Step 4: Run test to verify it passes**

Run: `pytest aurora/tests/test_session_routing.py::test_collect_routes_query_with_explicit_policy_and_profile -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora/api aurora/tests/test_session_routing.py
git commit -m "feat: wire python session api to routing pipeline"
```

### Task 9: End-to-end Phase 0 verification

**Files:**
- Modify: `README-architecture-index.md`
- Create: `aurora/tests/test_phase0_end_to_end.py`

**Step 1: Write the failing test**

```python
def test_phase0_query_uses_single_backend_and_emits_profile():
    assert ...
```

**Step 2: Run test to verify it fails**

Run: `pytest aurora/tests/test_phase0_end_to_end.py::test_phase0_query_uses_single_backend_and_emits_profile -v`
Expected: FAIL until pipeline is fully connected

**Step 3: Write minimal implementation**

Wire missing pieces so end-to-end path passes.

**Step 4: Run full test suite**

Run: `pytest aurora/tests -v`
Expected: PASS

**Step 5: Commit**

```bash
git add aurora README-architecture-index.md docs/plans/2026-03-01-aurora-phase0-implementation.md
git commit -m "feat: complete phase0 local execution baseline"
```
