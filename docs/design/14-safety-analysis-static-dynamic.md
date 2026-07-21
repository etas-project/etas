# Safety, Analysis, and Static-Dynamic Boundaries

## 1. Design Philosophy

Etas should not try to prove every property of a multi-agent system at compile time. Agent systems interact with models, tools, files, users, networks, databases, clocks, random sources, and deployment platforms. Many of those facts are only known during execution.

The Etas design principle is:

> Make uncertainty explicit, check it conservatively at compile time, enforce it exactly at runtime, and record it as trace evidence.

This gives Etas a different safety model from ordinary host-language agent frameworks:

1. **Static analysis is conservative.**
   The compiler infers what may happen. It may reject or warn about programs whose safety depends on values that are not statically known.

2. **Runtime enforcement is exact.**
   The runtime sees concrete paths, URLs, accounts, model tool-call requests, user decisions, token counts, and handler dispatch. It can enforce the exact trace spec for the actual run.

3. **Trace is evidence.**
   The trace records what was attempted, handled, executed by a default action implementation, approved, denied, retried, replayed, or resampled. Safety is not only a pre-run check; it is also an auditable execution artifact.

4. **Effects describe behavior; trace specs constrain behavior.**
   An effect row such as `![AcademicSearch.search, ProjectWorkspace.write<"reports/**">]` says what a computation may do. A `spec ...: trace` says which of those actions are allowed, denied, or guarded by approval, sanitization, review, or other conditions.

5. **Handlers interpret actions; they do not grant authority.**
   A handler may turn `ProjectWorkspace.write` into a dry-run log entry, request approval before resuming, or convert `Error<IOError>` into a value. It does not bypass effect rows, trace-spec checks, limits, sandboxing, or trace recording.

6. **Limits are reliability contracts.**
   `limit` is not just a runtime timeout. It is a source-level resource contract that supports static rejection of unbounded non-deterministic loops and runtime enforcement of tokens, steps, cost, wall time, context, and user-defined budgets.

The result is a two-layer safety story:

```text
source semantics
  -> static may-analysis and deployment checks
  -> runtime exact enforcement
  -> semantic trace evidence
```

## 2. Why Static Analysis Is Conservative

Etas static analysis is a may-analysis: it computes an upper bound over possible behavior. This is similar in spirit to other conservative static analyses, such as borrow checking, checked exceptions, taint analysis, and effect inference.

For example:

```etas
flow MaybeSearch(debug: bool, q: string) -> string {
    if debug {
        let pages = academic.search(q);
        return summarize(pages);
    }

    return "skipped";
}
```

The compiler infers that `MaybeSearch` may perform `AcademicSearch.search`, even though a concrete run with `debug = false` does not. This is intentional. The inferred effect row is a safe upper bound for deployment, trace-spec review, and manifest generation.

When static information is not precise enough, Etas has three options:

1. reject the program or require a narrower annotation;
2. accept the program with residual runtime checks;
3. report an audit warning while relying on runtime enforcement.

The exact choice can depend on compiler mode, deployment profile, and trace-spec severity.

## 3. Static vs Dynamic Responsibilities

| Topic | Static responsibility | Dynamic responsibility | Reason for the split | Example |
|---|---|---|---|---|
| Type checking | Check flow, tool, agent, handler, record, collection, and message types. | Validate runtime-provided symbols and model-structured outputs when needed. | Most source-level types are known before execution, but models and host adapters cross trust boundaries. | A `tool Save(d: Draft) -> unit` cannot be called with a `Review`; an agent output is validated before becoming `Draft`. |
| Effect inference | Infer may-effects for flows, tools, agents, handlers, and compositions. | Record the concrete actions that occurred in one run. | Static analysis gives an upper bound; runtime observes facts. | `if debug { academic.search(q) }` infers `AcademicSearch.search` even when `debug` is false at runtime. |
| Explicit effect rows | Check that inferred effects are covered by the declared row. | Still enforce concrete actions against active authority and trace specs. | A declared row is a contract, not a proof that every action will happen. | `flow F() ![ProjectWorkspace.write<"reports/**">]` may write reports, but a particular run may write nothing. |
| Tool boundaries | Check model-callable tools have declared or inferred body effects and a visible authority boundary. | Reject, retry, or route model requests for tools outside the agent's exposed tool set. | Model tool choice is a runtime event, but the tool surface is statically declared. | An agent exposed only to `Search` cannot call `Command.run`. |
| Trace spec `+A` / `-A` | Prove definite violations and narrow allowed action sets where possible. | Match concrete action refs, paths, URLs, accounts, tenants, and arguments exactly. | Many trace-spec subjects are runtime values. | `ProjectWorkspace.write<_>` is checked exactly against `ProjectWorkspace.write<"reports/**">` at runtime. |
| Trace spec `>>` / `<<` | Check dominance, obvious missing guards, and required post-actions on analyzable paths. | Verify the concrete trace has the required approval, sanitization, audit, or review event. | Control flow and agent behavior can depend on runtime choices. | `Approval.request >> CompanyEmail.send<WorkAccount>` can be partially checked statically and exactly verified by trace. |
| Handler effect elimination | Infer which handled effects are consumed and which effects escape from the handler body. | Dispatch performed actions to the nearest scoped handler or the action's default implementation. | Handler types are statically visible, but handler execution is dynamic. | A dry-run handler consumes `ProjectWorkspace.write<_>` and produces `Log.write`. |
| Error handling | Check whether `Error<E>` is handled, transformed, returned, or allowed to escape. | Raise concrete errors and execute handler/default behavior. | Error possibilities are static; occurrence is dynamic. | `read_line()?` converts `Error<IOError>` into a value-level result at the call site. |
| Limits and budgets | Require limits for unbounded or non-deterministic loops; check declared resource contracts. | Decrement actual step, token, cost, time, context, and custom budgets. | Resource use depends on model output, tool latency, and runtime data. | A reviewer loop needs `limit Iterations(5), Tokens(50_000)`; runtime stops when the first bound is exhausted. |
| Determinism inference | Classify flows as `Deterministic` or `NonDeterministic` from their constructs and effects. | Record actual model calls, random choices, external observations, and replay/resample decisions. | Whether a flow contains non-deterministic constructs is statically approximable; concrete results are runtime facts. | A pure normalization flow can be recomputed; an agent call is replayed from trace unless explicitly resampled. |
| Approval dominance | Check that sensitive actions are guarded by dominating approval where the control-flow graph is precise enough. | Confirm approval identity, result, scope, input hash, and freshness in the trace. | Static dominance can be conservative; runtime has exact event history. | If `CompanyEmail.send` occurs after approval of a different draft hash, runtime must reject or request approval again. |
| Parallel safety | Detect obvious effect conflicts and independent branches. | Schedule, lock, retry, or serialize based on concrete resources. | Alias, tenant, and path equality may be runtime-dependent. | Two branches writing the same report path cannot be freely parallelized. |
| Loop optimization | Identify loop-invariant deterministic work and repeated stable actions. | Use cache, trace replay, or deduplication when runtime keys match. | Static analysis finds candidates; runtime confirms concrete keys and trace specs. | A repeated retrieval with the same query can be hoisted or deduplicated if the action and trace spec allow it. |
| Agent tool use | Check the declared agent tool set and the effect/action surface of each tool. | Mediate actual model tool-call requests and schema-validate arguments. | The model chooses tools dynamically. | If a model asks for a tool not listed in `@tools(...)`, the runtime blocks or asks the model to retry. |
| Observability | Derive expected trace event kinds, source spans, AIR node ids, and required metadata. | Emit concrete trace events, spans, metrics, costs, outputs, and handler decisions. | The execution path is dynamic, but the semantic event vocabulary is compiler-visible. | `perform AcademicSearch.search(q)` records attempted action, handler dispatch or default action execution, result, latency, and source span. |
| Optimization | Perform effect-aware dead flow elimination, partial evaluation, retrieval hoisting, prompt specialization, and scheduling plans when safe. | Use runtime cache, memoization, deduplication, replay, and branch-specific resampling. | Agent computations are non-deterministic and trace-spec-sensitive, so optimization must preserve effects, limits, and trace semantics. | A deployment-time `false` branch containing `LegalReviewer` can be removed; a runtime repeated search can be deduplicated. |

## 4. Trace-Spec, Effect, and Trace Judgments

A compact way to describe the design is:

```text
Γ ⊢ e : T ! ε
```

The expression `e` has type `T` and may perform effects/actions `ε`.

Authority and trace-spec checking is a separate judgment:

```text
A; Π; L; H; τ ⊢ r allowed
```

where:

| Symbol | Meaning |
|---|---|
| `A` | active authority boundary derived from effect rows, tool declarations, deployment manifest, and runtime host grants |
| `Π` | active trace specs |
| `L` | remaining limits and resource budgets |
| `H` | scoped handler stack |
| `τ` | trace prefix |
| `r` | concrete action request, such as `ProjectWorkspace.write("reports/a.md", body)` |

This separation is important:

- effect inference answers **what may happen**;
- trace-spec checking answers **whether it is allowed**;
- handler dispatch answers **how it is interpreted**;
- limit checking answers **whether enough budget remains**;
- trace answers **what actually happened**.

## 5. Static Trace-Spec Analysis As Monitor Interpretation

Etas static trace-spec analysis can be described as abstract interpretation over the program graph while carrying a trace monitor state. Equivalently, the compiler may first build an abstract program action automaton and then check it against the trace monitor. The design does not require choosing one implementation strategy yet.

Conceptually:

```text
analyze(program, trace_spec):
    Mπ = compile_trace_spec_to_monitor(trace_spec)
    abstract_interpret(program, initial_monitor_state(Mπ))
```

Equivalent automata view:

```text
P# = abstract_program_action_automaton(program)
Mπ = compile_trace_spec_to_monitor(trace_spec)
check L(P#) ⊆ L(Mπ)
```

The analysis is:

| Property | Choice | Reason |
|---|---|---|
| Flow sensitivity | Flow-sensitive | Trace specs depend on action order. `Approval.request; CompanyEmail.send` differs from `CompanyEmail.send; Approval.request`. |
| Path sensitivity | Path-insensitive by default | Simpler and scalable, but can create false positives at branch joins. |
| Context sensitivity | Summary-based for flows and tools | Reuse inferred summaries without fully inlining every call. |
| Runtime residuals | Allowed where configured | Dynamic values such as paths, URLs, tenants, and input hashes often cannot be proven statically. |

### 5.1 Compiling Trace Spec Atoms To Monitors

Programmers write declarative trace spec atoms. They do not write `Bad` states. The compiler constructs monitor states and violation states from the atom semantics.

The pipeline is:

```text
trace spec source
  -> normalize action patterns
  -> build the trace dependency graph
  -> reject unsatisfiable dependency cycles
  -> build one monitor per clause
  -> compose clause monitors by product
  -> mark violation states as Bad
```

The trace-spec dependency graph is built from causal requirements:

```text
A >> B  => A -> B
A << B  => B -> A
```

The dependency graph should be acyclic in the MVP. A cycle means the trace spec
is unsatisfiable or at least too hard to explain:

```etas
spec BadTraceSpec: trace =
    (Approval.request >> CompanyEmail.send<WorkAccount>)
    & (CompanyEmail.send<WorkAccount> >> Approval.request);
```

This yields:

```text
Approval.request -> CompanyEmail.send<WorkAccount> -> Approval.request
```

and should be rejected before monitor construction.

The compiled monitor itself may have cycles. Runtime traces are arbitrary-length event sequences, so monitors need loops for repeated legal actions.

### 5.2 Clause Semantics

Each trace spec atom compiles into a small monitor. The full trace monitor is
the product of those atom monitors.

| Trace spec atom | Monitor state | Bad condition | Accepting condition |
|---|---|---|---|
| `-A` | No extra state needed | Current action matches `A` | Every non-Bad state |
| `+A` | Active narrowed authority set | Current governed action is not covered by any active allow pattern | Every non-Bad state |
| `G >> A` | Whether guard `G` has been observed for the relevant scope | Current action matches `A` before matching `G` | Every non-Bad state |
| `G << A` | Pending obligations created by `G` | Scope exits with an undischarged `A` obligation | Non-Bad state with no pending obligations |
| Constrained action pattern | Static spec facts and residual runtime argument checks | Constraint is statically false or a residual check fails | Constraint is proven or checked |
| `limit L` | Resource summary/counter abstraction | A hard limit may be exceeded | No exceeded hard limit |

Action-pattern constraints can include type-level spec predicates, such as
`Fs.read<R ~ Within<ReportsRoot>>`, or concrete argument patterns that may leave
residual runtime checks. Spec predicates are discharged from imported spec
implementations during static analysis; they do not need a runtime value check
unless the constraint also mentions runtime data.

For a pure safety trace spec, all non-Bad monitor states are accepting. For
`<<` obligations, accepting states must also have no pending obligations at the
trace-spec boundary.

Example:

```etas
spec SafeEmail: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

The compiler constructs a monitor equivalent to:

```text
state WaitingApproval
state Approved
state Bad

WaitingApproval --Approval.request--> Approved
WaitingApproval --CompanyEmail.send<WorkAccount>--> Bad
WaitingApproval --Other--> WaitingApproval

Approved --CompanyEmail.send<WorkAccount>--> Approved
Approved --Approval.request--> Approved
Approved --Other--> Approved
```

The user wrote only the `>>` atom. `Bad` is generated from the meaning of
`before`.

### 5.3 Abstract Domain

The abstract domain is not the trace monitor itself. It is the current set of possible monitor states, combined with the rest of the program analysis state.

For an ordinary finite monitor:

```text
Dπ = Powerset(Mπ.State)
```

The full abstract state includes other analysis facts:

```text
D =
    AbstractEnv
  × AbstractValues
  × MayEffects
  × TrustLabels
  × LimitSummary
  × TraceMonitorStates
  × PendingObligations
```

For parameterized actions, the trace monitor becomes symbolic:

```etas
spec SameDraftApproval: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

The monitor must remember symbolic registers such as the approved draft identity:

```text
Waiting
  --Approval.request--> Approved(x)

Approved(x)
  --CompanyEmail.send<WorkAccount> / y == x--> Approved(x)
  --CompanyEmail.send<WorkAccount> / y != x--> Bad
```

If the compiler cannot prove `x == y`, it can either reject in strict mode or emit a residual runtime check.

### 5.4 Transfer, Join, And Widening

For ordinary statements, the trace monitor state does not change. For a performed action, the transfer function steps the monitor:

```text
transfer(perform a, S):
    a# = abstract_action(a)
    S.effects = S.effects ∪ {a#}
    S.monitor = step(Mπ, S.monitor, a#)

    if Bad ∈ S.monitor:
        reject or emit residual runtime check
```

At branch merges, monitor states join by union:

```text
join({Approved}, {WaitingApproval}) = {Approved, WaitingApproval}
```

This is the source of useful false positives:

```etas
if approved {
    std.ui.approve(req);
}

perform CompanyEmail.send(WorkAccount, draft);
```

After the branch, the monitor may be either `Approved` or `WaitingApproval`. Sending email from that joined state may reach `Bad`, so the compiler cannot prove the program safe without more information.

For finite trace monitors, no widening is needed for the trace-state component because the lattice is finite:

```text
Dπ = Powerset(Mπ.State)
```

Widening or bounded abstraction is needed for the surrounding domains:

| Domain component | Widening or abstraction |
|---|---|
| Dynamic paths, URLs, accounts, tenants | Collapse large sets to `_` or a symbolic pattern |
| Pending obligations in loops | Track `0`, `1`, or `many` pending obligations |
| Numeric resources | Use intervals and widen upper bounds when needed |
| Trace summaries | Keep bounded prefixes plus may/must summaries |
| Symbolic registers | Merge equalities into unknown when path correlation is lost |

### 5.5 Runtime Trace Check

Static trace-spec analysis uses an abstract trace. Runtime trace-spec enforcement uses the concrete semantic trace prefix:

```text
τ = [ActionAttempt(...), Approval(...), TraceSpecDecision(...), ...]
```

Before executing an action request `r` through its default implementation, the runtime checks:

```text
A; Π; L; H; τ ⊢ r allowed
```

This is the exact counterpart of the static monitor analysis. The static analysis proves what it can over all abstract paths; the runtime enforces the concrete action against the exact trace prefix, exact arguments, active handlers, active policies, and remaining limits.

### 5.6 Reject vs Residual Runtime Checks

Etas should distinguish source-level contract errors from runtime-dependent safety checks.

The guiding rule is:

> Reject immediately when the program violates a static contract or cannot be made safe by checking concrete values before the side effect. Insert a residual runtime check when the only missing fact is a concrete value that the runtime can check before the action is executed through its default implementation.

#### Static Rejection

The compiler should reject these cases immediately and ask the programmer to provide more information or restructure the program:

| Case | Reason | Example |
|---|---|---|
| Type mismatch | The program is not type-correct. | Passing `Review` to `tool Save(d: Draft)`. |
| Explicit effect row is too narrow | The programmer's declared contract is false. | `flow F() ![] { perform AcademicSearch.search(q); }` |
| Model-callable `tool` hides high-impact effects | A tool is an authority boundary exposed to a model. Its possible external actions must be visible. | A `tool Delete(...)` performs `ProjectWorkspace.write<...>` without declaring or exposing that boundary. |
| Runtime-provided symbol lacks complete metadata | The compiler cannot inspect host-language code. | A package binding for `Search(q)` has no effect row or output schema. |
| Trace dependency graph has a cycle | The trace spec is unsatisfiable or too hard to explain. | `A >> B` and `B >> A`. |
| Definite deny violation | The compiler knows the action is forbidden on every path. | `-Command.run; perform Command.run(cmd);` |
| Definite allow violation | The action ref is statically known and outside the allowed set. | `+ProjectWorkspace.write<"reports/**">; perform ProjectWorkspace.write("tmp/a.md", body);` |
| Unbounded non-deterministic loop | Waiting until runtime may consume unbounded tokens, context, cost, or time. | An agent refinement loop without `limit`. |
| Definite prompt-trust violation | Control-plane prompt injection must be blocked before deployment. | Passing `Untrusted<string>` directly to `Prompt.system(...)`. |
| Unsound handler type | The handler claims to eliminate an effect but lets it escape. | A handler typed as `![ProjectWorkspace.write<_> => []]` still delegates to the default implementation of `ProjectWorkspace.write`. |
| Violates core call-graph restrictions | The MVP can restrict dangerous recursion or model-callable boundaries. | A `tool` recursively calls an `agent` if that restriction is active. |

These errors should produce diagnostics that point to the missing evidence: a narrower type, an explicit effect row, a declared `limit`, a dominating approval, a handler type fix, trusted host-binding metadata, or a clearer trace-spec scope.

#### Residual Runtime Checks

The compiler may insert residual runtime checks when static analysis cannot prove the property, but the runtime can check it exactly before the external action occurs:

| Case | Runtime fact | Example |
|---|---|---|
| Dynamic path, URL, account, or tenant | Concrete action argument | Check `ProjectWorkspace.write(path, body)` against `ProjectWorkspace.write<"reports/**">`. |
| Approval scope | Concrete input hash, actor, decision, and freshness | Check that the same draft hash was approved before `CompanyEmail.send`. |
| `after` obligation | Concrete scope-exit trace | Check that `Audit.log` occurred after `StripePayment.charge`. |
| Residual action-pattern predicate | Concrete values | Check `request.tenant == resource.tenant`. |
| Resource limit | Runtime token, step, cost, wall-time, or context debit | Stop before exceeding `Tokens(50_000)`. |
| Model tool-call request | Actual tool requested by the model | Reject or retry if the model asks for a tool outside `@tools(...)`. |
| Structured output validation | Concrete model output | Decode and validate the output as `Draft`. |
| Runtime-provided symbol result | Concrete host adapter result | Validate `Array<Untrusted<Page>>` returned by `BrowserSearch`. |
| Handler dispatch | Dynamic handler stack | Route `ProjectWorkspace.write` to a scoped dry-run handler if present. |
| Trace redaction/export | Concrete secret values and export target | Redact `Secret<T>` before exporting a trace. |

For example:

```etas
spec ReportOnly: trace =
    +ProjectWorkspace.write<"reports/**">;

flow Save(path: Path, body: string) ~ ReportOnly {
    perform ProjectWorkspace.write(path, body);
}
```

If the compiler cannot prove that `path` is always under `reports/**`, it can insert a residual check before executing `ProjectWorkspace.write` through its default implementation. In a stricter deployment mode, the compiler can instead reject and ask the programmer to use a narrower type or constructor:

```etas
let path: ReportPath = project.workspace.report_path(name);
perform ProjectWorkspace.write(path, body);
```

Recommended modes:

| Mode | Possible trace-spec violation | Use case |
|---|---|---|
| Strict | Reject and require static evidence. | High-assurance deployment, payment, publishing, secrets, shell, production writes. |
| Normal | Insert residual runtime checks when exact dynamic checking is possible. | Most production applications. |
| Audit | Warn, enforce dynamically, and preserve trace evidence. | Prototyping, migration, trace-spec discovery. |

Definite violations should be rejected in all modes. Residual runtime checks are for unknown facts, not for known-bad programs.

### 5.7 Trace-Spec Composition And Conflict Detection

Etas trace-spec scopes should compose by monotonic narrowing:

```text
effective_trace_spec = outer_trace_spec ∧ inner_trace_spec
```

An inner trace-spec scope can restrict behavior further, but it cannot grant authority
or override an outer denial. This keeps the safety model explainable and gives a
useful monotonicity property:

```text
adding a trace spec can only reduce the accepted trace language
```

Equivalently:

```text
Π ⊆ Π'  implies  accepted_traces(Π') ⊆ accepted_traces(Π)
```

Recommended composition rules:

| Trace spec component | Composition rule | Example |
|---|---|---|
| `-A` | Union; any matching denial wins. | Outer `-Command.run<_>` still applies inside every inner scope. |
| `+A` | Intersect active allowed sets for the same governed action family. | Outer `+ProjectWorkspace.write<"reports/**">` plus inner `+ProjectWorkspace.write<"reports/private/**">` yields the private reports subset. |
| `G >> A` | Accumulate requirements. | If outer requires approval and inner requires sanitization before publish, both are required. |
| `G << A` | Accumulate pending obligations. | A payment can require both `Audit.log` and `Receipt.store` after charge. |
| Action-pattern constraints | Conjoin constraints. | `tenant == request.tenant` and `region == "us"` must both hold. |
| `limit` | Conjoin limits; for the same dimension use the stricter bound. | `Tokens(10_000)` inside `Tokens(50_000)` yields `Tokens(10_000)`. |

The compiler should check two levels:

1. **Trace-spec consistency.** The active trace specs themselves are contradictory
   or unsatisfiable.
2. **Program-trace compatibility.** The active trace specs are valid, but the
   program may perform actions that violate them.

#### 5.7.1 Trace-Spec Conflicts

These conflicts can be detected before or during static trace-spec analysis:

| Conflict | Example | Handling |
|---|---|---|
| Temporal dependency cycle | `A >> B` and `B >> A` | Reject the trace spec set as unsatisfiable. |
| Empty allow intersection | Outer `+ProjectWorkspace.write<"reports/**">`, inner `+ProjectWorkspace.write<"tmp/**">` for the same scope | Warn or reject the scope; reject if the body may require that action family. |
| Required guard is denied | `-Approval.request` and `Approval.request >> CompanyEmail.send<WorkAccount>` | Reject any scope where `CompanyEmail.send<WorkAccount>` may occur; optionally warn that the requirement is impossible. |
| Required after action is denied | `-Audit.log` and `Audit.log << StripePayment.charge<Account>` | Reject any scope where `StripePayment.charge<Account>` may occur. |
| Self-dependency with no meaningful witness | `Approval.request >> Approval.request` | Reject as ill-formed or require an explicitly different scoped event. |
| Predicate contradiction | Action constraints imply both `tenant == "a"` and `tenant == "b"` | Reject when statically decidable; otherwise insert residual runtime contradiction checks if useful. |
| Limit contradiction | `Iterations(0)` around a body that must execute at least once | Reject if the execution requirement is statically known; otherwise runtime limit enforcement stops before the body. |
| Handler evidence conflict | A trace spec requires evidence for an action, but a scoped handler hides all evidence for that action | Reject or require the handler to emit an accepted evidence event. Handled actions must remain visible in trace. |

The trace dependency graph should be checked separately from the compiled
automaton. The dependency graph is expected to be acyclic in the MVP. The
compiled trace automaton may still contain loops because traces can be
arbitrary length and legal actions may repeat.

#### 5.7.2 Program-Trace Conflicts

A trace spec can be internally consistent but still reject a program:

```etas
spec NoNetwork: trace =
    -Network;

flow F() ~ NoNetwork {
    academic.search(q);
}
```

`NoNetwork` is valid. `F` is not compatible with it because the inferred action
summary includes `AcademicSearch.search`, and `AcademicSearch.search <= Network`.

For possible violations caused by imprecise static facts, use the checking mode
from [Reject vs Residual Runtime Checks](#56-reject-vs-residual-runtime-checks):

| Static result | Strict mode | Normal mode | Audit mode |
|---|---|---|---|
| Definite violation | Reject | Reject | Reject |
| Possible violation with exact pre-action runtime check | Reject and request stronger evidence | Insert residual runtime check | Warn, enforce dynamically, and trace |
| Possible violation with no safe runtime interception point | Reject | Reject | Reject |

#### 5.7.3 Caller-Side Trace-Spec Checking

Trace-spec scopes should not mutate callee definitions. A flow or tool summary
is computed independently, then consumed by the caller's active trace-spec
context.

```etas
flow AskApproval(draft: Draft) {
    approve(draft);
}

flow Send(draft: Draft) {
    perform CompanyEmail.send(WorkAccount, draft);
}

flow Publish(draft: Draft) ~ SafeEmail {
    AskApproval(draft);
    Send(draft);
}
```

The summaries are composed in caller order:

```text
Summary(AskApproval) = Approval.request
Summary(Send)        = CompanyEmail.send<WorkAccount>
Summary(Publish)     = Summary(AskApproval) ; Summary(Send)
```

`SafeEmail` is checked against the composed caller trace summary. The trace spec is
not implicitly pushed into the definitions of `AskApproval` or `Send`. This is a
summary-based interprocedural analysis, not full inlining and not dynamic global
trace-spec propagation.

## 6. Soundness Targets

Etas should aim for sound but pragmatic safety properties.

### 6.1 Effect Soundness

If the compiler infers:

```text
Γ ⊢ e : T ! ε
```

then every action executed through a default implementation in a runtime trace of `e` should be covered by `ε`, unless it crosses an explicitly trusted host-binding boundary recorded in compiler/runtime metadata.

### 6.2 Trace-Spec Safety

If static trace-spec checking succeeds with residual runtime checks, and the
runtime enforces those checks for every concrete action request, then no action
executed through a default implementation violates the active trace spec.

### 6.3 Handler Transparency

If a handler consumes an action, the trace must still record the attempted action and the handler decision. Handling an action may remove it from the escaping effect row, but it must not erase audit evidence.

### 6.4 Limit Safety

If a computation is accepted under a limit contract, runtime execution must stop, suspend, or fail with a budget error before exceeding any active hard limit.

## 7. Handling False Positives

Conservative static analysis may reject programs that are safe for all intended runtime inputs. This is not unique to Etas. Rust's borrow checker, checked exceptions, static information-flow systems, refinement checkers, and taint analyzers all make conservative approximations.

Etas should provide practical escape and precision mechanisms without weakening the core model:

| Mechanism | Purpose | Example |
|---|---|---|
| More precise inference | Reduce false positives through constant propagation, control-flow analysis, region-sensitive effect inference, and path-sensitive trace-spec checks. | Prove a deployment-time disabled branch cannot call `CompanyEmail.send`. |
| Scoped narrowing | Let users make authority and trace-spec scope smaller and clearer. | Move the sensitive operation into a helper flow and annotate it with `~ (+ProjectWorkspace.write<"reports/**">)`. |
| Runtime residual checks | Accept code when exact values are dynamic but enforce the final decision at runtime. | Check concrete path glob membership before `ProjectWorkspace.write(path, body)`. |
| Handler elimination | Let users intercept high-impact actions and turn them into safer effects. | Handle `ProjectWorkspace.write<_>` as dry-run `Log.write`. |
| Trusted host-binding metadata | Allow integration with host systems while marking the boundary explicitly. | A package manifest declares `BrowserSearch` with type, schema, and `![AcademicSearch.search]`. |
| Audit mode | Permit uncertain code in lower-assurance deployments while preserving trace evidence and warnings. | Run a prototype with dynamic trace-spec enforcement and compiler warnings. |

The default should be strict for high-impact actions such as shell commands, file writes, publishing, email, payment, identity, deployment, and secret access. Lower-risk actions can use warnings or residual checks depending on deployment policy.

## 8. Relationship To Other Design Documents

This document defines the safety and analysis boundary. Detailed mechanics live in the rest of the design:

| Topic | Document |
|---|---|
| User-facing syntax | [Syntax Principles](09-syntax-principles.md) |
| Type system and `Error<E>` | [Type System and Errors](05-type-system-and-errors.md) |
| Effect actions, handlers, and inference | [Effect System and Inference](06-effect-system-and-inference.md) |
| Flows, limits, policies, and protocols | [Flows, Human Gates, and Protocols](04-flows-human-gates-and-protocols.md) |
| Formal core and abstract interpretation | [Formal Core, Static Analyses, and PL Context](08-formal-core-static-analyses-and-pl-context.md) |
| IR, trace, replay, and runtime execution | [IR Stack, FIR, and AIR](11-agent-intermediate-representation.md) |
