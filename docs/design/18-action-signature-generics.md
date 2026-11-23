# Action Signature Generics

## 1. Proposal Scope

This remains a language extension proposal, not a replacement for the current
[source grammar](09-syntax-principles.md). It separates inferred signature
parameters from explicit action selectors; adoption requires synchronizing
that grammar and the standard action declarations.

An action may declare inferred type and effect-row parameters between `action`
and its name. Parameters after the name retain their static selector semantics.

```etas
effect Async {
    action<T, effect E> await(task: Task<T, ![E]>) -> T;
}
```

## 2. Declaration And Reference Syntax

In this proposal the two parameter lists may appear independently or together.

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

## 5. Portable Contract And Conformance

This proposal adds no new authority or continuation privilege. If adopted,
checked package metadata and code restoration must preserve these facts:

| Contract | Requirement |
|---|---|
| Binder identity | Keep signature binders separate from selectors, including kinds, bounds and owner scope. Renaming a binder does not change its meaning. |
| Request instantiation | Infer fresh signature arguments per request; retain the solved payload, result and produced-row relationship. |
| Matching | Effect rows and handler patterns match action identity plus static selectors, not inferred payload instantiations. |
| Generic handlers | Check uniform behavior for every admitted instantiation, not only requests observed in one run. |
| Default handling | Substitute the request's effect-row arguments into the default-handler contract without erasing requested-action metadata. |
| Separate compilation | Preserve the same signature and selector meaning through imports, reflection, cached code and checkpoint restoration. |
| Diagnostics | Reject duplicate binders, invalid kinds/bounds, selector arity mismatches, unresolved inference and incompatible resume values. |

Conformance tests cover prefix-only, suffix-only and combined declarations;
integer/string requests to one generic action; empty, concrete and open task
rows; reusable handlers; and separately compiled imports. Release coverage,
migration branches, artifact versions and engineering task assignments are not
part of this language proposal.
