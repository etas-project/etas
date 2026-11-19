# Legacy Flow Implementation

The accepted contracts are in [Standard Library, sections 5 through 9](../design/19-standard-library.md#5-numeric-helpers).
This document records engineering work pending as of 2026-11-16. Publishing the
design does not establish runtime or provider acceptance.

## 1. Work Items

Paths below are relative to the named repository.

| Work item | Code locations | Required result |
| --- | --- | --- |
| Numeric type evidence | `etas-core` — `crates/etas_std/library/std/math/complete.es`, `src/intrinsic/source_role.rs`; `etas-frontend` — `crates/etas_types` spec evidence, numeric operator constraints and metadata | Declare source-owned `Number` with its closed primitive numeric contract. Check generic bodies and imported evidence through the ordinary type pipeline. |
| Math implementations | `etas-core` — `crates/etas_std/library/std/math/complete.es`, `src/modules/core/math.rs`; `etas-interpreter` — `src/value/numeric.rs`, `src/eval/operator_expr.rs`, numeric primitive dispatch under `crates/etas_interpreter` | Correct `min/max` to binary signatures, implement source bodies and explicitly bound primitive helpers. Match widths, overflow, NaN, infinity and signed-zero semantics. Preserve the legacy baseline and record the API correction. |
| Group implementation | `etas-core` — `crates/etas_std/library/std/agent/group/complete.es`, `src/modules/agent/group.rs` | Add the source error, stop reason and result types. Implement the checked generic callback loop with its row-polymorphic signature. Reuse ordinary calls, loops and error propagation. |
| Trace labels and annotation lowering | `etas-core` — `crates/etas_std/library/std/runtime/trace/complete.es`, `src/modules/runtime/trace.rs`, trace-label payload and codec contracts; `etas-frontend` — annotation resolution and checked agent metadata; `etas-interpreter` — agent trace-plan emission and checkpoint codecs | Implement the pure opaque composite-label binding and source wrapper. Resolve annotated callable references to typed stage labels. Retain structured identities and emit actual runtime events independently of the logical plan. |
| Typed security strategies | `etas-core` — `crates/etas_std/library/std/security/declassify/complete.es`, `src/modules/security/declassify.rs`, `src/modules/security/trust.rs`; `etas-frontend` — trust-wrapper checking, metadata projection and std ABI validation | Replace identity signatures with typed input/output strategy contracts and errors. Bind only validated strategy handles and checked bodies. Validate nested trust and protected construction paths. |
| Secret release mediation | `etas-core` — Secret source action declarations, execution-role descriptors, `crates/etas_host/src/context/authority/`, trace and binding protocols; `etas-interpreter` — checked-action mediation, protected strategy execution and checkpoint restoration | Add `Secret.declassify`, strategy-scoped permission checks, protected payload access, output validation and redacted audit outcomes. Preserve the checks for custom handlers and restored executions. |
| Installed provider acceptance | `etas` — `crates/etas_cli/tests/cli/std_source_sdk.rs`, `docs/migration/std-bootstrap-api.json`; integration suites in the other repositories | Install and execute the corrected API offline in a moved SDK. Cover separately compiled imports and checkpoint/resume. Record actual results and deliberate signature changes in the migration report. |

The numeric contract is a prerequisite for checked generic math bodies. Source
group orchestration and label construction can proceed independently. Security
strategy declarations, protected bindings and authority mediation must be
integrated before reporting declassification support.

## 2. Acceptance Cases

- For every integer width, test `abs(0)`, positive and negative values, unsigned
  maxima and signed minima. Signed minima produce the same checked overflow
  fault as negation. Test matching and mixed operand types for `min/max`.
- For both float widths, test operand order, equal values, infinities, both
  signed zeros, one NaN and two NaNs with distinct payloads. Preserve the
  specified selected bits. Check compiler evaluation and interpreter agreement.
- Reject nonnumeric and unconstrained generic arguments. Accept bounded generic
  helpers and separately compiled imports of the numeric spec and flows.
- Run three callbacks for five turns and observe `0, 1, 2, 0, 1`. Verify empty
  roster rejection, zero turns, initial predicate success, early stop, final-turn
  predicate priority and the exact returned state, count and reason.
- Verify callback row propagation with pure, Agentic and other effectful
  callbacks. Check inherited handlers, errors, limits and cancellation. Pause
  during a callback and restore without repeating a completed turn.
- Construct empty, singleton, duplicate and nested virtual-stage composites.
  Check annotation callable resolution and import aliases, opaque-label codecs,
  structured trace identity and absence of synthetic model/tool/usage events.
- Sanitize using explicit accepting, transforming and rejecting strategies.
  Reject absent or mismatched strategy evidence. Confirm secret input is
  rejected by the sanitizer signature and that sanitized data cannot acquire
  instruction-channel trust through wrapper nesting.
- Declassify under granted, denied and missing authority. Test an empty reason,
  unknown strategy, changed revision, mismatched types, rejected output and an
  explicitly permitted identity strategy. Preserve nested-secret restrictions.
- Verify that direct wrapper constructors, local handlers and restored strategy
  handles cannot bypass protected release checks. Confirm audit metadata and
  typed failures without secret payloads in ordinary traces or errors. Preserve
  existing secret-storage policy and restored strategy-authority checks across
  checkpoint/resume.
- Exercise all seven exports through installed source artifacts. Preserve
  legacy release availability during migration, classify signature differences
  and verify shared execution semantics wherever both providers support them.

Use existing suites and normal entry points. Mark a migration item complete
only after its installed execution checks pass. The corrected public API and
supporting contracts require a minor-version increment under repository policy
in the implementation change set.
