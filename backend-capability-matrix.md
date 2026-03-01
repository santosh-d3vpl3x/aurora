# Backend Capability Matrix

## 1) Purpose

Define how backend and device capability is evaluated and how fallback decisions are made.

Backends in scope:
- DataFusion
- DuckDB
- Polars
- Velox

Devices in scope:
- CPU
- GPU

---

## 2) Support Levels

- `NATIVE`: operator executes directly on selected backend/device
- `FALLBACK_CPU`: selected backend/device cannot execute; CPU path available
- `ROUTE_BACKEND`: must reroute operator/fragment to another backend
- `UNSUPPORTED`: no safe execution path

---

## 3) Initial Matrix (Planning Baseline)

```
+------------------+--------------+--------------+-----------------------------+
| Backend          | CPU          | GPU          | Notes                       |
+------------------+--------------+--------------+-----------------------------+
| DataFusion       | strong       | none         | baseline correctness path   |
| DuckDB           | strong       | none         | local-first acceleration    |
| Polars           | strong       | partial      | GPU coverage evolving       |
| Velox            | strong       | experimental | feature-flag only initially |
+------------------+--------------+--------------+-----------------------------+
```

Interpretation:
- `strong`: production candidate with broad core coverage
- `partial`: usable with fallback visibility
- `experimental`: disabled by default in production

---

## 4) Capability Dimensions

Evaluation axes:
- Logical operator kind
- Expression support
- Data types
- File format requirements
- Runtime constraints (memory, spill, sort strategy)

Capability key shape:
`(backend, device, operator, expr_class, data_type_family, format_context)`

---

## 5) Fallback Rules

Rule order:
1. Try selected backend/device
2. If `FALLBACK_CPU`, keep backend and downgrade device to CPU
3. If `ROUTE_BACKEND`, reroute fragment only as fallback when selected backend cannot execute
4. If `UNSUPPORTED`, fail in strict mode or fail with actionable warning in warn mode

Never silent:
- Any non-native execution must emit reason codes.

Fallback-only cross-backend rule:
- `ROUTE_BACKEND` is a degradation/fallback state, not an optimization state.

---

## 6) Reason Codes

Required fallback reason taxonomy:
- `unsupported_operator`
- `unsupported_expression`
- `unsupported_datatype`
- `unsupported_format_feature`
- `gpu_unavailable`
- `gpu_memory_pressure`
- `backend_unavailable`

Reason severity:
- `error`: unsupported_operator, unsupported_datatype, backend_unavailable
- `warning`: unsupported_expression, unsupported_format_feature
- `resource`: gpu_unavailable, gpu_memory_pressure

Reason hierarchy for diagnosis:
1. availability (backend/device not ready)
2. capability (operator/expression/type)
3. resource (memory/pressure)

---

## 7) Explain/Profile Requirements

Each plan fragment shows:
- chosen backend
- chosen device
- support level
- reason code when non-native

---

## 8) Quality Gates

- Matrix is versioned and tested per release.
- New backend/device pair cannot default to `auto` until coverage threshold is met.
- Any matrix downgrade requires release note and migration warning.

GPU gating policy (v0):
- `device=auto` never chooses GPU.
- GPU route is opt-in (`device=gpu`) and requires backend status >= `opt_in`.
- `experimental` GPU backends require explicit feature flag.

Coverage thresholds:
- `certified_auto`: >= 95% core operator coverage in conformance suite
- `opt_in`: >= 80% core operator coverage with fallback diagnostics
- `experimental`: below `opt_in` threshold
