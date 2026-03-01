# Semantic Layer Specification

## 1) Purpose

Define the semantic source of truth for Aurora's Python-first, Spark-compatible interface.

This layer is responsible for meaning, not execution speed. It decides what a query means before any backend-specific planning happens.

---

## 2) Design Principles

- Spark-like semantics are defined in Aurora, not delegated to any backend.
- User-selected execution controls (`engine`, `device`, `execution_profile`) are respected after semantic analysis.
- Cross-backend query fragments are fallback-only, never an optimization strategy.
- Backends are execution targets; semantic correctness is validated before lowering.

---

## 3) Semantic Contract Boundary

```
Python API / SQL text
        |
        v
Semantic Layer
  - parse
  - analyze
  - validate
  - produce AnalyzedPlan
        |
        v
Backend Router + Adapter Lowering
```

Contract rule:
- The `AnalyzedPlan` is the semantic contract boundary.
- Any backend that cannot preserve `AnalyzedPlan` semantics must fail with stable errors.

---

## 4) Logical IR Strategy (v0)

Aurora uses an Aurora-owned logical IR for semantics.

### 4.1 Why Aurora-owned IR

- Prevents backend-specific semantics from leaking into user behavior.
- Keeps backend interchangeability feasible (DataFusion, DuckDB, Polars, Velox).
- Allows one compatibility policy for Python/Spark-like behavior.

### 4.2 Core IR shape

Plan nodes (minimal v0 set):
- `Project`
- `Filter`
- `Join` (inner, left)
- `Aggregate`
- `Sort`
- `Limit`
- `Union`
- `Distinct`
- `TableScan`
- `SubqueryAlias`

Expression nodes (minimal v0 set):
- `AttributeRef`
- `Literal`
- `BinaryOp`
- `UnaryOp`
- `FunctionCall`
- `Cast`
- `IsNull` / `IsNotNull`
- `InList`

Type system (minimal v0 set):
- `Boolean`, `Int32`, `Int64`, `Float64`, `Decimal(p,s)`, `String`, `Date`, `Timestamp`, `Null`

---

## 5) SQL Parsing Strategy

### 5.1 Front-end parser

- `session.sql(...)` parses SQL in Spark dialect mode.
- Parser output is immediately converted to Aurora unresolved IR.
- No parser AST is treated as a stable internal contract.

### 5.2 Parsing policy

- Unsupported SQL constructs fail fast with `AnalysisError`.
- No best-effort rewriting that changes semantics silently.
- SQL text and DataFrame API both converge into the same unresolved IR path.

---

## 6) Analyzer Pipeline (v0)

```
UnresolvedPlan
   |
   v
Pass 1: Relation Resolution
   |
   v
Pass 2: Attribute/Name Resolution
   |
   v
Pass 3: Function + Type Resolution
   |
   v
Pass 4: Semantic Validation
   |
   v
AnalyzedPlan
```

### 6.1 Pass responsibilities

Pass 1 (`ResolveRelations`):
- Resolve tables/views through catalog abstraction.
- Expand `*` using resolved schemas.

Pass 2 (`ResolveAttributes`):
- Bind columns to stable attribute identifiers.
- Detect ambiguous references.

Pass 3 (`ResolveTypesAndFunctions`):
- Resolve builtin function signatures.
- Apply documented implicit cast rules for supported v0 subset.
- Assign nullability and type outputs for expressions.

Pass 4 (`ValidateSemantics`):
- Validate aggregate/grouping rules.
- Validate join conditions and output schema.
- Validate unresolved symbols are zero.

Output invariant:
- `AnalyzedPlan` contains no unresolved relation, attribute, function, or type.

---

## 7) Spark Compatibility Boundary

### 7.1 Compatibility target

Aurora targets a practical Spark-like subset in v0, not full Spark parity.

Guaranteed v0 domains:
- DataFrame projection/filter/join/groupby/order/limit/union/distinct
- SQL SELECT for equivalent operations
- Stable type resolution for supported type set
- Stable null semantics for supported expressions

Out of scope in v0:
- Full SQL grammar parity
- Window functions
- Correlated subqueries
- Advanced ANSI/LEGACY mode parity matrix
- Full nested complex type semantics beyond pass-through operations

### 7.2 Behavior policy

- If behavior is unspecified in v0, fail explicitly.
- No silent fallback to backend-specific semantics at semantic layer.

---

## 8) Interaction with Routing and Backends

Semantic layer runs before backend selection.

```
AnalyzedPlan
   |
   v
Capability Annotation
   |
   v
User-selected backend/device resolution
   |
   v
Backend lowering
```

Rules:
- `engine`/`device`/`execution_profile` do not alter semantic analysis.
- Backend lowering may insert casts/rewrites only if semantic equivalence is preserved.
- Cross-backend split is disabled by default; if enabled later, fallback-only with explicit reason.

---

## 9) Error Model at Semantic Boundary

Semantic layer emits stable error families:
- `AnalysisError`: unresolved names, ambiguous refs, invalid grouping, unsupported syntax
- `CapabilityError`: analyzed query cannot run on selected backend/device under policy

Error requirements:
- Include `query_id`
- Include concise reason code
- Include actionable next step (change query or change execution controls)

---

## 10) v0 Scope and Deferrals

### 10.1 Must-have v0

- Unresolved -> analyzed IR pipeline
- SQL/DataFrame unification into one semantic path
- Deterministic analyzer outputs
- Capability annotation hook after analysis

### 10.2 Deferred post-v0

- Cost-based semantic rewrites
- Extensive SQL dialect edge behavior parity
- Window/correlated subquery semantics
- Mixed-backend fragment planning
- Adaptive semantic rewrites based on runtime statistics

---

## 11) Conformance Tests (Semantic)

Required test suites:
- Resolution tests (tables, aliases, columns, star expansion)
- Type/cast tests (implicit and explicit cast matrix for v0 types)
- Null semantics tests
- Aggregate/grouping validation tests
- SQL/DataFrame equivalence tests (same semantics, same analyzed plan shape)
- Negative tests (unsupported syntax/features must fail predictably)

Test policy:
- Semantic tests are backend-agnostic.
- Backend adapters get separate lowering/execution tests.

---

## 12) Open Decisions to Track

- Exact Spark dialect version targeting for parser mode
- Formal implicit-cast matrix publication in a dedicated appendix
- Timezone and decimal overflow policy defaults for v0
