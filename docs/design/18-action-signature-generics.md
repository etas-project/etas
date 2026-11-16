# Action Signature Generics

## 1. Proposal And Implementation Status

Proposed design, 2026-11-16. This remains a language extension proposal. Its implementation is being validated on the std source migration draft branch; whole-library acceptance and release remain pending.

An action may declare inferred type and effect-row parameters between `action`
and its name. Parameters after the name retain their static selector semantics.

```etas
effect Async {
    action<T, effect E> await(task: Task<T, ![E]>) -> T;
}
```

The existing Rust registry already describes this signature with two generic
binders and zero selectors. The draft implementation now retains prefix binders separately from selectors,
preserves Type and Effect kinds, and declares `Async.await` in std source. The
source uses renamed binders to exercise positional, kind-aware ABI correspondence.

## 2. Declaration And Reference Syntax

The proposed grammar appears in [Syntax Principles](09-syntax-principles.md).
The two parameter lists may appear independently or together.

```etas
effect Examples {
    action ping() -> unit;
    action<T> echo(value: T) -> T;
    action choose<Output>(schema: Schema<Output>) -> Output;
    action<T> select<Key>(key: Key, value: T) -> T;
}
```

`echo` has one inferred signature parameter and zero selectors. `choose` has
one type selector. `select` has an inferred `T` and a selector `Key`.

Prefix parameters use the existing type parameter grammar, including checked
type bounds and `effect E` row binders. Suffix parameters use the existing
selector declaration grammar. An effect-row parameter in the suffix is an
error. Duplicate names across the two lists are errors. Existing owner-effect
parameter scope and shadowing checks apply to both lists.

Signature binders scope over payload types and the result type. Selectors scope
over the same signature. Action declarations remain signatures terminated by
`;`. A callable appearing in a payload or result can carry its own Effect row.

Request, Effect-row, handler-pattern and trace-spec syntax keep their existing
selector positions. Requests infer the prefix binders from payloads and any
available expected result type.

```etas
perform Async.await(task)
perform Examples.echo(7)
perform Examples.select<string>("key", 7)
```

The last request infers `T = i32` under the existing integer defaulting rules
and selects `Key = string`. Supplying `Async.await<i32>(task)` is a selector
arity error because `await` declares zero selectors. Additional explicit
signature-argument syntax requires a separate language decision.

## 3. Type And Effect Inference

Each request instantiates the action's signature binders with fresh variables.
The ordinary type solver unifies the payload and expected result. The existing
row solver infers and checks effect-row arguments, including an open row when
the surrounding declaration quantifies over `effect E`.

For a task with type `Task<string, ![Network]>`, awaiting it instantiates
`T = string` and `E = ![Network]`. Another request may instantiate the same
action with different binders. Per-request substitutions are retained in
checked action-call facts.

Existing kind, bound and unresolved-inference diagnostics apply. In particular,
an unresolved signature parameter must produce a diagnostic when the existing
generalization rules cannot bind it. Row variables must preserve their
constraints through solving and metadata projection. Replacing an unresolved
row with `![]` would lose effects.

An action reference consists of the resolved action identity and its static
selector arguments. Inferred signature arguments are attached to checked
request facts. `Async.await` consequently has empty `effect_args` and
`selector_param_names`, even when its payload mentions `T` and `E`.

The requested suspension action is `Async.await`. Its standard default-handler
contract has produced row `E`, instantiated using that request's substitution.
Effect inference applies the existing default-handler and explicit-handler
rules to this row. An explicit handler derives its produced effects from its
checked arms and forwarded actions. Action signature inference must retain the
distinction between the requested action and handler-produced effects.

## 4. Handler Checking

A handler arm for `Async.await` covers requests according to the action identity
and the empty selector list. Every request retains its payload, result and row
instantiation for type checking, continuation checking and runtime mediation.

An inline handler must type-check against every matching request instantiation.
A reusable handler that receives requests through an abstract computation must
meet the existing parametric handler rules. A single arm can therefore forward
or uniformly process requests with different result types. The solver must
reject a fixed resume payload when it conflicts with any covered request.

For example, a handler for the generic `Examples.echo` action can resume with
its received value for both integer and string requests. Resuming with `0`
fails when that handler also covers a string request. The handler's generic
checking must also retain effect-row constraints and any relationship between
the request row and produced effects.

The existing `resume`, `finish`, forwarding, continuation ownership, scheduler
scope, cancellation and resource rules apply to these actions. Matching and
type checking use the same checked signature throughout execution.

## 5. Compiler Representation And Pipeline

Use the existing pass manager and source, HIR, type, Effect and publication
pipelines for action compilation.

In syntax, `EffectActionDecl.type_params` records the prefix signature binders
and `selector_params` records suffix selector declarations. Record them
separately so formatting, source restoration and diagnostics preserve their
placement. Existing suffix-only declarations obtain their binders from
`selector_params`. Update consumers that currently read the duplicated suffix
binders from `type_params`.

HIR creates one action scope and lowers prefix binders first, then suffix
binders. `HirEffectActionDecl.type_params` contains the complete ordered binder
set. `selector_params` references only the suffix symbols. Resolve selector
references through their declaration mapping. The current lowering pairs
selectors with the first generic symbols, which would bind them to prefix
parameters after this extension.

Reuse `EffectActionSignature.generic_params`, including each binder's actual
Type or Effect kind, together with `effect_args`, `selector_param_names` and
selector defaults. The existing row-aware generic instantiation machinery
handles signature parameters. Preserve kind, bounds, binder correspondence and
selector subsets in package metadata, std artifacts, source restoration,
reflection and cache fingerprints. Serialized AST/HIR compatibility follows
the existing artifact schema and cache-version mechanism.

The compiler and interpreter resolve the action through checked identities and
the existing action ABI. Binder correspondence is positional and kind-aware
after checking, so renaming `T` or `E` in source preserves its meaning.

## 6. Standard Library Migration

Add the declaration to `etas-core/crates/etas_std/library/std/task/mod.es` after
the frontend supports the syntax.

```etas
@std.role("std.task.Async")
public effect Async {
    action yield() -> unit;
    action<T, effect E> await(task: Task<T, ![E]>) -> T;
}
```

The checked source owns the generic binders, their kinds and bounds, payload,
result and selector list. The existing action role retains stable execution
identity, scheduler requirements and its default-handler execution contract.
The default-handler row tail must map to the checked source Effect binder by
binder correspondence, including after renaming it.

Validate the source signature against the existing TaskAwait ABI before
publishing an artifact. Both generic binders, the `Task<T, ![E]>` payload, the
result `T`, the empty selector list and the default-handler row mapping must
agree. Retain the legacy registry during migration and run provider comparison
through the existing selection mechanism. Artifact mode must execute with its
selected checked source declarations and existing scheduler binding.

## 7. Engineering Tasks And Acceptance

Paths below are relative to the named repository. Complete the work in order.

| Task | Code locations | Required result |
| --- | --- | --- |
| 1. Parse and retain both lists | `etas-frontend` — `crates/etas_syntax/src/grammar/item.rs`, `ast/item.rs`, `format/`, `dump.rs` and retained-source visitors | Prefix Type/Effect binders and suffix selectors round-trip in their original positions. Old declarations parse and format with their existing meaning. |
| 2. Lower and resolve binders | `etas-frontend` — `crates/etas_hir/src/lower/item.rs` and action-scope resolution | Prefix and suffix binders have distinct symbols in one scope. Selector mappings remain correct when prefix binders precede them. |
| 3. Check and instantiate signatures | `etas-frontend` — `crates/etas_types/src/pipeline/signature/collect/effect_actions.rs`, `pipeline/body/collect/perform.rs`, `action_selector.rs` and `handler_request.rs`, `solver/handler.rs` | Type/Effect kinds and bounds are preserved. Each request infers its own arguments. Handler payload/resume checking covers every request instantiation and abstract reusable handlers. |
| 4. Propagate Effect rows | `etas-frontend` — `crates/etas_effects/src/infer/semantics/actions.rs`, `default_handlers.rs` and existing row substitution | Requested action coverage uses selectors. Standard default handlers substitute the request's `E`. Explicit handlers preserve their checked produced rows. |
| 5. Publish portable metadata | `etas-frontend` — `crates/etas_frontend/src/package_metadata/projection.rs`, `passes/analysis/type_check_bridge.rs`, `external_metadata_validation.rs`, `artifact/external_fingerprint.rs` and checked-source restoration | Importing a separately built action retains both binder kinds, their scope, selector subset and row relationships. Cache identity reflects these semantics. |
| 6. Migrate await declaration | `etas-core` — `crates/etas_std/library/std/task/mod.es`; `etas-frontend` — `passes/analysis/standard_library.rs` and existing source-role publication | Source await passes ABI validation with zero selectors. Renamed binders and their default-handler row mapping publish correctly. |
| 7. Validate installed execution | `etas` — `crates/etas_cli/tests/cli/std_source_sdk.rs`; `etas-interpreter` — checked-action dispatch, task handlers and checkpoint restoration | Installed artifact mode imports and executes source await with the existing scheduler. Local interception, completion, cancellation and checkpoint/resume retain their contracts. |

Extend the existing syntax, action selector, handler instantiation, task type,
Effect and installed SDK suites with these acceptance cases.

- Prefix-only, suffix-only and combined lists parse, format and restore.
- Duplicate binder names, suffix `effect E`, kind errors, unsatisfied bounds,
  invalid selector arity and unresolved inference produce useful diagnostics.
- Integer and string requests to one generic action infer independently. A
  uniform reusable handler accepts both, while an incompatible resume fails.
- Awaiting `Task<T, ![]>`, a concrete nonempty row and an open `![E]` preserves
  the corresponding result type and Effect facts. A boundary excluding a
  surviving produced effect is rejected.
- Combined prefix/suffix actions preserve selector matching. Existing
  `Query.choose<Output>`, Error and Fs selector suites pass.
- A user-defined prefix-generic action compiles and executes through the same
  mechanism. Separately compiled imports preserve its signature and handler
  behavior.
- Renaming both await binders in `.es` preserves ABI validation and default
  Effect propagation. Adding a selector or changing the payload, result or
  binder kind causes ABI validation to fail before publication.
- The installed std artifact exercises await with different task result types
  and rows, local handlers, cancellation and checkpoint/resume. Legacy/artifact
  behavior comparison passes for the covered cases.

Update migration reports with measured acceptance results. The language feature
implementation receives the version increment required by repository policy in
its implementation change set.

## 8. Draft Implementation Evidence

The migration branch parses and checks prefix, suffix and combined declarations,
independent integer/string requests, Task rows and reusable uniform handlers.
Installed SDK acceptance compiles a new generic action and forwarding flow from
`.es`, imports them in another process, and runs source `await` with integer and
string tasks. An unannotated closure yields before saving an already parked
await; a fresh process resumes the checkpoint successfully. The common scheduler
sample also agrees between legacy and artifact providers.

Effect analysis requests missing concrete request ABI types through a typed phase
handoff. The frontend appends those solved types and reruns analysis before
publishing facts; unresolved rows retain their constraints. Generic callable
templates preserve their checked declaration scope during checkpoint projection.

This evidence does not close the full std migration gate. Remaining work includes
the pre-existing outer caller Handler boundary across `task.run`, the remaining
std modules, minimal bootstrap contract review and whole-library compatibility
and performance acceptance. Release continues to default to legacy.
