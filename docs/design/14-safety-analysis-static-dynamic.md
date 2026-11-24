# Safety Contracts

## 1. Checked Behavior

Types classify values; effect rows bound escaping obligations; requested-action
facts retain handled operations; traces preserve their order and outcomes.
None grants authority. Effective permission is restricted by every applicable
trace spec, deployment grant, sandbox, approval and resource bound.

| Fact | Required behavior |
|---|---|
| May-effect summary | Conservatively includes reachable operations, not only the path taken in one run. |
| Action trace | Ordered attempted/completed events with result, request/resource identity and applicable scope; not a sequence of whole program states. |
| Handler | Interprets an operation without deleting its attempted-action evidence or widening authority. |
| Call/delegation | Preserves caller restrictions through flows, tools, agent requests and imported libraries. |
| Constructor injection | Performed providers remain visible; later stored-field reads do not resolve them again. |
| Scope exit | Inner restrictions do not mutate the outer scope or callee definition. Completed evidence remains subject to its validity/scope rules. |

## 2. Trace Constraints

Source operators and local allow-set formation are defined in
[Trace Specs](05-type-system-and-errors.md#154-trace-specs).
`G >> A` and `A << G` give local permission to both operands and require
matching successful G evidence before each A. A denied, pending, failed or
unrelated approval is not a witness. An empty trace is allowed; the operator
does not require the target action to occur.

Evidence is associated with the relevant request, resource, actor, input and
freshness constraints, not a global "approved once" flag. Unrelated actions
remain subject to authority/denial checks. Type-spec predicates use checked
evidence; dynamic payload restrictions are checked before the governed action.

Temporal dependencies must be acyclic, including dependencies introduced by
simultaneously active constraints. This restriction does not prohibit loops in
program execution or repeated legal events.

## 3. Scope Composition

```text
effective constraints = outer constraints AND inner constraints
```

Adding a scope can only narrow accepted behavior. Denials accumulate, applicable
allow sets intersect, prerequisites accumulate and budgets retain ancestor
charges. For `S | T`, one complete alternative must accept; do not mix
permissions from one branch with ordering evidence from another.

Callee behavior is checked under the caller's active constraints without
rewriting the callee definition. Effect sets alone cannot establish ordering:
imported/higher-order behavior must retain an adequate ordered contract or
receive an enforceable dynamic check. Unknown behavior cannot silently pass.

## 4. Rejection And Dynamic Checks

Reject a static contract violation or any uncertainty that cannot be checked
before the protected behavior. A concrete-value uncertainty may use an exact
pre-action check; this never converts a known violation into an accepted one.

### 4.1 Static Rejection

| Case | Reason | Example |
|---|---|---|
| Type mismatch | The program is not type-correct. | Passing `Review` to `tool Save(d: Draft)`. |
| Explicit effect row is too narrow | The programmer's declared contract is false. | `flow F() ![] { perform AcademicSearch.search(q); }` |
| Model-callable `tool` hides high-impact effects | A tool is an authority boundary exposed to a model. Its possible external actions must be visible. | A `tool Delete(...)` performs `ProjectWorkspace.write<...>` without declaring or exposing that boundary. |
| Runtime-provided symbol lacks complete metadata | The compiler cannot inspect host-language code. | A package binding for `Search(q)` has no effect row or output schema. |
| Trace dependency graph has a cycle | The trace spec is unsatisfiable or too hard to explain. | `A >> B` and `B >> A`. |
| Definite deny violation | The compiler knows the action is forbidden on every path. | `-Command.run; perform Command.run(cmd);` |
| Definite allow violation | The action ref is statically known and outside the allowed set. | `+ProjectWorkspace.write<"reports/**">; perform ProjectWorkspace.write("tmp/a.md", body);` |
| Unbounded non-deterministic loop | Waiting until runtime may consume unbounded tokens, context, cost, or time. | An agent refinement loop without `@limits(...)`. |
| Definite prompt-trust violation | Control-plane prompt injection must be blocked before deployment. | Passing `Untrusted<string>` directly to `Prompt.system(...)`. |
| Unsound handler type | The handler claims to eliminate an effect but lets it escape. | A handler typed as `![E => []]` raises an unhandled `Error<IOError>` from its arm. |
| Unsafe delegation or capture | Calls through tools/roles preserve transitive behavior, scopes and authority; no captured mutable caller state crosses requests. | Reject a tool whose delegated agent exceeds its boundary, or a cross-request handler capturing caller-local `var`. |
| Active ownership/turn violation | Do not erase reference scopes or exclusive updating turns. | Reject an escaped child reference or a proven unavoidable closed wait cycle; uncertain wait cycles retain runtime detection. |

### 4.2 Dynamic Checks

| Case | Runtime fact | Example |
|---|---|---|
| Dynamic path, URL, account, or tenant | Concrete action argument | Check `ProjectWorkspace.write(path, body)` against `ProjectWorkspace.write<"reports/**">`. |
| Approval scope | Concrete input hash, actor, decision, and freshness | Check that the same draft hash was approved before `CompanyEmail.send`. |
| Prior-action obligation | Concrete action and matching earlier evidence | For `Audit.log << StripePayment.charge`, check a matching charge before each audit action. |
| Residual action-pattern predicate | Concrete values | Check `request.tenant == resource.tenant`. |
| Resource limit | Runtime token, step, cost, wall-time, or context debit | Stop before exceeding `Tokens(50_000)`. |
| Model tool-call request | Actual tool requested by the model | Reject or retry if the model asks for a tool outside `@tools(...)`. |
| Structured output validation | Concrete model output | Decode and validate the output as `Draft`. |
| Runtime-provided symbol result | Concrete host adapter result | Validate `Array<Untrusted<Page>>` returned by `BrowserSearch`. |
| Handler dispatch | Dynamic handler stack | Route `ProjectWorkspace.write` to a scoped dry-run handler if present. |
| Trace redaction/export | Concrete secret values and export target | Redact `Secret<T>` before exporting a trace. |

| Mode | Unknown but dynamically checkable fact |
|---|---|
| Strict | Reject and require stronger static evidence. |
| Normal | Enforce a residual pre-action check. |
| Audit | Warn, enforce dynamically and retain trace evidence. |

Definite violations and missing safe interception points are rejected in every
mode. An explicit effect-row bound is never relaxed merely because a programmer
believes a branch unreachable. Narrow inputs, factor behavior or provide valid
evidence; do not disable the boundary.

## 5. Constraint Conflicts

| Conflict | Example | Handling |
|---|---|---|
| Temporal dependency cycle | `A >> B` and `B >> A` | Reject the trace spec set as unsatisfiable. |
| Empty allow intersection | Outer `+ProjectWorkspace.write<"reports/**">`, inner `+ProjectWorkspace.write<"tmp/**">` for the same scope | Warn or reject the scope; reject if the body may require that action family. |
| Required guard is denied | `-Approval.request` and `Approval.request >> CompanyEmail.send<WorkAccount>` | Reject any scope where `CompanyEmail.send<WorkAccount>` may occur; optionally warn that the requirement is impossible. |
| Required prior action is denied | `-StripePayment.charge<Account>` and `Audit.log << StripePayment.charge<Account>` | Reject any scope where `Audit.log` may occur without another valid matching charge. |
| Self-dependency with no meaningful witness | `Approval.request >> Approval.request` | Reject as ill-formed or require an explicitly different scoped event. |
| Predicate contradiction | Action constraints imply both `tenant == "a"` and `tenant == "b"` | Reject when statically decidable; otherwise insert residual runtime contradiction checks if useful. |
| Limit contradiction | `Iterations(0)` around a body that must execute at least once | Reject if the execution requirement is statically known; otherwise runtime limit enforcement stops before the body. |
| Handler evidence conflict | A trace spec requires evidence for an action, but a scoped handler hides all evidence for that action | Reject or require the handler to emit an accepted evidence event. Handled actions must remain visible in trace. |

Constraint consistency and program compatibility are separate: a valid
`-Network` constraint still rejects a reachable network action. Guard success,
authority and budget exhaustion are protected execution facts, not claims
established by constructing an ordinary record.

## 6. Guarantees

| Contract | Guarantee |
|---|---|
| Effects | Checked observable behavior stays within its declared/inferred boundaries, including transitive calls and handler-produced actions. |
| Trace constraints | A governed action cannot execute before its required evidence and dynamic argument checks succeed. |
| Handler transparency | Interpretation cannot erase requested behavior or grant missing authority. |
| Budgets | Nested work is charged to applicable shared accounts; retry, recovery and `?` do not reset them. |
| Trust | Prompt/control-plane and secret-release requirements survive abstraction and serialization boundaries. |

Internal analysis algorithms, monitor representation and compiler data
structures are outside this specification.
